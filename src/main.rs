#[macro_use]
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_timer;
extern crate bytes;
extern crate time;

use std::{io, str};
use std::net::ToSocketAddrs;

use futures::{stream, Future, Stream, Sink, IntoFuture};
use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;
use tokio_io::AsyncRead;

pub mod codec;
use codec::{LineCodec, PingPong};
use codec::line::Line;

use tokio_timer::Timer;
use std::time::{Duration};
use std::thread;
use futures_cpupool::CpuPool;

//const FILE_TIMEOUT_MS: u64 = 1_000;


use std::io::BufWriter;
use std::fs::File;
use std::io::Write;
use futures_cpupool::CpuFuture;

struct FileClosed;

//fn close_handle(mut f: File) {
fn close_handle() -> FileClosed {
    let mut f = File::create("test.txt").unwrap();
    thread::sleep(Duration::from_millis(1000));
    f.write(b"Hello world\n");
    f.flush().unwrap();
    FileClosed
}

fn main() {
    //close_handle();
    let pool = CpuPool::new(2);
    let timer = Timer::default();
        //let timeout = timer.sleep(Duration::from_millis(750)).then(|_| Err(()));
        //let func = pool.spawn_fn(|| Ok(close_handle()));
        //let winner = timeout.select(func).map(|(win, _)| win);
        //func.foo();
    //let func: CpuFuture<_,()> = pool.spawn_fn(|| Ok(close_handle()));
    //let timeout = timer.timeout(func, Duration::from_millis(900));
        //timeout.foo();
        //timeout.wait();
    let a = timer.sleep(Duration::from_millis(500))
        .then(|x| x.map(|_| close_handle())).boxed();

    //a.foo();
        //.and_then(|_| futures::future::ok(()));

    //thread::sleep(Duration::from_millis(1200));

    //let action = futures::future::ok::<_, tokio_timer::TimeoutError<_>>(false)
    //    .and_then(|i| {
    //        Timer::default()
    //            .sleep(Duration::from_millis(1000))
    //            .wait().unwrap();
    //        futures::future::ok(true)
    //});
        //let timer = Timer::default();
    //let f = timer.timeout(action, Duration::from_millis(2000));
    //f.foo();
    //thread::sleep(Duration::from_millis(5_000));
}

fn main2() {
    println!("test");
    let f = futures::future::ok::<bool, tokio_timer::TimeoutError<()>>(false);
    let g = f.and_then(|i| {
        Timer::default()
            .sleep(Duration::from_millis(1000))
            .wait().unwrap();
        futures::future::ok(true)
    });
    let timer = Timer::default();
    let t = timer.sleep(Duration::from_millis(2000)).into_future()
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


