use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::collections::HashMap;
use std::borrow::Cow;

pub struct LogFile(BufWriter<File>);
pub struct Logs(HashMap<String,LogFile>, PathBuf);

impl Logs {
    pub fn new(path: &str) -> Self {
        Logs(HashMap::new(), PathBuf::from(path))
    }
    pub fn add(&mut self, name: String) -> io::Result<()> {
        let mut path = self.1.clone();
        path.push(&name);
        let f = OpenOptions::new().write(true).create(true).open(path)?;
        let bw = BufWriter::new(f);
        self.0.insert(name, LogFile(bw));
        Ok(())
    }
    pub fn write(&mut self, name: Cow<str>, text: &str) -> io::Result<()> {
        if self.0.contains_key(name.as_ref()) == false {
            self.add(name.to_string())?;
        }
        if let Some(&mut LogFile(ref mut bw)) = self.0.get_mut(name.as_ref()) {
            bw.write_all(text.as_bytes())
                ?;bw.flush()    // TODO: comment this line out (just for debugging)
        } else {
            // error?
            Ok(())
        }
    }
}

/*
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
*/
