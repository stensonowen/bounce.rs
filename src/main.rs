#[macro_use]
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;

use std::str;
use std::io;
//use std::io::{self, ErrorKind, Write};
use std::net::ToSocketAddrs;

use futures::{future, Future, BoxFuture};
use futures::{Stream, Poll, Async};
use futures::{Sink, AsyncSink, StartSend};
use tokio_proto::TcpClient;
use tokio_proto::pipeline::ClientProto;
use tokio_service::Service;
use tokio_core::reactor::Core;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
//use bytes::{BytesMut, BufMut};
use bytes::BytesMut;


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
                Err(_) => Err(io::Error::new(io::ErrorKind::Other,
                                             "invalid UTF-8")),
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
    server: Option<String>,
}

impl<T> PingPong<T> {
    fn new(t: T) -> Self {
        PingPong {
            upstream: t,
            server: None,
        }
    }
}

impl<T> Stream for PingPong<T>
    where T: Stream<Item = String, Error = io::Error>,
          T: Sink<SinkItem = String, SinkError = io::Error>,
{
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<String>, io::Error> {
        // Poll the upstream transport
        match try_ready!(self.upstream.poll()) {
            Some(ref msg) if msg.starts_with("PING ") => { 
                // Intercept pings
                self.server = Some(msg[5..].to_owned());
                // what's this ↓ ?
                // Try flushing the pong, only bubble up errors
                Ok(Async::Ready(try_ready!(self.upstream.poll())))
            }
            m => Ok(Async::Ready(m))
        }
    }
}

impl<T> Sink for PingPong<T>
    where T: Sink<SinkItem = String, SinkError = io::Error>,
{
    type SinkItem = String;
    type SinkError = io::Error;

    fn start_send(&mut self, item: String) -> StartSend<String, io::Error> {
        // Only accept the write if there are no pending pongs
        if self.server.is_some() {
            return Ok(AsyncSink::NotReady(item));
        }
        self.upstream.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        if let Some(s) = self.server.take() {
            let pong = format!("PONG {}", s);
            self.upstream.start_send(pong)?;
        }
        self.upstream.poll_complete()
    }
}



pub struct LineProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for LineProto {
    type Request = String;
    type Response = String;
    type Transport = PingPong<Framed<T, LineCodec>>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(PingPong::new(io.framed(LineCodec)))
    }
}


pub struct Echo;

impl Service for Echo {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        future::ok(req).boxed()
    }
}

fn main() {
    //let addr = "0.0.0.0:12345".parse().unwrap();
    //TcpServer::new(IntProto, addr)
    //    .serve(|| Ok(Doubler));
    let cm = "USER qj 0 * qjkx\r\nNICK qjk\r\nJOIN #test\r\n".to_string();

    //let addr = "irc.freenode.org:6697".to_socket_addrs().unwrap().next().unwrap();
    let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();

    //LineCodec.poll();
    //let lc = LineCodec;
    //lc.AsyncReadframed(LineCodec);

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    //let cli = TcpClient::new(IntProto);
    let cli = TcpClient::new(LineProto);
    let socket = cli.connect(&addr, &handle);
    //let response = socket.and_then(|x| x.call(420));
    let response = socket.and_then(|x| x.call(cm));

    let data = core.run(response);//.unwrap();
    println!("{:?}", data);
    //let (_socket, data) = core.run(response).unwrap();
    //println!("{}", String::from_utf8_lossy(&data));
}

