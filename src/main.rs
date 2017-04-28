#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;

use std::{io, str};
use std::net::ToSocketAddrs;

use futures::{stream, Future, Poll, StartSend};
use futures::{Async, Stream, Sink, AsyncSink};
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::AsyncRead;
use bytes::BytesMut;

// All lines are sent to a client
pub enum Line {
    // PMs are logged sometimes trigger alerts
    PrivMsg { src: String, dst: String, text: String, orig: String },
    // Metadata is not logged 
    Meta { orig: String },
}

impl Line {
    fn new_meta(s: &str) -> Self {
        Line::Meta{ orig: s.to_string() }
    }
}

impl std::string::ToString for Line {
    fn to_string(&self) -> String {
        match *self {
            Line::PrivMsg { orig: ref o, .. } => o,
            Line::Meta { orig: ref o, .. } => o,
        }.clone()
    }
}

impl std::str::FromStr for Line {
    type Err = ();

    fn from_str(input: &str) -> Result<Self,Self::Err> {
        // TODO: adhere closer to the RFC
        // e.g. `:Angel!wings@irc.org PRIVMSG Wiz message goes here`
        let mut parts = input.splitn(4, ' ');
        let src = parts.nth(0);
        let op  = parts.nth(0);
        let dst = parts.nth(0);
        let msg = parts.nth(0);
        if let (Some(m), Some(d), Some(o), Some(s)) = (msg, dst, op, src) {
            if o == "PRIVMSG" || o == "NOTICE" {
                let i = if s.starts_with(':') { 1 } else { 0 };
                let j = s.find(':').unwrap_or(s.len()-1);
                let src_fixed = &s[i..j];
                let msg_fixed = if m.starts_with(':') { &m[1..] } else { m };
                // TODO: do something with "\r\n" ending?
                Ok(Line::PrivMsg {
                    src: src_fixed.to_string(),
                    dst: d.to_string(),
                    text: msg_fixed.to_string(),
                    orig: input.to_string(),
                })
            } else {
                Ok(Line::new_meta(input))
            }
        } else {
            Ok(Line::new_meta(input))
        }
    }
}

#[derive(Default)]
pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<String>> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            let line = buf.split_to(i);
            buf.split_to(1);
            match str::from_utf8(&line) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "bad utf8")),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, msg: String, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(msg.as_bytes());
        buf.extend(b"\n");
        Ok(())
    }
}


pub struct PingPong<T> {
    upstream: T,
    response: Option<String>
    // TODO: `away` state?
    //  others? 
}

impl<T> PingPong<T> {
    fn new(t: T) -> Self {
        PingPong {
            upstream: t,
            response: None,
        }
    }
}

impl<T> Stream for PingPong<T>
    where T: Stream<Item = String, Error = io::Error>,
          T: Sink<SinkItem = String, SinkError = io::Error>
{
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<String>, io::Error> {
        // Poll the upstream transport
        match try_ready!(self.upstream.poll()) {
            Some(ref msg) if msg.starts_with("PING ") => {
                // Intercept pings
                println!("\tGETTING PING");
                let resp = msg.replacen("PING", "PONG", 1);
                self.response = Some(resp);
                self.poll_complete()?;

                let poll = try_ready!(self.upstream.poll());
                // does this actually work? never tested it
                println!("NOTE: {:?}", poll);
                Ok(Async::Ready(poll))
            }
            // Final output:
            m => Ok(Async::Ready(m)),
        }
    }
}

impl<T> Sink for PingPong<T>
    where T: Sink<SinkItem = String, SinkError = io::Error>
{
    type SinkItem = String;
    type SinkError = io::Error;

    fn start_send(&mut self, item: String) -> StartSend<String, io::Error> {
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


fn main() {
    let conn_msg: Vec<Result<String, io::Error>> = vec![
        Ok("USER a b c d".to_string()), 
        Ok("NICK qjkxk".to_string()),
        Ok("JOIN #test".to_string())
    ];

    //let addr = "irc.freenode.org:6667".to_socket_addrs().unwrap().next().unwrap();
    let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let stream = TcpStream::connect(&addr, &handle);
    let listen = stream.and_then(|socket| {
        let transport = PingPong::new(socket.framed(LineCodec));
        let (sink, stream) = transport.split();
        sink.send_all(stream::iter(conn_msg))
            .and_then(|_| {
                stream.for_each(|i| {
                    println!("SAW: `{:?}`", i);
                    futures::future::ok(())
                })
            })
    });
    // empty tuple
    core.run(listen).unwrap();
}


