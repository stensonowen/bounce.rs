use std::{io, str};
use futures::{Poll, StartSend};
use futures::{Async, Stream, Sink, AsyncSink};
use tokio_io::codec::{Encoder, Decoder};
use bytes::BytesMut;

pub mod line;
use self::line::Line;

#[derive(Default)]
pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = Line;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Line>> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            let line = buf.split_to(i);
            buf.split_to(1);
            match str::from_utf8(&line) {
                Ok(s) => Ok(Some(Line::from_str(s))),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "bad utf8")),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for LineCodec {
    type Item = Line;
    type Error = io::Error;

    fn encode(&mut self, line: Line, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(line.to_string().as_bytes());
        buf.extend(b"\n");
        Ok(())
    }
}


pub struct PingPong<T> {
    upstream: T,
    response: Option<Line>,
    // TODO: `away` state?
    //  others? 
}

impl<T> PingPong<T> {
    pub fn new(t: T) -> Self {
        PingPong {
            upstream: t,
            response: None,
        }
    }
}

impl<T> Stream for PingPong<T>
    where T: Stream<Item=Line, Error=io::Error>,
          T: Sink<SinkItem=Line, SinkError=io::Error>
{
    type Item = Line;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Line>, io::Error> {
        // Poll the upstream transport
        match try_ready!(self.upstream.poll()) {
            Some(Line::Ping{ orig: ref msg }) => {
                // Intercept pings
                println!("\tGETTING PING");
                let resp = Line::pong_from_ping(msg);
                self.response = Some(resp);
                self.poll_complete()?;

                let poll = try_ready!(self.upstream.poll());
                // does this actually work? never tested it
                println!("NOTE: {:?}", poll);
                Ok(Async::Ready(poll))
            },

            // Final output:
            m => Ok(Async::Ready(m)),
        }
    }
}

impl<T> Sink for PingPong<T>
    where T: Sink<SinkItem=Line, SinkError=io::Error>
{
    type SinkItem = Line;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Line) -> StartSend<Line, io::Error> {
        // Only accept the write if there are no pending pong
        match self.response {
            Some(_) => Ok(AsyncSink::NotReady(item)),
            None => self.upstream.start_send(item),
        }
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        if let Some(pong) = self.response.take() {
            println!("\tSENDING PONG");
            self.upstream.start_send(pong)?;
        }
        self.upstream.poll_complete()
    }
}
