use std::collections::HashMap;
use std::io::{self, Read};
use std::fs::File;
use rpassword;
use toml;

const DEFAULT_UTC: bool = false;
const DEFAULT_TLS: bool = false;
const DEFAULT_LOG: bool = true;
const DEFAULT_MODE: u8 = 0;



#[derive(Deserialize)]
struct ConfigToml {
    nick: Option<String>,
    logs_dir: String,
    timefmt: Option<String>,
    utc: Option<bool>,
    user: Option<String>,
    mode: Option<u8>,
    realname: Option<String>,
    servers: HashMap<String,ServerToml>,
}

#[derive(Deserialize)]
struct ServerToml {
    addr: String,
    chans: Vec<String>,
    chan_keys: Option<Vec<String>>,
    tls: Option<bool>,
    password: Option<bool>,
    nick: Option<String>,
    log: Option<bool>,
    user: Option<String>,
    mode: Option<u8>,
    realname: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub logs_dir: String,
    pub timefmt: Option<String>,
    pub servers: HashMap<String,Server>,
    pub utc: bool,
}

#[derive(Debug)]
pub struct Server {
    pub nick: String,
    pub user: String,
    pub mode: u8,
    pub realname: String,
    addr: String,
    pub chans: Vec<String>,
    pub chan_keys: Option<Vec<String>>,
    pub password: Option<String>,
    pub tls: bool,
    pub log: bool,
}

#[derive(Debug)]
pub enum ParseError {
    ReadError(io::Error),
    ParseError(toml::de::Error),
    ResolveError(String),
}

use std::error::Error;
impl ServerToml {
    fn build(self, 
             serv_name: &str,
             alt_n: &Option<String>, 
             alt_u: &Option<String>,
             alt_m: &Option<u8>, 
             alt_r: &Option<String>
             ) -> Result<Server,String> {
        let pw = if self.password.unwrap_or(false) {
            let prompt = format!("Please enter password for {}: ", serv_name);
            let pw = rpassword::prompt_password_stdout(&prompt)
                .map_err(|e| e.description().to_owned())?;
            Some(pw)
        } else {
            None
        };
        Ok(Server {
            nick: self.nick.or(alt_n.clone())
                .ok_or(format!("`Nick` field missing from {}", serv_name))?,
            user: self.user.or(alt_u.clone())
                .ok_or(format!("`User` field missing from {}", serv_name))?,
            mode: self.mode.or(alt_m.clone()).unwrap_or(DEFAULT_MODE),
            realname: self.realname.or(alt_r.clone())
                .ok_or(format!("`Realname` field missing from {}", serv_name))?,
            addr: self.addr,
            chans: self.chans,
            chan_keys: self.chan_keys,
            password: pw,
            tls: self.tls.unwrap_or(DEFAULT_TLS),
            log: self.log.unwrap_or(DEFAULT_LOG),
        })
    }
}

/*
fn build_server(old: ServerToml, 
                alt_nick: &Option<String>, 
                alt_user: &Option<String>, 
                alt_mode: &Option<u8>, 
                alt_real: &Option<String>
                ) -> Result<Server,String> {
    Ok(Server {
        nick: old.nick.or(alt_nick.clone()).ok_or("`Nick` required".to_string())?,
        user: old.user.or(alt_user.clone()).ok_or("`User` required".to_string())?,
        mode: old.mode.or(alt_mode.clone()).unwrap_or(DEFAULT_MODE),
        realname: old.realname.or(alt_real.clone())
            .ok_or("`Realname` required".to_string())?,
        addr: old.addr,
        chans: old.chans,
        chan_keys: old.chan_keys,
        password: old.password.unwrap_or(false),
        tls: old.tls.unwrap_or(DEFAULT_TLS),
        log: old.log.unwrap_or(DEFAULT_LOG),
    })
}
*/

fn build_servers(olds: HashMap<String,ServerToml>, 
                 alt_nick: Option<String>, 
                 alt_user: Option<String>, 
                 alt_mode: Option<u8>, 
                 alt_real: Option<String>
                 ) -> Result<HashMap<String,Server>,String> {
    let mut servers = HashMap::new();
    for (name,serv) in olds {
        let new = serv.build(&name, &alt_nick, &alt_user, &alt_mode, &alt_real)?;
        servers.insert(name.to_string(),new);
    }
    Ok(servers)
}

impl ConfigToml {
    fn build(self) -> Result<Config,String> {
        Ok(Config {
            servers: build_servers(self.servers, self.nick, 
                                   self.user, self.mode, self.realname)?,
            logs_dir: self.logs_dir,
            timefmt: self.timefmt,
            utc: self.utc.unwrap_or(DEFAULT_UTC),
        })
    }
}

/*
fn build_config(old: ConfigToml) -> Result<Config,String> {
    Ok(Config {
        servers: build_servers(old.servers, old.nick, 
                               old.user, old.mode, old.realname)?,
        logs_dir: old.logs_dir,
        timefmt: old.timefmt,
        utc: old.utc.unwrap_or(DEFAULT_UTC),
    })
}
*/
impl Server {
    pub fn conn_msg(&self) -> Vec<String> {
        vec![
            format!("USER {} {} * {}", self.user, self.mode, self.realname), 
            format!("NICK {}", self.nick), 
            if let Some(ref keys) = self.chan_keys {
                format!("JOIN {} {}", self.chans.join(", "), keys.join(", "))
            } else {
                format!("JOIN {}", self.chans.join(", "))
            }
        ]
    }
    pub fn get_addr(&self) -> String {
        if self.addr.contains(':') {
            self.addr.clone()
        } else {
            let mut s = self.addr.clone();
            s.push(':');
            if self.tls {
                s.push_str("6697");
            } else {
                s.push_str("6667");
            }
            s
        }
    }
}

impl Config {
    //pub fn from(path: &str) -> io::Result<Config> {
    pub fn from(path: &str) -> Result<Config,ParseError> {
        use self::ParseError::*;
        let mut f = File::open(path).map_err(|e| ReadError(e))?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(|e| ReadError(e))?;
        let config: ConfigToml = toml::from_str(&s).map_err(|e| ParseError(e))?;
        Ok(config.build().map_err(|e| ResolveError(e))?)
    }
}

/*
pub fn parse_config(config_path: &str) -> io::Result<Config> {
    let mut f = File::open(config_path)?;
    let mut text = String::new();
    f.read_to_string(&mut text)?;
    let config = toml::from_str(&text).unwrap();
    Ok(build_config(config).unwrap())
}
*/
