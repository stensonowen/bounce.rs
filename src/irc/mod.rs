// https://tools.ietf.org/html/rfc2812#section-3
mod helpers;
use self::helpers::*;

#[allow(dead_code)]
enum CmdCodec {
    Conn(ConnCmd),
    Chan(ChanCmd),
    Mesg(MesgCmd),
    Serv(ServCmd),
    Qury(QuryCmd),
    User(UserCmd),
    Misc(MiscCmd),
    Optn(OptnCmd),
}


enum ConnCmd {
    Pass{password: String},
    Nick{nickname: String},
    User{user: String, mode: u8, realname: String}, // TODO: unused?
    Oper{name: String, password: String},
    Mode{nickname: String, flags: Vec<ModeMsg>},    // TODO
    Service{nickname: String, r1: String, distribution: String, 
        t1: String, r2: String, info: String},      // TODO: reserved?
    Quit,
    Squit{server: String, comment: String},
}

enum ChanCmd { 
    Join(JoinMsg),
    Part{channels: Vec<String>, message: Option<String>},
    Mode{channel: String, modes: Vec<ModeMsg>},
    Topic{channel: String, topic: Option<String>},
    Names{channels: Vec<String>, target: Option<String>},
    List{channels: Vec<String>, target: Option<String>},
    Invite{nickname: String, channel: String},
    Kick{channels: Vec<String>, users: Vec<String>, comment: Option<String>},
}

enum MesgCmd {
    PrivMsg{msgtarget: String, text: String},
    Notice{msgtarget: String, text: String},
}

enum ServCmd {
    Motd{target: Option<String>},
    Lusers{mask_target: Option<(String, Option<String>)>},
    Version{target: Option<String>},
    Stats{query_target: Option<(String, Option<String>)>},
    Links{remote_servermask: Option<(Option<String>, String)>},
    Time{target: Option<String>},
    Connect{target_server: String, port: u16, remote_server: Option<String>},
    Trace{target: Option<String>},
    Admin{target: Option<String>},
    Info{target: Option<String>},
}

enum QuryCmd {
    ServList{mask_type: Option<(String, Option<String>)>},
    Squery{servicename: String, text: String},
}

enum UserCmd {
    Who{mask_o: Option<(String, bool)>},
    Whois{target: Option<String>, masks: Vec<String>},
    Whowas{nicknames: Vec<String>, count_target: Option<(u32, Option<String>)>},
}

enum MiscCmd {
    Kill{name: String, comment: String},
    Ping{server1: String, server2: Option<String>},
    Pong{server1: String, server2: Option<String>},
    Error{message: String},
}

enum OptnCmd {
    Away{text: String},
    Rehash,
    Die,
    Restart,
    Summon{user: String, target_channel: Option<(String, Option<String>)>},
    Users{target: Option<String>},
    Operwall{text: String},
    Userhost{nicknames: Vec<String>},
    Ison{nicknames: Vec<String>},
}
