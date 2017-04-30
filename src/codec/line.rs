use std::string::ToString;
use std::borrow::Cow;
use time;

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
    pub fn format_privmsg(&self, srv_name: &str) -> Option<(Cow<str>,String)> {
        // (name,msg)
        // if this message was in a public channel, `name` should be that channel
        // if it was a private message from another user, it should be their nick
        if let &Line::PrivMsg{ ref src, ref dst, ref text, .. } = self {
            let now = time::now();
            let msg = format!("{} {:>9}:  {}\n", now.rfc3339(), src, text);
            // https://tools.ietf.org/html/rfc2812#section-2.3.1
            let valid_nick_start = |c: char| 
                c >= char::from(0x41) && c <= char::from(0x7d);
            let name: Cow<str> = if dst.starts_with(valid_nick_start) { 
                Cow::Owned(format!("{}_{}", src, srv_name))
            } else { 
                Cow::Borrowed(dst)
            };
            Some((name,msg))
        } else {
            None
        }
    }
    pub fn mention(&self, nick: &str) -> bool {
        if let &Line::PrivMsg{ ref dst, ref text, .. } = self {
            dst == nick || text.contains(nick)
        } else {
            false
        }
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
            //(Some(s), Some("NOTICE"), Some(d), Some(m)) |
            (Some(s), Some("PRIVMSG"), Some(d), Some(m)) =>
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


