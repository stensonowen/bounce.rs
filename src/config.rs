use std::collections::HashMap;
use std::io::{self, Read};
use std::fs::File;
use rpassword;
use toml;
use slog;

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
    register: Option<bool>,
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
    pub register: Option<String>,
    pub tls: bool,
    pub log_path: Option<String>,
    //pub do_log: bool,
    pub logger: slog::Logger,

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
             log_dir: &str,
             alt_n: &Option<String>, 
             alt_u: &Option<String>,
             alt_m: &Option<u8>, 
             alt_r: &Option<String>,
             logger: &slog::Logger,
             ) -> Result<Server,String> {
        let server_pw = if self.password.unwrap_or(false) {
            let prompt = format!("Enter server password for {}: ", serv_name);
            let pw = rpassword::prompt_password_stdout(&prompt)
                .map_err(|e| e.description().to_owned())?;
            Some(pw)
        } else {
            None
        };
        let regis_pw = if self.register.unwrap_or(false) {
            let prompt = format!("Enter registration password for {}: ", serv_name);
            let pw = rpassword::prompt_password_stdout(&prompt)
                .map_err(|e| e.description().to_owned())?;
            Some(pw)
        } else {
            None
        };

        Ok(Server {
            logger: logger.new(o!("Name" => serv_name.to_owned(), 
                                  "IP"   => self.addr.to_owned(), 
                                  "Nick" => self.nick.to_owned())),
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
            password: server_pw,
            register: regis_pw,
            tls: self.tls.unwrap_or(DEFAULT_TLS),
            //do_log: self.log.unwrap_or(DEFAULT_LOG),
            //log_path: log_dir.to_owned(),
            log_path: match self.log {
                Some(true)          => Some(log_dir.to_owned()),
                None if DEFAULT_LOG => Some(log_dir.to_owned()),
                Some(false)         => None,
                None                => None,
            },
        })
    }
}

fn build_servers(olds: HashMap<String,ServerToml>, 
                 alt_nick: Option<String>, 
                 alt_user: Option<String>, 
                 alt_mode: Option<u8>, 
                 alt_real: Option<String>,
                 log_dir: &str,
                 logger: &slog::Logger,
                 ) -> Result<HashMap<String,Server>,String> {
    let mut servers = HashMap::new();
    for (name,serv) in olds {
        let new = serv.build(&name, log_dir, &alt_nick, &alt_user, &alt_mode, &alt_real, logger)?;
        servers.insert(name.to_string(),new);
    }
    Ok(servers)
}

impl ConfigToml {
    fn build(self, log: &slog::Logger) -> Result<Config,String> {
        Ok(Config {
            servers: build_servers(self.servers, self.nick, 
                                   self.user, self.mode, self.realname, &self.logs_dir, log)?,
            logs_dir: self.logs_dir,
            timefmt: self.timefmt,
            utc: self.utc.unwrap_or(DEFAULT_UTC),
        })
    }
}

impl Server {
    pub fn conn_msg(&self) -> Vec<String> {
        let mut cm = Vec::new();
        if let Some(ref srv_pw) = self.password {
            cm.push(format!("PASSWORD {}", srv_pw));
        }
        cm.push(format!("USER {} {} * {}", self.user, self.mode, self.realname));
        cm.push(format!("NICK {}", self.nick));
        if let Some(ref keys) = self.chan_keys {
            cm.push(format!("JOIN {} {}", self.chans.join(","), keys.join(",")));
        } else {
            cm.push(format!("JOIN {}", self.chans.join(",")));
        }
        if let Some(ref reg_pw) = self.register {
            cm.push(format!("PRIVMSG NickServ IDENTIFY {} {}", self.nick, reg_pw));
        }
        cm
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
    /*
    pub fn get_dir(&self) -> &Path {
        Path::new(self.log_path.unwrap())
    }
    */
}

impl Config {
    pub fn from(path: &str, log: &slog::Logger) -> Result<Config,ParseError> {
        use self::ParseError::*;
        let mut f = File::open(path).map_err(|e| ReadError(e))?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(|e| ReadError(e))?;
        let config: ConfigToml = toml::from_str(&s).map_err(|e| ParseError(e))?;
        Ok(config.build(log).map_err(|e| ResolveError(e))?)
    }
}

