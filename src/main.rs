#[macro_use]
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_timer;
extern crate bytes;
extern crate time;

use std::{io, str};
use std::net::ToSocketAddrs;

use futures::{stream, Future, Stream, Sink};
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use tokio_io::AsyncRead;

pub mod codec;
use codec::{LineCodec, PingPong};
use codec::line::Line;


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
    core.run(listen).unwrap();
}


