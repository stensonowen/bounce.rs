#[macro_use]
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;

use std::{io, str};
use std::net::ToSocketAddrs;

use futures::{Future, Stream, Poll, Async};
use futures::{Sink, AsyncSink, StartSend};
use futures::future::{self, loop_fn, FutureResult, Loop, LoopFn};
use tokio_proto::TcpClient;
use tokio_proto::pipeline::ClientProto;
use tokio_service::Service;
use tokio_core::reactor::Core;
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
        //println!("Sending: {}", msg);
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
        //loop {
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
                //m => {
            //    self.upstream.foo();
            //},
        }
        //}
    }
}

impl<T> Sink for PingPong<T>
    where T: Sink<SinkItem = String, SinkError = io::Error>,
{
    type SinkItem = String;
    type SinkError = io::Error;

    fn start_send(&mut self, item: String) -> StartSend<String, io::Error> {
        // Only accept the write if there are no pending pongs
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
    //let addr = "0.0.0.0:12345".parse().unwrap();
    //TcpServer::new(IntProto, addr)
    //    .serve(|| Ok(Doubler));
    let cm = "USER qj 0 * qjkx\r\nNICK qjk\r\nJOIN #test\r\n".to_string();

    //let addr = "irc.freenode.org:6697".to_socket_addrs().unwrap().next().unwrap();
    let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let tc = TcpClient::new(LineProto);
    let response = tc
        .connect(&addr, &handle)
        .and_then(|client| {
            client.call("ONE\r\n".to_string())
                .and_then(move |response| {
                    client.call("TWO".to_string())
                        .and_then(move |response| {
                            client.call("THREE".to_string())
                        })
                })
        });
    //let socket = tc.connect(&addr, &handle);
    //let response = socket.and_then(|x| x.call(cm));
    let data = core.run(response);//.unwrap();
    println!("{:?}", data);

    
    //let client = tc.and_then(|x| x.call(cm));
    //response.foo();
    //response.and_then(|x| future::ok(x).foo());
                    //client.call("BYE\r\n".to_string())

    //let listen = cli.connect(&addr, &handle).and_then(|r| { });
    //let listen = loop_fn(cli.connect(&addr, &handle), |msg| {
    //let listen = loop_fn(socket.and_then(|s| s.call(cm)), |msg| {
    
    //let listen: LoopFn<Result<_,io::Error>, _> = loop_fn(conn, |cli| {
        //cli.and_then(|x| cli.call());
        //cli.foo();
        //cli.and_then(|x| future::ok(x).foo())
        //let r = cli.and_then(|x| x.call("HI\r\n"));
        //let r = cli.call("HI\r\n");
        //r.foo();
    //});

    //let mut a = core.run(listen).unwrap();
    //a.poll();
        /*
    let listen: LoopFn<Result<_,io::Error>, _> = loop_fn(response, |x| {
        k
        x.and_then(|y| {
            //y.call("BAD\r\n").and_then(
            if y.contains("fuck") {
                //Ok(Loop::Break(y))
                Ok(Loop::Break(x))
            } else {
                Ok(Loop::Continue(x))
            }
        })
        */
    /*
        if true {
            Ok(Loop::Break(msg))
        } else {
            //Ok(Loop::Continue(msg))
            Ok(Loop::Break(msg))
        }
    });
        */

        /*
    let req = cli.connect(&addr, &handle)
        .and_then(|client| {
            client.call(cm)
                .and_then(move |response| {
                    println!("CLIENT: {:?}", response);
                    client.call("BYE\r\n".to_string())
                })
            .and_then(|response| {
                println!("CLIENT_: {:?}", response);
                Ok(())
            })
        });
        */
        
    //let response = socket.and_then(|x| x.call(cm));

    //let data = core.run(response);//.unwrap();
    //println!("{:?}", data);
    //let (_socket, data) = core.run(response).unwrap();
    //println!("{}", String::from_utf8_lossy(&data));
}
