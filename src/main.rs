extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_io;
extern crate tokio_core;
extern crate bytes;

use std::str;
use std::io::{self, ErrorKind, Write};
use std::net::ToSocketAddrs;

use futures::{future, Future, BoxFuture};
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

//impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for IntProto {
impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for IntProto {
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
    //let addr = "0.0.0.0:12345".parse().unwrap();
    //TcpServer::new(IntProto, addr)
    //    .serve(|| Ok(Doubler));
    let cm = "USER qj 0 * qjkx\r\nNICK qjk\r\nJOIN #test\r\n".as_bytes();

	//let addr = "irc.freenode.org:6697".to_socket_addrs().unwrap().next().unwrap();
	let addr = "0.0.0.0:12345".to_socket_addrs().unwrap().next().unwrap();

	let mut core = Core::new().unwrap();
	let handle = core.handle();
    let cli = TcpClient::new(IntProto);
    let socket = cli.connect(&addr, &handle);
    let response = socket.and_then(|x| x.call(420));

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
