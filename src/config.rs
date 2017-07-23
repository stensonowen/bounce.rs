/* TODO: when(if?) serde supports it, deserialize arrays to `&'a [&'a str]` 
 *  instead of `Vec<&'a str>`
 */

use std::collections::HashMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::net::{SocketAddr, ToSocketAddrs};
use std::fs::File;
use toml;
use std::borrow::Cow;

const DEFAULT_UTC: bool = false;
const DEFAULT_TLS: bool = false;
const DEFAULT_LOG: bool = true;
const DEFAULT_MODE: u8 = 0;


#[derive(Deserialize)]
struct ConfigToml<'a> {
    nick:       Option<&'a str>,
    nick_alt:   Option<&'a str>,
    logs_dir:   &'a str,
    timefmt:    Option<&'a str>,
    utc:        Option<bool>,
    tls:        Option<bool>,
    user:       Option<&'a str>,
    mode:       Option<u8>,
    realname:   Option<&'a str>,
    servers:    HashMap<&'a str, ServerToml<'a>>,
}

#[derive(Deserialize)]
struct ServerToml<'a> {
    addr:       &'a str,
    chans:      Vec<&'a str>,
    chan_keys:  Option<Vec<&'a str>>,
    tls:        Option<bool>,
    password:   Option<bool>,
    register:   Option<bool>,
    nick:       Option<&'a str>,
    nick_alt:   Option<&'a str>,
    log:        Option<bool>,
    user:       Option<&'a str>,
    mode:       Option<u8>,
    realname:   Option<&'a str>,
}

#[derive(Debug)]
pub struct Config<'a> {
    pub logs_dir:   PathBuf,
    pub timefmt:    Option<&'a str>,
    pub servers:    HashMap<&'a str, Server<'a>>,
    pub utc:        bool,
}

#[derive(Debug)]
pub struct Server<'a> {
    pub nick:       &'a str,
    pub nick_alt:   Option<&'a str>,
    pub user:       &'a str,
    pub mode:       u8,
    pub realname:   &'a str,
    pub addr:       SocketAddr,
    pub chans:      Vec<&'a str>,
    pub chan_keys:  Option<Vec<&'a str>>,
    pub password:   Option<String>,
    pub register:   Option<String>,
    pub tls:        bool,
    pub log:        bool,
}

#[derive(Debug)]
pub enum ConfigError<'a> {
    ReadError(io::Error),
    ParseError(toml::de::Error),
    ResolveError(String),
    MissingData { field: &'static str, from: &'a str },
    BadAddress,
}

impl<'a> Config<'a> {
    pub fn load_to_string<P: AsRef<Path>>(path: P, s: &'a mut String) 
            -> Result<Config<'a>,ConfigError<'a>> {
        use self::ConfigError::*;
        let mut f = File::open(path).map_err(ReadError)?;
        f.read_to_string(s).map_err(ReadError)?;

        let config: ConfigToml = toml::from_str(s).map_err(ParseError)?;
        config.build()
    }
}

impl<'a> ConfigToml<'a> {
    fn build(self) -> Result<Config<'a>,ConfigError<'a>> {
        use rpassword::prompt_password_stdout;
        use self::ConfigError::*;
        let mut servers: HashMap<&'a str, Server> = HashMap::new();
        for (name,srv) in self.servers {
            let server_pw = if srv.password.unwrap_or(false) {
                let prompt = format!("Enter server password for {}: ", name);
                let pw = prompt_password_stdout(&prompt).map_err(ReadError)?;
                Some(pw)
            } else {
                None
            };
            let regis_pw = if srv.register.unwrap_or(false) {
                let prompt = format!("Enter registration password for {}: ", name);
                let pw = prompt_password_stdout(&prompt).map_err(ReadError)?;
                Some(pw)
            } else {
                None
            };
            let from = name;
            let nick = srv.nick.or(self.nick).ok_or(MissingData{field:"Nick", from})?;
            let user = srv.user.or(self.user).ok_or(MissingData{field:"User", from})?;
            let mode = srv.mode.or(self.mode).unwrap_or(DEFAULT_MODE);
            let real = srv.realname.or(self.realname).unwrap_or(user);
            let tls = srv.tls.or(self.tls).unwrap_or(DEFAULT_TLS);
            let nick_alt = srv.nick_alt.or(self.nick_alt);

            let addr_s: Cow<str> = if srv.addr.contains(':') {
                Cow::Borrowed(srv.addr)
            } else {
                Cow::Owned(format!("{}:{}", srv.addr, if tls {6697} else {6667}))
            };
            let mut addr_i = addr_s.to_socket_addrs().map_err(ReadError)?;
            let addr = addr_i.nth(0).ok_or(BadAddress)?;

            let server = Server { nick, nick_alt, user, mode, addr, 
                realname: real, chans: srv.chans, chan_keys: srv.chan_keys, 
                password: server_pw, register: regis_pw, 
                tls: srv.tls.unwrap_or(DEFAULT_TLS),
                log: srv.log.unwrap_or(DEFAULT_LOG),
            };
            servers.insert(name,server);
        }

        Ok(Config {     servers,
            logs_dir:   PathBuf::from(self.logs_dir),
            timefmt:    self.timefmt,
            utc:        self.utc.unwrap_or(DEFAULT_UTC),
        })
    }
}

