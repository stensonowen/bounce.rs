use std::str;
use std::io::{self, ErrorKind, Write};

use tokio_io::codec::{Encoder, Decoder};
use bytes::{BytesMut, BufMut};

/*
#[derive(Debug)]
pub enum Command {
    Num(u16),
    Str(String),
}

#[derive(Debug)]
pub struct Line {
    // https://tools.ietf.org/html/rfc2812#section-2.3
    prefix: Option<String>,
    command: Command,
    params: Vec<String>,
}

// TODO: use error chain or something instead?
fn io_err(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

#[derive(Debug)]
pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = Line;
    type Error = io::Error;

    // handle case of sending text without newlines?
    // discard everything more than 512 bytes before a newline?

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Line>, io::Error> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            // verify byte before is a b'\r'?
            let line = buf.split_to(i+1);
            let slice = if line.ends_with(&[b'\r', b'\n']) {
                &line[..i-1]
            } else {
                &line[..i]
            };
            // parse `slice` into a Line:
            let prefix = if slice.starts_with(&[b':']) {
                let delim = buf.iter().position(|&b| b == b' ')
                    .ok_or(io_err("bad line: unfinished prefix"))?;
                // start at 1? include the colon?
                let p = &slice[..delim];
                Some(str::from_utf8(p).map_err(|_| io_err("bad line: utf8"))?)
            } else {
                None
            };
            unimplemented!()

        } else {
            Ok(None)
        }
    }
}

impl Encoder for LineCodec {
    type Item = Line;
    type Error = io::Error;

    fn encode(&mut self, item: Line, into: &mut BytesMut) -> io::Result<()> {
        panic!();
        Ok(())
    }
}
*/



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
