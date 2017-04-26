#[macro_use]
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;

use std::{io, str};
use std::net::ToSocketAddrs;

use futures::{Future, Poll, StartSend};
use futures::{Async, Stream, Sink, AsyncSink};
use futures::future::{loop_fn, Loop};
use tokio_proto::TcpClient;
use tokio_proto::pipeline::{ClientProto, ClientService};
use tokio_service::Service;
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use bytes::BytesMut;


#[derive(Default)]
pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<String>> {
        println!("Received: {:?}", std::str::from_utf8(buf));
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
    response: Option<String>,
    // TODO: stuff like AWAY ?
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
          T: Sink<SinkItem = String, SinkError = io::Error>,
{
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<String>, io::Error> {
        // Poll the upstream transport
        match try_ready!(self.upstream.poll()) {
            Some(ref msg) if msg.starts_with("PING ") => { 
                // Intercept pings
                let resp = msg.replacen("PING", "PONG", 1);
                self.response = Some(resp);
                self.poll_complete()?;
                let _poll = try_ready!(self.upstream.poll());
                // Never hit:
                println!("UHHHH `{:?}`", _poll);
                unimplemented!();
                //Ok(Async::Ready(_poll))
            },
            // Final output:
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
        // Only accept the write if there are no pending pong
        if self.response.is_some() {
            return Ok(AsyncSink::NotReady(item));
        }
        self.upstream.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        if let Some(pong) = self.response.take() {
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


fn main() {
    let cm = "USER qj 0 * qjkx\r\nNICK qjk\r\nJOIN #test\r\n".to_string();

    //let addr = "irc.freenode.org:6697".to_socket_addrs().unwrap().next().unwrap();
    let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let tc = TcpClient::new(LineProto);
    let response = tc.connect(&addr, &handle)
        .and_then(|client| client.call(cm)
            .and_then(move |_r0| loop_fn(client, |c| c.call("ACK".to_string())
                .and_then(|_ri| {
                    let lcs: Loop<ClientService<TcpStream,LineProto>,_> 
                        = Loop::Continue(c);
                    Ok(lcs)
                })
            ))
        );
    
    core.run(response).unwrap();
}

