
#[macro_use]
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate bytes;

extern crate time;
extern crate rpassword;

#[macro_use]
extern crate serde_derive;
extern crate toml;

#[macro_use]
extern crate log;

use std::{io, str};
use std::net::ToSocketAddrs;

use futures::{future, stream, Future, Stream, Sink};
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpStream;
use tokio_io::AsyncRead;

pub mod logs;
pub mod codec;
pub mod config;
use codec::{LineCodec, PingPong};
use codec::line::Line;
use logs::Logs;
use config::{Config, Server};

fn _server(srv_name: String, srv: Server, log_path: &str, handle: Handle) {
    // TODO: make sure someone's nick can't contains directory traversal
    // `NICK ../../../../dev/sda1`
    // TODO: stop passwords from leaking into log files (don't long conn msg)
    let mut logs = Logs::new(log_path);
    let conn_msg: Vec<_> = srv.conn_msg();
    //info!(srv.logger, "Initiating connection: {:?}", conn_msg);
    let conn_lines: Vec<Result<Line, io::Error>> = conn_msg
        .iter().map(|s| Ok(Line::from_str(s))).collect();
    let addr = srv.get_addr().to_socket_addrs().unwrap().next().unwrap();
    //info!(srv.logger, "Connecting to {} w/ tls={}", addr, srv.tls);

    let stream = TcpStream::connect(&addr, &handle);
    let listen = stream.and_then(move |socket| {
        let transport = PingPong::new(socket.framed(LineCodec));
        let (sink, stream) = transport.split();
        sink.send_all(stream::iter(conn_lines))
            .and_then(move |_| {
                stream.for_each(move |line| {
                    //info!(srv.logger, "{:?}", line); // if super verbose
                    //info!(srv.logger, "{}", line.to_string());
                    if let Some((name,text)) = line.format_privmsg(&srv_name) {
                        logs.write(name,&text).unwrap();
                    }
                    future::ok(())
                })
            })
    }).map_err(|_| ());
    handle.spawn(listen);
}

fn _main() {
    // TODO: clap/docopt CLI args for logging verbosity|output / config file
    let config_file = "config2.toml";

    /*
    let dec = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(dec).build().fuse();
    let async_drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(async_drain, o!("cfg" => config_file));

    let config = Config::from(config_file, &log).unwrap();

    let mut core = Core::new().unwrap();
    for (name,srv) in config.servers {
        _server(name, srv, &config.logs_dir, core.handle());
    }
    let empty: future::Empty<(),()> = future::empty();
    core.run(empty).unwrap();
    */

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


use std::net::SocketAddr;
use tokio_core::net::TcpListener;

fn listener(addr: SocketAddr, handle: &Handle) -> io::Result<()> {
    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    let (tx, rx) = futures::sync::mpsc::unbounded();
    tx.send(String::from("foo"));

    let data = socket.incoming().for_each(|(stream,addr)| {
        let (reader, writer) = stream.split();
        futures::future::ok(())
    });


    Ok(())
}

fn main() {

    let conn_msg: Vec<Result<Line, io::Error>> = vec![
        Ok(Line::from_str("USER a b c d")),
        Ok(Line::from_str("NICK qjkxk")),
        Ok(Line::from_str("JOIN #test")),
    ];
    let mut logs = Logs::new("/tmp/irc_logs");

    let mut core = Core::new().unwrap();
    //let addr = "irc.freenode.org:6667".to_socket_addrs().unwrap().next().unwrap();
    //let addr = "irc.mozilla.org:6665".to_socket_addrs().unwrap().next().unwrap();
    let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();


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
}

