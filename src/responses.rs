
// https://tools.ietf.org/html/rfc2812#section-5.1
#[allow(non_camel_case_types)]
pub enum CmdRsp {
    RPL_WELCOME,
    RPL_YOURHOST,
    RPL_CREATED,
    RPL_MYINFO,
    RPL_BOUNCE,
    RPL_USERHOST,
    RPL_ISON,
    RPL_AWAY,
    RPL_UNAWAY,
    RPL_NOWAWAY,
    RPL_WHOISUSER,
    RPL_WHOISSERVER,
    RPL_WHOISOPERATOR,
    RPL_WHOISIDLE,
    RPL_ENDOFWHOIS,
    RPL_WHOISCHANNELS,
    RPL_WHOWASUSER,
    RPL_ENDOFWHOWAS,
    RPL_LISTSTART,
    RPL_LIST,
    RPL_LISTEND,
    RPL_UNIQOPIS,
    RPL_CHANNELMODEIS,
    RPL_NOTOPIC,
    RPL_TOPIC,
    RPL_INVITING,
    RPL_SUMMONING,
    RPL_INVITELIST,
    RPL_ENDOFINVITELIST,
    RPL_EXCEPTLIST,
    RPL_ENDOFEXCEPTLIST,
    RPL_VERSION,
    RPL_WHOREPLY,
    RPL_ENDOFWHO,
    RPL_NAMREPLY,
    RPL_ENDOFNAMES,
    RPL_LINKS,
    RPL_ENDOFLINKS,
    RPL_BANLIST,
    RPL_ENDOFBANLIST,
    RPL_INFO,
    RPL_ENDOFINFO,
    RPL_MOTDSTART,
    RPL_MOTD,
    RPL_ENDOFMOTD,
    RPL_YOUREOPER,
    RPL_REHASHING,
    RPL_YOURESERVICE,
    RPL_TIME,
    RPL_USERSSTART,
    RPL_USERS,
    RPL_ENDOFUSERS,
    RPL_NOUSERS,
    RPL_TRACELINK,
    RPL_TRACECONNECTING,
    RPL_TRACEHANDSHAKE,
    RPL_TRACEUNKNOWN,
    RPL_TRACEOPERATOR,
    RPL_TRACEUSER,
    RPL_TRACESERVER,
    RPL_TRACESERVICE,
    RPL_TRACENEWTYPE,
    RPL_TRACECLASS,
    RPL_TRACERECONNECT,
    RPL_TRACELOG,
    RPL_TRACEEND,
    RPL_STATSLINKINFO,
    RPL_STATSCOMMANDS,
    RPL_ENDOFSTATS,
    RPL_STATSUPTIME,
    RPL_STATSOLINE,
    RPL_UMODEIS,
    RPL_SERVLIST,
    RPL_SERVLISTEND,
    RPL_LUSERCLIENT,
    RPL_LUSEROP,
    RPL_LUSERUNKNOWN,
    RPL_LUSERCHANNELS,
    RPL_LUSERME,
    RPL_ADMINME,
    RPL_ADMINLOC1,
    RPL_ADMINLOC2,
    RPL_ADMINEMAIL,
    RPL_TRYAGAIN,
}

impl CmdRsp {
    fn from(code: u16) -> Option<CmdRsp> {
        match code {
            001 => Some(CmdRsp::RPL_WELCOME),
            002 => Some(CmdRsp::RPL_YOURHOST),
            003 => Some(CmdRsp::RPL_CREATED),
            004 => Some(CmdRsp::RPL_MYINFO),
            005 => Some(CmdRsp::RPL_BOUNCE),
            302 => Some(CmdRsp::RPL_USERHOST),
            303 => Some(CmdRsp::RPL_ISON),
            301 => Some(CmdRsp::RPL_AWAY),
            305 => Some(CmdRsp::RPL_UNAWAY),
            306 => Some(CmdRsp::RPL_NOWAWAY),
            311 => Some(CmdRsp::RPL_WHOISUSER),
            312 => Some(CmdRsp::RPL_WHOISSERVER),
            313 => Some(CmdRsp::RPL_WHOISOPERATOR),
            317 => Some(CmdRsp::RPL_WHOISIDLE),
            318 => Some(CmdRsp::RPL_ENDOFWHOIS),
            319 => Some(CmdRsp::RPL_WHOISCHANNELS),
            314 => Some(CmdRsp::RPL_WHOWASUSER),
            369 => Some(CmdRsp::RPL_ENDOFWHOWAS),
            321 => Some(CmdRsp::RPL_LISTSTART),
            322 => Some(CmdRsp::RPL_LIST),
            323 => Some(CmdRsp::RPL_LISTEND),
            325 => Some(CmdRsp::RPL_UNIQOPIS),
            324 => Some(CmdRsp::RPL_CHANNELMODEIS),
            331 => Some(CmdRsp::RPL_NOTOPIC),
            332 => Some(CmdRsp::RPL_TOPIC),
            341 => Some(CmdRsp::RPL_INVITING),
            342 => Some(CmdRsp::RPL_SUMMONING),
            346 => Some(CmdRsp::RPL_INVITELIST),
            347 => Some(CmdRsp::RPL_ENDOFINVITELIST),
            348 => Some(CmdRsp::RPL_EXCEPTLIST),
            349 => Some(CmdRsp::RPL_ENDOFEXCEPTLIST),
            351 => Some(CmdRsp::RPL_VERSION),
            352 => Some(CmdRsp::RPL_WHOREPLY),
            315 => Some(CmdRsp::RPL_ENDOFWHO),
            353 => Some(CmdRsp::RPL_NAMREPLY),
            366 => Some(CmdRsp::RPL_ENDOFNAMES),
            364 => Some(CmdRsp::RPL_LINKS),
            365 => Some(CmdRsp::RPL_ENDOFLINKS),
            367 => Some(CmdRsp::RPL_BANLIST),
            368 => Some(CmdRsp::RPL_ENDOFBANLIST),
            371 => Some(CmdRsp::RPL_INFO),
            374 => Some(CmdRsp::RPL_ENDOFINFO),
            375 => Some(CmdRsp::RPL_MOTDSTART),
            372 => Some(CmdRsp::RPL_MOTD),
            376 => Some(CmdRsp::RPL_ENDOFMOTD),
            381 => Some(CmdRsp::RPL_YOUREOPER),
            382 => Some(CmdRsp::RPL_REHASHING),
            383 => Some(CmdRsp::RPL_YOURESERVICE),
            391 => Some(CmdRsp::RPL_TIME),
            392 => Some(CmdRsp::RPL_USERSSTART),
            393 => Some(CmdRsp::RPL_USERS),
            394 => Some(CmdRsp::RPL_ENDOFUSERS),
            395 => Some(CmdRsp::RPL_NOUSERS),
            200 => Some(CmdRsp::RPL_TRACELINK),
            201 => Some(CmdRsp::RPL_TRACECONNECTING),
            202 => Some(CmdRsp::RPL_TRACEHANDSHAKE),
            203 => Some(CmdRsp::RPL_TRACEUNKNOWN),
            204 => Some(CmdRsp::RPL_TRACEOPERATOR),
            205 => Some(CmdRsp::RPL_TRACEUSER),
            206 => Some(CmdRsp::RPL_TRACESERVER),
            207 => Some(CmdRsp::RPL_TRACESERVICE),
            208 => Some(CmdRsp::RPL_TRACENEWTYPE),
            209 => Some(CmdRsp::RPL_TRACECLASS),
            210 => Some(CmdRsp::RPL_TRACERECONNECT),
            261 => Some(CmdRsp::RPL_TRACELOG),
            262 => Some(CmdRsp::RPL_TRACEEND),
            211 => Some(CmdRsp::RPL_STATSLINKINFO),
            212 => Some(CmdRsp::RPL_STATSCOMMANDS),
            219 => Some(CmdRsp::RPL_ENDOFSTATS),
            242 => Some(CmdRsp::RPL_STATSUPTIME),
            243 => Some(CmdRsp::RPL_STATSOLINE),
            221 => Some(CmdRsp::RPL_UMODEIS),
            234 => Some(CmdRsp::RPL_SERVLIST),
            235 => Some(CmdRsp::RPL_SERVLISTEND),
            251 => Some(CmdRsp::RPL_LUSERCLIENT),
            252 => Some(CmdRsp::RPL_LUSEROP),
            253 => Some(CmdRsp::RPL_LUSERUNKNOWN),
            254 => Some(CmdRsp::RPL_LUSERCHANNELS),
            255 => Some(CmdRsp::RPL_LUSERME),
            256 => Some(CmdRsp::RPL_ADMINME),
            257 => Some(CmdRsp::RPL_ADMINLOC1),
            258 => Some(CmdRsp::RPL_ADMINLOC2),
            259 => Some(CmdRsp::RPL_ADMINEMAIL),
            263 => Some(CmdRsp::RPL_TRYAGAIN),
            _   => None,
        }
    }
}

// https://tools.ietf.org/html/rfc2812#section-5.2
#[allow(non_camel_case_types)]
pub enum ErrRsp {
    ERR_NOSUCHNICK,
    ERR_NOSUCHSERVER,
    ERR_NOSUCHCHANNEL,
    ERR_CANNOTSENDTOCHAN,
    ERR_TOOMANYCHANNELS,
    ERR_WASNOSUCHNICK,
    ERR_TOOMANYTARGETS,
    ERR_NOSUCHSERVICE,
    ERR_NOORIGIN,
    ERR_NORECIPIENT,
    ERR_NOTEXTTOSEND,
    ERR_NOTOPLEVEL,
    ERR_WILDTOPLEVEL,
    ERR_BADMASK,
    ERR_UNKNOWNCOMMAND,
    ERR_NOMOTD,
    ERR_NOADMININFO,
    ERR_FILEERROR,
    ERR_NONICKNAMEGIVEN,
    ERR_ERRONEUSNICKNAME,
    ERR_NICKNAMEINUSE,
    ERR_NICKCOLLISION,
    ERR_UNAVAILRESOURCE,
    ERR_USERNOTINCHANNEL,
    ERR_NOTONCHANNEL,
    ERR_USERONCHANNEL,
    ERR_NOLOGIN,
    ERR_SUMMONDISABLED,
    ERR_USERSDISABLED,
    ERR_NOTREGISTERED,
    ERR_NEEDMOREPARAMS,
    ERR_ALREADYREGISTRED,
    ERR_NOPERMFORHOST,
    ERR_PASSWDMISMATCH,
    ERR_YOUREBANNEDCREEP,
    ERR_YOUWILLBEBANNED,
    ERR_KEYSET,
    ERR_CHANNELISFULL,
    ERR_UNKNOWNMODE,
    ERR_INVITEONLYCHAN,
    ERR_BANNEDFROMCHAN,
    ERR_BADCHANNELKEY,
    ERR_BADCHANMASK,
    ERR_NOCHANMODES,
    ERR_BANLISTFULL,
    ERR_NOPRIVILEGES,
    ERR_CHANOPRIVSNEEDED,
    ERR_CANTKILLSERVER,
    ERR_RESTRICTED,
    ERR_UNIQOPPRIVSNEEDED,
    ERR_NOOPERHOST,
    ERR_UMODEUNKNOWNFLAG,
    ERR_USERSDONTMATCH,
}

impl ErrRsp {
    fn from(code: u16) -> Option<ErrRsp> {
        match code {
            401 => Some(ErrRsp::ERR_NOSUCHNICK),
            402 => Some(ErrRsp::ERR_NOSUCHSERVER),
            403 => Some(ErrRsp::ERR_NOSUCHCHANNEL),
            404 => Some(ErrRsp::ERR_CANNOTSENDTOCHAN),
            405 => Some(ErrRsp::ERR_TOOMANYCHANNELS),
            406 => Some(ErrRsp::ERR_WASNOSUCHNICK),
            407 => Some(ErrRsp::ERR_TOOMANYTARGETS),
            408 => Some(ErrRsp::ERR_NOSUCHSERVICE),
            409 => Some(ErrRsp::ERR_NOORIGIN),
            411 => Some(ErrRsp::ERR_NORECIPIENT),
            412 => Some(ErrRsp::ERR_NOTEXTTOSEND),
            413 => Some(ErrRsp::ERR_NOTOPLEVEL),
            414 => Some(ErrRsp::ERR_WILDTOPLEVEL),
            415 => Some(ErrRsp::ERR_BADMASK),
            421 => Some(ErrRsp::ERR_UNKNOWNCOMMAND),
            422 => Some(ErrRsp::ERR_NOMOTD),
            423 => Some(ErrRsp::ERR_NOADMININFO),
            424 => Some(ErrRsp::ERR_FILEERROR),
            431 => Some(ErrRsp::ERR_NONICKNAMEGIVEN),
            432 => Some(ErrRsp::ERR_ERRONEUSNICKNAME),
            433 => Some(ErrRsp::ERR_NICKNAMEINUSE),
            436 => Some(ErrRsp::ERR_NICKCOLLISION),
            437 => Some(ErrRsp::ERR_UNAVAILRESOURCE),
            441 => Some(ErrRsp::ERR_USERNOTINCHANNEL),
            442 => Some(ErrRsp::ERR_NOTONCHANNEL),
            443 => Some(ErrRsp::ERR_USERONCHANNEL),
            444 => Some(ErrRsp::ERR_NOLOGIN),
            445 => Some(ErrRsp::ERR_SUMMONDISABLED),
            446 => Some(ErrRsp::ERR_USERSDISABLED),
            451 => Some(ErrRsp::ERR_NOTREGISTERED),
            461 => Some(ErrRsp::ERR_NEEDMOREPARAMS),
            462 => Some(ErrRsp::ERR_ALREADYREGISTRED),
            463 => Some(ErrRsp::ERR_NOPERMFORHOST),
            464 => Some(ErrRsp::ERR_PASSWDMISMATCH),
            465 => Some(ErrRsp::ERR_YOUREBANNEDCREEP),
            466 => Some(ErrRsp::ERR_YOUWILLBEBANNED),
            467 => Some(ErrRsp::ERR_KEYSET),
            471 => Some(ErrRsp::ERR_CHANNELISFULL),
            472 => Some(ErrRsp::ERR_UNKNOWNMODE),
            473 => Some(ErrRsp::ERR_INVITEONLYCHAN),
            474 => Some(ErrRsp::ERR_BANNEDFROMCHAN),
            475 => Some(ErrRsp::ERR_BADCHANNELKEY),
            476 => Some(ErrRsp::ERR_BADCHANMASK),
            477 => Some(ErrRsp::ERR_NOCHANMODES),
            478 => Some(ErrRsp::ERR_BANLISTFULL),
            481 => Some(ErrRsp::ERR_NOPRIVILEGES),
            482 => Some(ErrRsp::ERR_CHANOPRIVSNEEDED),
            483 => Some(ErrRsp::ERR_CANTKILLSERVER),
            484 => Some(ErrRsp::ERR_RESTRICTED),
            485 => Some(ErrRsp::ERR_UNIQOPPRIVSNEEDED),
            _   => None,
        }
    }
}

// https://tools.ietf.org/html/rfc2812#section-5.3
#[allow(non_camel_case_types)]
pub enum MscRsp {
    RPL_SERVICEINFO,
    RPL_ENDOFSERVICES,
    RPL_SERVICE,
    RPL_NONE,
    RPL_WHOISCHANOP,
    RPL_KILLDONE,
    RPL_CLOSING,
    RPL_CLOSEEND,
    RPL_INFOSTART,
    RPL_MYPORTIS,
    RPL_STATSCLINE,
    RPL_STATSNLINE,
    RPL_STATSILINE,
    RPL_STATSKLINE,
    RPL_STATSQLINE,
    RPL_STATSYLINE,
    RPL_STATSVLINE,
    RPL_STATSLLINE,
    RPL_STATSHLINE,
    RPL_STATSSLINE,
    RPL_STATSPING,
    RPL_STATSBLINE,
    RPL_STATSDLINE,
    ERR_NOSERVICEHOST,
}

impl MscRsp {
    fn from(code: u16) -> Option<MscRsp> {
        match code {
            231 => Some(MscRsp::RPL_SERVICEINFO),
            232 => Some(MscRsp::RPL_ENDOFSERVICES),
            233 => Some(MscRsp::RPL_SERVICE),
            300 => Some(MscRsp::RPL_NONE),
            316 => Some(MscRsp::RPL_WHOISCHANOP),
            361 => Some(MscRsp::RPL_KILLDONE),
            362 => Some(MscRsp::RPL_CLOSING),
            363 => Some(MscRsp::RPL_CLOSEEND),
            373 => Some(MscRsp::RPL_INFOSTART),
            384 => Some(MscRsp::RPL_MYPORTIS),
            213 => Some(MscRsp::RPL_STATSCLINE),
            214 => Some(MscRsp::RPL_STATSNLINE),
            215 => Some(MscRsp::RPL_STATSILINE),
            216 => Some(MscRsp::RPL_STATSKLINE),
            217 => Some(MscRsp::RPL_STATSQLINE),
            218 => Some(MscRsp::RPL_STATSYLINE),
            240 => Some(MscRsp::RPL_STATSVLINE),
            241 => Some(MscRsp::RPL_STATSLLINE),
            244 => Some(MscRsp::RPL_STATSHLINE),
            246 => Some(MscRsp::RPL_STATSPING),
            247 => Some(MscRsp::RPL_STATSBLINE),
            250 => Some(MscRsp::RPL_STATSDLINE),
            492 => Some(MscRsp::ERR_NOSERVICEHOST),
            _   => None,
        }
    }
}


