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


use tokio_timer::Timer;
use std::time::{Duration};
use std::thread;

fn main() {
    println!("test");
    let f = futures::future::ok::<bool, tokio_timer::TimeoutError<()>>(false);
    let g = f.and_then(|i| {
        Timer::default()
            .sleep(Duration::from_millis(1000))
            .wait().unwrap();
        futures::future::ok(true)
    });
    let timer = Timer::default();
    let t = timer.sleep(Duration::from_millis(2000)).boxed()
        .map_err(|e| tokio_timer::TimeoutError::Timer((), e))
        .map(|e| false);
    let w = t.select(g).map(|(a,_)| a);
    let x = w.wait();
    let r = x.unwrap_or(false);
    println!("Action completed: {}", r);

}

fn _main() {
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


