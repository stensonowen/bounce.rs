/*
extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::str;
use bytes::{BytesMut};
use tokio_io::codec::{Encoder, Decoder};

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
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "bad utf8"))
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

use tokio_proto::pipeline::ServerProto;
pub struct LineProto;

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
    // For this protocol style, `Request` matches the codec `In` type
    type Request = String;

    // For this protocol style, `Response` matches the coded `Out` type
    type Response = String;

    // A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

use tokio_service::Service;
pub struct Echo;
use futures::{future, Future, BoxFuture};

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

use tokio_proto::TcpServer;

fn main() {
    // Specify the localhost address
    let addr = "0.0.0.0:12345".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(LineProto, addr);

    // We provide a way to *instantiate* the service for each new
    // connection; here, we just immediately return a new instance.
    server.serve(|| Ok(Echo));
}
*/


extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_io;
extern crate bytes;

use std::io;

use futures::{future, Future, BoxFuture};
use tokio_proto::TcpServer;
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;
use tokio_io::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};

// First, we implement a *codec*, which provides a way of encoding and
// decoding messages for the protocol. See the documentation for `Codec` in
// `tokio-core` for more details on how that works.

//mod irc; 

mod responses;
mod codec;
use codec::*;

pub enum Response {
    Command(responses::CmdRsp),
    Error(  responses::ErrRsp),
    Misc(   responses::MscRsp),
}

/*
#[derive(Debug)]
pub enum Command {
    Tmp,
}

#[derive(Debug)]
pub struct Line {
    // https://tools.ietf.org/html/rfc2812#section-2.3
    prefix: Option<String>,
    command: Command,
    params: Vec<String>,
}

#[derive(Debug)]
pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = Line;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        Ok(Some(Line {
            prefix: None,
            command: Command::Tmp,
            params: vec![],
        }))
    }
}



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
*/

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
