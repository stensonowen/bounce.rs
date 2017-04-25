#[macro_use]
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;

use std::str;
use std::io::{self, ErrorKind, Write};
use std::net::ToSocketAddrs;

use futures::{future, Future, BoxFuture};
use futures::{Stream, Poll, Async};
use futures::{Sink, AsyncSink, StartSend};
//use tokio_proto::TcpServer;
use tokio_proto::TcpClient;
//use tokio_proto::pipeline::ServerProto;
use tokio_proto::pipeline::ClientProto;
use tokio_service::Service;
use tokio_core::reactor::Core;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
use bytes::{BytesMut, BufMut};


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
        loop {
            // Poll the upstream transport. `try_ready!` will bubble up errors
            // and Async::NotReady.
            match try_ready!(self.upstream.poll()) {
                Some(ref msg) if msg.starts_with("PING ") => { 
                    // Intercept [ping] messages
                    self.server = Some(msg[4..].to_owned());
                    // Try flushing the pong, only bubble up errors
                    try!(self.poll_complete());
                }
                m => return Ok(Async::Ready(m)),
            }
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
        //if self.pongs_remaining > 0 {
        if self.server.is_some() {
            return Ok(AsyncSink::NotReady(item));
        }

        // If there are no pending pongs, then send the item upstream
        self.upstream.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        //while self.pongs_remaining > 0 {
		//while let Some(s) = self.server {
		while self.server.is_some() {
			let s = self.server.take();
            // Try to send the pong upstream
            let res = try!(self.upstream.start_send("[pong]".to_string()));

            if !res.is_ready() {
                // The upstream is not ready to accept new items
                break;
            }
        }

        // Call poll_complete on the upstream
        //
        // If there are remaining pongs to send, this call may create additional
        // capacity. One option could be to attempt to send the pongs again.
        // However, if a `start_send` returned NotReady, and this poll_complete
        // *did* create additional capacity in the upstream, then *our*
        // `poll_complete` will get called again shortly.
        //
        // Hopefully this makes sense... it probably doesn't, so please ask
        // questions in the Gitter channel and help me explain this better :)
        self.upstream.poll_complete()
    }
}



pub struct LineProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for LineProto {
    /// For this protocol style, `Request` matches the `Item` type of the codec's `Encoder`
    type Request = String;

    /// For this protocol style, `Response` matches the `Item` type of the codec's `Decoder`
    type Response = String;

    /// A bit of boilerplate to hook in the codec:
    type Transport = PingPong<Framed<T, LineCodec>>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(PingPong::new(io.framed(LineCodec)))
    }
}


pub struct Echo;

impl Service for Echo {
    // These types must match the corresponding protocol types:
    type Request = String;
    type Response = String;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        // In this case, the response is immediate.
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

/*
extern crate futures;
extern crate native_tls;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_tls;

use std::io;
use std::net::ToSocketAddrs;

use futures::Future;
use native_tls::TlsConnector;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_tls::TlsConnectorExt;


fn main() {
    let cm = "USER qj 0 * qjkx\r\nNICK qjk\r\nJOIN #test\r\n".as_bytes();

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let addr = "irc.freenode.org:6697".to_socket_addrs().unwrap().next().unwrap();

    let cx = TlsConnector::builder().unwrap().build().unwrap();
    let socket = TcpStream::connect(&addr, &handle);

    let tls_handshake = socket.and_then(|socket| {
        println!("Shaking");
        let tls = cx.connect_async("irc.freenode.org", socket);
        tls.map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e)
        })
    });
    let request = tls_handshake.and_then(|socket| {
        println!("Writing connect message");
        tokio_io::io::write_all(socket, cm)
    });
    let response = request.and_then(|(socket, _request)| {
        tokio_io::io::read_to_end(socket, vec![])
    });

    let (_socket, data) = core.run(response).unwrap();
    println!("{}", String::from_utf8_lossy(&data));
}

*/


/*

extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_io;
extern crate bytes;

use std::str;
use std::io::{self, ErrorKind, Write};

use futures::{future, Future, BoxFuture};
use tokio_proto::TcpServer;
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
use bytes::{BytesMut, BufMut};

// First, we implement a *codec*, which provides a way of encoding and
// decoding messages for the protocol. See the documentation for `Codec` in
// `tokio-core` for more details on how that works.

#[derive(Default)]
pub struct IntCodec;

fn parse_u64(from: &[u8]) -> Result<u64, io::Error> {
    Ok(str::from_utf8(from)
       .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?
       .parse()
       .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?)
}

impl Decoder for IntCodec {
    type Item = u64;
    type Error = io::Error;

    // Attempt to decode a message from the given buffer if a complete
    // message is available; returns `Ok(None)` if the buffer does not yet
    // hold a complete message.
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<u64>, io::Error> {
        if let Some(mut i) = buf.iter().position(|&b| b == b'\n') {
            // remove the line, including the '\n', from the buffer
            let full_line = buf.split_to(i + 1);
            // strip the `\n' (and `\r' if present)
            if full_line.ends_with(&[b'\r', b'\n']) {
                i -= 1;
            }
            let slice = &full_line[..i];
            Ok(Some(parse_u64(slice)?))
        } else {
            Ok(None)
        }
    }

    // Attempt to decode a message assuming that the given buffer contains
    // *all* remaining input data.
    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<u64>, io::Error> {
        let amt = buf.len();
        Ok(Some(parse_u64(&buf.split_to(amt)[..])?))
    }
}

impl Encoder for IntCodec {
    type Item = u64;
    type Error = io::Error;

    fn encode(&mut self, item: u64, into: &mut BytesMut) -> io::Result<()> {
        writeln!(into.writer(), "{}", item)?;
        Ok(())
    }
}

// Next, we implement the server protocol, which just hooks up the codec above.

pub struct IntProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for IntProto {
    type Request = u64;
    type Response = u64;
    type Transport = Framed<T, IntCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(IntCodec))
    }
}

// Now we implement a service we'd like to run on top of this protocol

pub struct Doubler;

impl Service for Doubler {
    type Request = u64;
    type Response = u64;
    type Error = io::Error;
    type Future = BoxFuture<u64, io::Error>;

    fn call(&self, req: u64) -> Self::Future {
        // Just return the request, doubled
        future::finished(req * 2).boxed()
    }
}

// Finally, we can actually host this service locally!
fn main() {
    let addr = "0.0.0.0:12345".parse().unwrap();
    TcpServer::new(IntProto, addr)
        .serve(|| Ok(Doubler));
}
*/
