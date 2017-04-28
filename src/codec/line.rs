use std::string::ToString;
use std::path::PathBuf;
use std::io::{self, BufWriter};
use std::fs::File;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Line {
    // PMs are logged sometimes trigger alerts (and are sent to client(s))
    PrivMsg { src: String, dst: String, text: String, orig: String },
    // Metadata is not logged (but is sent to client(s))
    Meta { orig: String },
    // Pings must be ponged but are neither logged nor sent to client(s)
    Ping { orig: String },
}

impl Line {
    pub fn new_meta(s: &str) -> Self {
        Line::Meta{ orig: s.to_string() }
    }
    pub fn new_ping(s: &str) -> Self {
        Line::Ping{ orig: s.to_string() }
    }
    pub fn new_pm(src: &str, dst: &str, text: &str, orig: &str) -> Self {
        Line::PrivMsg{
            src: src.to_string(),
            dst: dst.to_string(),
            text: text.to_string(),
            orig: orig.to_string(),
        }
    }
    pub fn pong_from_ping(p: &str) -> Line {
        let s = p.replacen("PING ", "PONG ", 1);
        Line::Ping { orig: s }
    }
    pub fn from_str(input: &str) -> Self {
        // TODO: adhere closer to the RFC
        // e.g. `:Angel!wings@irc.org PRIVMSG Wiz message goes here`
        // TODO: treat PRIVMSG and NOTICE differently?
        // TODO: handle '\r' better?
        let in_fixed = input.trim_right();
        let mut parts = in_fixed.splitn(4, ' ');
        let a = parts.nth(0);
        let b = parts.nth(0);
        let c = parts.nth(0);
        let d = parts.nth(0);
        match (a, b, c, d) {
            (Some(s), Some("PRIVMSG"), Some(d), Some(m)) | 
                (Some(s), Some("NOTICE"), Some(d), Some(m)) => 
            {
                let i = if s.starts_with(':') { 1 } else { 0 };
                let j = s.find('!').unwrap_or(s.len()-1);
                let src_fixed = &s[i..j];
                let msg_fixed = if m.starts_with(':') { &m[1..] } else { m };
                Line::new_pm(src_fixed, d, msg_fixed, in_fixed)
            },
            (Some("PING"), _, _, _) => Line::new_ping(in_fixed),
            _ => Line::new_meta(input)
        }
    }
}

impl ToString for Line {
    fn to_string(&self) -> String {
        match *self {
            Line::PrivMsg { orig: ref o, .. } => o,
            Line::Meta { orig: ref o, .. } => o,
            Line::Ping { orig: ref o, .. } => o,
        }.clone()
    }
}

// Maintain on-disk logs *in addition to* in-memory buffer
pub enum FileState {
    Open(BufWriter<File>),
    Closed(PathBuf),
}

pub struct Logs(HashMap<String,FileState>);

impl Logs {
    pub fn new() -> Self {
        Logs(HashMap::new())
    }
    pub fn update(line: &Line) -> Result<usize,io::Error> {
        Ok(0)
    }
}

