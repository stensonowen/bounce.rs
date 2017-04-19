// TODO: make custom UserName, ChannelName, ServerTarget, etc. that can only
//  be formed from valid strings (e.g. don't contain forbidden characters) ?

pub enum Mode {
    a,
    i,
    w,
    r,
    o,
    O,
    s,
}

pub struct ModeMsg {
    plus: bool,
    modes: Vec<Mode>,
    modeparam: Option<String>,
}

pub enum JoinMsg {
    Join{channels: Vec<String>, keys: Option<Vec<String>>},
    Leave,
}
