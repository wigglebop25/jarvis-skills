use regex::Regex;
use std::sync::OnceLock;
use super::IntentType;

pub enum ParamExtractor {
    List(Vec<(Regex, &'static str)>),
    Regex(Regex),
}

impl IntentType {
    pub fn patterns(&self) -> &'static [Regex] {
        match self {
            Self::DirectoryList => {
                static RE: OnceLock<Vec<Regex>> = OnceLock::new();
                RE.get_or_init(|| {
                    vec![
                        Regex::new(r"(?i)(show|list|see|view)\s*(the)?\s*(content|contents|folders|files)").unwrap(),
                        Regex::new(r"(?i)(what('s|s| is)\s*(inside|in))\s*(the)?\s*(drive|folder)").unwrap(),
                        Regex::new(r"(?i)(folders?|files?)\s*(inside|in)\s*(the)?\s*(drive|folder)").unwrap(),
                        Regex::new(r"(?i)(list|show)\s*(folders?|files?)\s*(in|inside)\s*[a-z]:").unwrap(),
                        Regex::new(r"(?i)(drive)\s*[a-z]\s*(content|contents|folders|files)").unwrap(),
                        Regex::new(r"(?i)(content|contents|contend)\s*(in|inside|of)\s*(the)?\s*[a-z]\b").unwrap(),
                    ]
                })
            }
            Self::SystemInfo => {
                static RE: OnceLock<Vec<Regex>> = OnceLock::new();
                RE.get_or_init(|| {
                    vec![
                        Regex::new(r"(?i)^system\s*(info|information|status)$").unwrap(),
                        Regex::new(r"(?i)system\s*(usage|utilization)").unwrap(),
                        Regex::new(r"(?i)get\s*system\s*(info|information|status|usage)").unwrap(),
                        Regex::new(r"(?i)get_system_info").unwrap(),
                        Regex::new(r"(?i)(cpu|processor)\s*(usage|load|status)?").unwrap(),
                        Regex::new(r"(?i)(ram|memory)\s*(usage|status)?").unwrap(),
                        Regex::new(r"(?i)(storage|disk)\s*(space|usage|status)").unwrap(),
                        Regex::new(r"(?i)drive\s*(space|usage|status)").unwrap(),
                        Regex::new(r"(?i)(network|internet)\s*(status|connection)$").unwrap(),
                        Regex::new(r"(?i)(what|which)\s*(network|wifi|wi-fi|internet).*(connect|connected|connecting|status)").unwrap(),
                        Regex::new(r"(?i)how\s*(much|many)\s*(memory|ram|storage|space)").unwrap(),
                        Regex::new(r"(?i)what('s|s| is)\s*(the|my)?\s*(cpu|memory|ram|storage)").unwrap(),
                    ]
                })
            }
            Self::VolumeControl => {
                static RE: OnceLock<Vec<Regex>> = OnceLock::new();
                RE.get_or_init(|| {
                    vec![
                        Regex::new(r"(?i)(volume|sound)\s*(up|down|louder|quieter|higher|lower)").unwrap(),
                        Regex::new(r"(?i)(turn|set)\s*(up|down)?\s*(the)?\s*volume").unwrap(),
                        Regex::new(r"(?i)(mute|unmute)(\s+(the|my|the\s+)?)?(\s+(sound|volume|audio|music))?").unwrap(),
                        Regex::new(r"(?i)(increase|decrease|raise|lower)\s*(the)?\s*volume").unwrap(),
                        Regex::new(r"(?i)adjust\s*(the)?\s*volume(\s*(to|at)\s*\d+)?").unwrap(),
                        Regex::new(r"(?i)volume\s*(to|at)?\s*(\d+)").unwrap(),
                        Regex::new(r"(?i)(set|change)\s*(the)?\s*volume\s*(to|at)?\s*(\d+)").unwrap(),
                        Regex::new(r"(?i)make\s*it\s*(louder|quieter)").unwrap(),
                    ]
                })
            }
            Self::MusicControl => {
                static RE: OnceLock<Vec<Regex>> = OnceLock::new();
                RE.get_or_init(|| {
                    vec![
                        Regex::new(r"(?i)(play|pause|stop)\s*(music|song|track|spotify)?").unwrap(),
                        Regex::new(r"(?i)(next|previous|skip)\s*(track|song)?").unwrap(),
                        Regex::new(r"(?i)(spotify|music)\s*(play|pause|next|previous|skip)").unwrap(),
                        Regex::new(r"(?i)what('s|s| is)?\s*(music|song|track)?\s*(is\s*)?playing").unwrap(),
                        Regex::new(r"(?i)current\s*(track|song)").unwrap(),
                        Regex::new(r"(?i)resume\s*(music|playback|spotify)?").unwrap(),
                        Regex::new(r"(?i)(search|find)\s*(on\s*)?(spotify|music)").unwrap(),
                        Regex::new(r"(?i)(add\s*to\s*queue|queue\s*up)").unwrap(),
                    ]
                })
            }
            Self::NetworkToggle => {
                static RE: OnceLock<Vec<Regex>> = OnceLock::new();
                RE.get_or_init(|| {
                    vec![
                        Regex::new(r"(?i)(turn|toggle|switch)\s*(on|off)\s*(the)?\s*(wifi|wi-fi|bluetooth|ethernet)").unwrap(),
                        Regex::new(r"(?i)(enable|disable)\s*(the)?\s*(wifi|wi-fi|bluetooth|ethernet)").unwrap(),
                        Regex::new(r"(?i)(wifi|wi-fi|bluetooth|ethernet)\s*(on|off)").unwrap(),
                        Regex::new(r"(?i)(connect|disconnect)\s*(to)?\s*(wifi|wi-fi|bluetooth|ethernet)").unwrap(),
                    ]
                })
            }
            Self::FileOrganization => {
                static RE: OnceLock<Vec<Regex>> = OnceLock::new();
                RE.get_or_init(|| {
                    vec![
                        Regex::new(r"(?i)(organize|clean|sort)\s*(my|the)?\s*(downloads|desktop|documents|folder|files)").unwrap(),
                        Regex::new(r"(?i)(organize|sort)\s*(files|folder)").unwrap(),
                        Regex::new(r"(?i)(clean up|tidy)\s*(downloads|desktop|documents|folder)?").unwrap(),
                        Regex::new(r"(?i)(arrange|group)\s*(files)\s*(by)?\s*(type|extension|date)").unwrap(),
                    ]
                })
            }
            Self::BluetoothControl => {
                static RE: OnceLock<Vec<Regex>> = OnceLock::new();
                RE.get_or_init(|| {
                    vec![
                        Regex::new(r"(?i)(list|show|see|find)\s*(my)?\s*bluetooth\s*(devices?)?").unwrap(),
                        Regex::new(r"(?i)(connect|disconnect)\s*(to|from)?\s*(the)?\s*(bluetooth\s*)?device").unwrap(),
                    ]
                })
            }
            Self::PathResolve => {
                static RE: OnceLock<Vec<Regex>> = OnceLock::new();
                RE.get_or_init(|| {
                    vec![
                        Regex::new(r"(?i)resolve\s*(the)?\s*path\s*(for|to)?").unwrap(),
                        Regex::new(r"(?i)where\s*is\s*(my)?\s*(downloads|documents|desktop|home|project)\s*(folder)?").unwrap(),
                    ]
                })
            }
            _ => &[],
        }
    }

    pub fn extractors(&self) -> &'static [(&'static str, ParamExtractor)] {
        match self {
            Self::VolumeControl => {
                static EX: OnceLock<Vec<(&'static str, ParamExtractor)>> = OnceLock::new();
                EX.get_or_init(|| {
                    vec![
                        (
                            "direction",
                            ParamExtractor::List(vec![
                                (Regex::new(r"(?i)\bunmute\b").unwrap(), "unmute"),
                                (Regex::new(r"(?i)\bmute\b").unwrap(), "mute"),
                                (Regex::new(r"(?i)(up|louder|higher|increase|raise)").unwrap(), "up"),
                                (Regex::new(r"(?i)(down|quieter|lower|decrease)").unwrap(), "down"),
                            ]),
                        ),
                        ("level", ParamExtractor::Regex(Regex::new(r"(?i)(\d+)\s*(%|percent)?").unwrap())),
                    ]
                })
            }
            Self::MusicControl => {
                static EX: OnceLock<Vec<(&'static str, ParamExtractor)>> = OnceLock::new();
                EX.get_or_init(|| {
                    vec![(
                        "action",
                        ParamExtractor::List(vec![
                            (Regex::new(r"(?i)\b(play|resume)\b").unwrap(), "play"),
                            (Regex::new(r"(?i)\b(pause|stop)\b").unwrap(), "pause"),
                            (Regex::new(r"(?i)\b(next|skip)\b").unwrap(), "next"),
                            (Regex::new(r"(?i)\bprevious\b").unwrap(), "previous"),
                            (Regex::new(r"(?i)(what('s|s| is)?\s*(music|song|track)?\s*(is\s*)?playing|current\s*(track|song))").unwrap(), "current"),
                            (Regex::new(r"(?i)\b(search|find)\b").unwrap(), "search"),
                            (Regex::new(r"(?i)\b(queue|add)\b").unwrap(), "queue"),
                        ]),
                    )]
                })
            }
            Self::NetworkToggle => {
                static EX: OnceLock<Vec<(&'static str, ParamExtractor)>> = OnceLock::new();
                EX.get_or_init(|| {
                    vec![
                        (
                            "device",
                            ParamExtractor::List(vec![
                                (Regex::new(r"(?i)\b(wifi|wi-fi)\b").unwrap(), "wifi"),
                                (Regex::new(r"(?i)\bbluetooth\b").unwrap(), "bluetooth"),
                                (Regex::new(r"(?i)\bethernet\b").unwrap(), "ethernet"),
                            ]),
                        ),
                        (
                            "state",
                            ParamExtractor::List(vec![
                                (Regex::new(r"(?i)\b(on|enable|connect)\b").unwrap(), "on"),
                                (Regex::new(r"(?i)\b(off|disable|disconnect)\b").unwrap(), "off"),
                            ]),
                        ),
                    ]
                })
            }
            Self::FileOrganization => {
                static EX: OnceLock<Vec<(&'static str, ParamExtractor)>> = OnceLock::new();
                EX.get_or_init(|| {
                    vec![
                        (
                            "target_folder",
                            ParamExtractor::List(vec![
                                (Regex::new(r"(?i)\bdownloads?\b").unwrap(), "downloads"),
                                (Regex::new(r"(?i)\bdesktop\b").unwrap(), "desktop"),
                                (Regex::new(r"(?i)\bdocuments?\b").unwrap(), "documents"),
                            ]),
                        ),
                        (
                            "strategy",
                            ParamExtractor::List(vec![
                                (Regex::new(r"(?i)\b(date|month|year)\b").unwrap(), "date"),
                                (Regex::new(r"(?i)\b(type|category)\b").unwrap(), "type"),
                                (Regex::new(r"(?i)\b(extension|ext)\b").unwrap(), "extension"),
                            ]),
                        ),
                        (
                            "dry_run",
                            ParamExtractor::List(vec![
                                (Regex::new(r"(?i)\b(preview|dry\s*run|simulate)\b").unwrap(), "true"),
                                (Regex::new(r"(?i)\b(now|apply|execute|do it)\b").unwrap(), "false"),
                            ]),
                        ),
                    ]
                })
            }
            Self::DirectoryList => {
                static EX: OnceLock<Vec<(&'static str, ParamExtractor)>> = OnceLock::new();
                EX.get_or_init(|| {
                    vec![(
                        "include_hidden",
                        ParamExtractor::List(vec![(Regex::new(r"(?i)\b(hidden|all files)\b").unwrap(), "true")]),
                    )]
                })
            }
            Self::BluetoothControl => {
                static EX: OnceLock<Vec<(&'static str, ParamExtractor)>> = OnceLock::new();
                EX.get_or_init(|| {
                    vec![(
                        "action",
                        ParamExtractor::List(vec![
                            (Regex::new(r"(?i)\b(list|show|find|see)\b").unwrap(), "list"),
                            (Regex::new(r"(?i)\bconnect\b").unwrap(), "connect"),
                            (Regex::new(r"(?i)\bdisconnect\b").unwrap(), "disconnect"),
                        ]),
                    )]
                })
            }
            Self::PathResolve => {
                static EX: OnceLock<Vec<(&'static str, ParamExtractor)>> = OnceLock::new();
                EX.get_or_init(|| {
                    vec![(
                        "name",
                        ParamExtractor::List(vec![
                            (Regex::new(r"(?i)\bdownloads?\b").unwrap(), "downloads"),
                            (Regex::new(r"(?i)\bdocuments?\b").unwrap(), "documents"),
                            (Regex::new(r"(?i)\bdesktop\b").unwrap(), "desktop"),
                            (Regex::new(r"(?i)\bhome\b").unwrap(), "home"),
                            (Regex::new(r"(?i)\bproject\b").unwrap(), "project"),
                        ]),
                    )]
                })
            }
            _ => &[],
        }
    }
}
