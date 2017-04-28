#[macro_use]
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;

use std::{io, str};
use std::net::ToSocketAddrs;

use futures::{stream, Future, Poll, StartSend};
use futures::{Async, Stream, Sink, AsyncSink};
//use futures::future::{loop_fn, Loop};
//use tokio_proto::TcpClient;
//use tokio_proto::pipeline::{ClientProto, ClientService};
use tokio_proto::pipeline::ClientProto;
//use tokio_service::Service;
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
//use bytes::{BufMut, BytesMut};
use bytes::BytesMut;


#[derive(Default)]
pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<String>> {
        //println!("Received: {:?}", std::str::from_utf8(buf));
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

                let poll = try_ready!(self.upstream.poll());
                // does this actually work? never tested it
                println!("NOTE: {:?}", poll);
                Ok(Async::Ready(poll))
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
    let conn_msg: Vec<Result<String,io::Error>> = vec![
        Ok("USER a b c d".to_string()),
        Ok("NICK qjk".to_string()),
        Ok("JOIN #test".to_string()),
    ];

    //let addr = "irc.freenode.org:6667".to_socket_addrs().unwrap().next().unwrap();
    let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    //let p = PingPong::new(LineCodec);

    let work = TcpStream::connect(&addr, &handle)
        .and_then(|socket| {
            // is this redundant?
            let (sink, stream) = socket.framed(LineCodec).split();
            //let (sink, stream) = socket.framed(LineProto).split();
            //let (sink, stream) = socket.framed(PingPong<LineCodec>).split();
            sink.send_all(stream::iter(conn_msg)).and_then(|_| {
                // better way to do this?
                stream.for_each(|i| {
                    println!("SAW: `{}`", i.replace("\r\n", ""));
                    futures::future::ok(())
                })
            })
        });
    // will be empty vec
    let data = core.run(work).unwrap();
    println!("DATA: {:?}", data);

/*
	let mut core = Core::new().unwrap();
    let handle = core.handle();
    let remote_addr = "127.0.0.1:14566".parse().unwrap();

    let work = TcpStream::connect(&remote_addr, &handle)
        .and_then(|socket| {
            // Once the socket has been established, use the `framed` helper to
            // create a transport.
            let transport = socket.framed(LineCodec);

            // We're just going to send a few "log" messages to the remote
            let lines_to_send: Vec<Result<String, io::Error>> = vec![
                Ok("Hello world".to_string()),
                Ok("This is another message".to_string()),
                Ok("Not much else to say".to_string()),
            ];

            // Send all the messages to the remote. The strings will be encoded by
            // the `Codec`. `send_all` returns a future that completes once
            // everything has been sent.
            transport.send_all(stream::iter(lines_to_send))
        });
    core.run(work).unwrap();
*/




    //let tc = TcpClient::new(LineProto);

    /*
    let tc = TcpClient::new(LineProto);
    let response = tc.connect(&addr, &handle)
        .and_then(|client| client.call("USER a b c d".to_string())
            .and_then(move |r1| {
                println!("GOT1: {}", r1);
                client.call("NICK qjkxk".to_string())}
                .and_then(move |r2| {
                    println!("GOT2: {}", r2);
                    client.call("JOIN #test".to_string())}
                    .and_then(move |r3| {
                        println!("GOT3: {}", r3);
                        client.call("PRIVMSG #test foo".to_string())}
                              )
                          )
                      )
            );
            */

    /*
    let response = tc.connect(&addr, &handle)
        .and_then(|client| client.call(cm)
            //.and_then(|_r0| loop_fn(client, |c| c.call("ACK".to_string())
            .and_then(|_r0| loop_fn(client, |c| futures::future::ok(()) // :/
            //.and_then(|_r0| loop_fn(client, |c| futures::future::ok(c)
                        .and_then(|_ri| {
                            //x.fetch_add(1, Ordering::SeqCst);
                            //println!("`{}`", _ri);
                            let lcs: Loop<ClientService<TcpStream,LineProto>,_> 
                                = Loop::Continue(c);
                            Ok(lcs)
                        })
                    ))
                );
                      */
    /*
    let response = tc.connect(&addr, &handle)
        .and_then(|client| client.foo().call("USER a b c d".to_string())
            .and_then(|rx| client.call("NICK qjkxk".to_string())
                .and_then(|ry| client.call("JOIN #test".to_string())
            //.and_then(|_r0| loop_fn(client, |c| c.call("ACK".to_string())
                    .and_then(|_r0| loop_fn(client, |c| c.call(String::new())
            //.and_then(|_r0| loop_fn(client, |c| futures::future::ok(c)
                        .and_then(|_ri| {
                            x.fetch_add(1, Ordering::SeqCst);
                            //println!("`{}`", _ri);
                            let lcs: Loop<ClientService<TcpStream,LineProto>,_> 
                                = Loop::Continue(c);
                            Ok(lcs)
                        })
                    ))
                )
            )
        );
        */
    
    //core.run(response).unwrap();
}

