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

#[derive(Debug)]
pub enum Line {
    // PMs are logged sometimes trigger alerts (and are sent to client(s))
    PrivMsg { src: String, dst: String, text: String, orig: String },
    // Metadata is not logged (but is sent to client(s))
    Meta { orig: String },
    // Pings must be ponged but are neither logged nor sent to client(s)
    Ping { orig: String },
}

impl Line {
    fn new_meta(s: &str) -> Self {
        Line::Meta{ orig: s.to_string() }
    }
    fn new_ping(s: &str) -> Self {
        Line::Ping{ orig: s.to_string() }
    }
    fn new_pm(src: &str, dst: &str, text: &str, orig: &str) -> Self {
        Line::PrivMsg{
            src: src.to_string(),
            dst: dst.to_string(),
            text: text.to_string(),
            orig: orig.to_string(),
        }
    }
    fn pong_from_ping(p: &str) -> Line {
        let s = p.replacen("PING ", "PONG ", 1);
        Line::Ping { orig: s }
    }
    fn from_str(input: &str) -> Self {
        // TODO: adhere closer to the RFC
        // e.g. `:Angel!wings@irc.org PRIVMSG Wiz message goes here`
        // TODO: treat PRIVMSG and NOTICE differently?
        // TODO: handle '\r' better?
        let in_fixed = input.trim_right();
        let mut parts = in_fixed.splitn(4, ' ');
        let a = parts.nth(0);
        let b = parts.nth(0);
        let c = parts.nth(0);
        let d = parts.nth(0);
        match (a, b, c, d) {
            (Some(s), Some("PRIVMSG"), Some(d), Some(m)) | 
                (Some(s), Some("NOTICE"), Some(d), Some(m)) => 
            {
                let i = if s.starts_with(':') { 1 } else { 0 };
                let j = s.find('!').unwrap_or(s.len()-1);
                let src_fixed = &s[i..j];
                let msg_fixed = if m.starts_with(':') { &m[1..] } else { m };
                Line::new_pm(src_fixed, d, msg_fixed, in_fixed)
            },
            (Some("PING"), _, _, _) => Line::new_ping(in_fixed),
            _ => Line::new_meta(input)
        }
    }
}

impl std::string::ToString for Line {
    fn to_string(&self) -> String {
        match *self {
            Line::PrivMsg { orig: ref o, .. } => o,
            Line::Meta { orig: ref o, .. } => o,
            Line::Ping { orig: ref o, .. } => o,
        }.clone()
    }
}


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
    fn new(t: T) -> Self {
        PingPong {
            upstream: t,
            response: None,
        }
    }
}

impl<T> Stream for PingPong<T>
    where T: Stream<Item = Line, Error = io::Error>,
          T: Sink<SinkItem = Line, SinkError = io::Error>
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
    where T: Sink<SinkItem = Line, SinkError = io::Error>
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


fn main() {
    let conn_msg: Vec<Result<Line, io::Error>> = vec![
        Ok(Line::from_str("USER a b c d")),
        Ok(Line::from_str("NICK qjkxk")),
        Ok(Line::from_str("JOIN #test")),
    ];

    let addr = "irc.freenode.org:6667".to_socket_addrs().unwrap().next().unwrap();
    //let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();

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


