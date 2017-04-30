#[macro_use]
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_timer;
extern crate bytes;

extern crate time;
extern crate rpassword;

#[macro_use]
extern crate serde_derive;
extern crate toml;


use std::{io, str};
use std::net::ToSocketAddrs;

use futures::{future, stream, Future, Stream, Sink};
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpStream;
use tokio_io::AsyncRead;

pub mod log;
pub mod codec;
pub mod config;
use codec::{LineCodec, PingPong};
use codec::line::Line;
use log::Logs;
use config::{Config, Server};

fn server(srv_name: String, srv: Server, handle: Handle) {
    // TODO: make sure someone's nick can't contains directory traversal
    // `NICK ../../../../dev/sda1`
    let mut logs = Logs::new("/tmp/irc_logs");
    let conn_msg: Vec<Result<Line, io::Error>> = srv.conn_msg()
        .iter().map(|s| Ok(Line::from_str(s))).collect();
    println!("{:?}", conn_msg);
    let addr = srv.get_addr().to_socket_addrs().unwrap().next().unwrap();
    let stream = TcpStream::connect(&addr, &handle);
    let listen = stream.and_then(move |socket| {
        let transport = PingPong::new(socket.framed(LineCodec));
        let (sink, stream) = transport.split();
        sink.send_all(stream::iter(conn_msg))
            .and_then(move |_| {
                stream.for_each(move |line| {
                    println!("SAW: `{:?}`", line);
                    if let Some((name,text)) = line.format_privmsg(&srv_name) {
                        logs.write(name,&text).unwrap();
                    }
                    future::ok(())
                })
            })
    }).map_err(|_| ());
    handle.spawn(listen);
}

fn main() {
    let config_file = "config2.toml";
    let config = Config::from(config_file).unwrap();

    let mut core = Core::new().unwrap();

    for (name,srv) in config.servers {
        server(name, srv, core.handle());
    }

    let empty: future::Empty<(),()> = future::empty();
    core.run(empty).unwrap();

    /*
    let conn_msg: Vec<Result<Line, io::Error>> = vec![
        Ok(Line::from_str("USER a b c d")),
        Ok(Line::from_str("NICK qjkxk")),
        Ok(Line::from_str("JOIN #test")),
    ];
    let mut logs = Logs::new("/tmp/irc_logs");

    let mut core = Core::new().unwrap();
    //let addr = "irc.freenode.org:6667".to_socket_addrs().unwrap().next().unwrap();
    let addr = "irc.mozilla.org:6667".to_socket_addrs().unwrap().next().unwrap();
    //let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();


    let stream = TcpStream::connect(&addr, &core.handle());
    let listen = stream.and_then(|socket| {
        let transport = PingPong::new(socket.framed(LineCodec));
        let (sink, stream) = transport.split();
        sink.send_all(stream::iter(conn_msg))
            .and_then(|_| {
                stream.for_each(|line| {
                    println!("SAW: `{:?}`", line);
                    if let Some((name,text)) = line.format_privmsg("mozilla") {
                        logs.write(name,&text).unwrap();
                    }
                    futures::future::ok(())
                })
            })
    });
    core.run(listen).unwrap();
    */
}


