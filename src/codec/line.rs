use std::string::ToString;
use std::path::PathBuf;
use std::io::{self, BufWriter};
use std::fs::File;
use std::collections::HashMap;

use futures_cpupool::CpuPool;

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


use tokio_timer::{Timer, Timeout, TimerError};
use futures_cpupool::CpuFuture;
use futures::Future;
use std::io::Write;
use std::thread;
use std::time::Duration;
use std::marker::Send;
pub struct FileClosed;
const FILE_TIMEOUT_MS: u64 = 1_000;

// Maintain on-disk logs *in addition to* in-memory buffer
// TODO: don't use a box
pub enum FileState {
    Open {
        writer: BufWriter<File>,
        //timeout: Box<Future<Item=io::Result<FileState>, Error=TimerError> + Send>,
        timeout: Box<Future<Item=FileState, Error=TimerError> + Send>,
        // can we chain onto this afterwards? p sure
        //timer: Option<Timeout<CpuFuture<FileClosed, ()>>>,
        path: PathBuf,
    },
    Closed(PathBuf),
}

impl FileState {
    fn new(name: &str) -> Self {
        FileState::Closed(PathBuf::from(name))
    }
    fn is_open(&self) -> bool {
        if let &FileState::Open{..} = self { true } else { false }
    }
    fn clone_path(&self) -> PathBuf {
        match self {
            &FileState::Open { path: ref p, .. } => p,
            &FileState::Closed(ref p) => p,
        }.clone()
    }
    fn open(self) -> io::Result<FileState> {
        if self.is_open() {
            Ok(self)
        } else {
            let pb = self.clone_path();
            let mut f = File::open(&pb)?;
            let bw = BufWriter::new(f);
            let timer = Timer::default();
            let close_event = timer.sleep(Duration::from_millis(FILE_TIMEOUT_MS))
                .then(|t| t.map(|_| self.close().unwrap()));
            Ok(FileState::Open {
                writer:  bw,
                timeout: close_event.boxed(),
                path:    pb,
            })
        }
    }
    fn close(self) -> io::Result<FileState> {
        if let FileState::Open{ writer: mut w, path: pb, .. } = self {
            w.flush()?;
            Ok(FileState::Closed(pb))
        } else {
            Ok(self)
        }
    }
}

pub struct Logs(HashMap<String,FileState>);
use futures;

impl Logs {
    pub fn new() -> Self {
        Logs(HashMap::new())
    }
    pub fn add(&mut self, name: &str) {
        // TODO: not clobber?
        let val = FileState::new(name);
        self.0.insert(name.to_string(), val);
    }
    pub fn open(&mut self, name: String) {
        let old_val = self.0.remove(&name).unwrap_or(FileState::new(&name));
        let new_val = if old_val.is_open() {
            let tmp = old_val.open().unwrap();
            if let FileState::Open{ timeout: ref t, .. } = tmp {
                t.and_then(|x| futures::future::ok(()));

            }
            tmp
        } else {
            old_val
            //self.0.insert(name.to_string(), old_val);
        };
        self.0.insert(name, new_val);
        //let entry = self.0.entry(name.to_string())
        //    .or_insert(FileState::new(name));
        //if entry.is_open() == false {
        //    let val = entry.open();
        //}
    }

}
/*
pub struct Logs {
    files: HashMap<String,FileState>,
    pool: CpuPool,
//(HashMap<String,FileState>);
}

impl Logs {
    pub fn new() -> Self {
        Logs {
            files: HashMap::new(),
            pool: CpuPool::new(2),
        }
    }

}
*/
/*
impl Logs {
    pub fn new() -> Self {
        Logs {
            files: HashMap::new(),
            pool: CpuPool::new(2),
        }
    }
    pub fn add(&mut self, name: &str) {
        if self.files.contains_key(name) == false {
            let val = FileState::Closed(PathBuf::from(name));
            self.files.insert(name.to_string(), val);
        } else {
            panic!("Value duplicated");
        }
    }
    pub fn open(&mut self, name: &str) -> io::Result<&BufWriter<File>> {
        let mut file = File::open(name)?;
        let val = FileState::Open(BufWriter::new(file));
        self.files.insert(name.to_owned(), val);
        match self.files.get(name) {
            Some(&FileState::Open(ref br)) => Ok(br),
            _ => unreachable!(),
        }
    }
    pub fn write_and_close(&mut self, name: &str) -> io::Result<()> {
        let g = self.files.get(name);
        let w = match g {
            Some(&FileState::Open(ref b)) => b,
            Some(&FileState::Closed(_)) => {
                self.open(name)?
            },
            None => {
                self.add(name);
                self.open(name)?
            }
        };
        Ok(())
    }
    //pub fn new() -> Self {
    //    Logs(HashMap::new())
    //}
    //pub fn update(line: &Line) -> Result<usize,io::Error> {
    //    Ok(0)
    //}
}
*/

