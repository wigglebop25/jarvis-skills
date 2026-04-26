use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntentType {
    DirectoryList,
    SystemInfo,
    VolumeControl,
    MusicControl,
    NetworkToggle,
    FileOrganization,
    Unknown,
    GeneralQuery,
}

impl IntentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DirectoryList => "DIRECTORY_LIST",
            Self::SystemInfo => "SYSTEM_INFO",
            Self::VolumeControl => "VOLUME_CONTROL",
            Self::MusicControl => "MUSIC_CONTROL",
            Self::NetworkToggle => "NETWORK_TOGGLE",
            Self::FileOrganization => "FILE_ORGANIZATION",
            Self::Unknown => "UNKNOWN",
            Self::GeneralQuery => "GENERAL_QUERY",
        }
    }

    pub fn tool_name(&self) -> Option<&'static str> {
        match self {
            Self::DirectoryList => Some("list_directory"),
            Self::SystemInfo => Some("get_system_info"),
            Self::VolumeControl => Some("control_volume"),
            Self::MusicControl => Some("control_spotify"),
            Self::NetworkToggle => Some("toggle_network"),
            Self::FileOrganization => Some("organize_folder"),
            _ => None,
        }
    }

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
                        Regex::new(r"(?i)what('s|s| is)\s*(playing|this song)").unwrap(),
                        Regex::new(r"(?i)current\s*(track|song)").unwrap(),
                        Regex::new(r"(?i)resume\s*(music|playback|spotify)?").unwrap(),
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
                            (Regex::new(r"(?i)(what('s| is)\s*playing|current\s*(track|song))").unwrap(), "current"),
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
            _ => &[],
        }
    }
}

pub enum ParamExtractor {
    List(Vec<(Regex, &'static str)>),
    Regex(Regex),
}

pub struct RouteDecision {
    pub intent: String,
    pub confidence: f64,
    pub tool_name: Option<String>,
    pub arguments: Map<String, Value>,
    pub should_execute: bool,
}

pub fn route_intent(text: &str) -> RouteDecision {
    let text_lower = text.trim().to_lowercase();
    if text_lower.is_empty() {
        return RouteDecision {
            intent: IntentType::Unknown.as_str().to_string(),
            confidence: 0.0,
            tool_name: None,
            arguments: Map::new(),
            should_execute: false,
        };
    }

    let mut best_match: Option<(IntentType, f64)> = None;
    let intent_types = vec![
        IntentType::DirectoryList,
        IntentType::SystemInfo,
        IntentType::VolumeControl,
        IntentType::MusicControl,
        IntentType::NetworkToggle,
        IntentType::FileOrganization,
    ];

    for intent_type in intent_types {
        for pattern in intent_type.patterns() {
            if pattern.is_match(&text_lower) {
                let mut confidence = 0.85;
                if text_lower.split_whitespace().count() <= 5 {
                    confidence += 0.1;
                }
                
                let cur = best_match.as_ref().map(|x| x.1).unwrap_or(0.0);
                if confidence > cur {
                    best_match = Some((intent_type.clone(), confidence.min(1.0)));
                }
                break;
            }
        }
    }

    let (intent_type, confidence) = if let Some(m) = best_match {
        m
    } else {
        return RouteDecision {
            intent: IntentType::GeneralQuery.as_str().to_string(),
            confidence: 0.6,
            tool_name: None,
            arguments: Map::new(),
            should_execute: false,
        };
    };

    let mut raw_params: Map<String, Value> = Map::new();
    for (param_name, extractor) in intent_type.extractors() {
        match extractor {
            ParamExtractor::List(list) => {
                for (pattern, val) in list {
                    if pattern.is_match(&text_lower) {
                        raw_params.insert(param_name.to_string(), Value::String(val.to_string()));
                        break;
                    }
                }
            }
            ParamExtractor::Regex(re) => {
                if let Some(caps) = re.captures(&text_lower) {
                    if let Some(m) = caps.get(1) {
                        raw_params.insert(param_name.to_string(), Value::String(m.as_str().to_string()));
                    }
                }
            }
        }
    }

    let mut final_args = raw_params.clone();
    
    // Map intent params to tool args
    match intent_type {
        IntentType::VolumeControl => {
            if let Some(direction) = raw_params.get("direction").and_then(|v| v.as_str()) {
                final_args.insert("action".to_string(), Value::String(direction.to_string()));
                final_args.remove("direction");
            } else if raw_params.contains_key("level") {
                final_args.insert("action".to_string(), Value::String("set".to_string()));
            } else {
                final_args.insert("action".to_string(), Value::String("get".to_string()));
            }

            if let Some(level) = raw_params.get("level").and_then(|v| v.as_str()) {
                if let Ok(num) = level.parse::<i32>() {
                    final_args.insert("level".to_string(), Value::Number(num.into()));
                } else {
                    final_args.remove("level");
                }
            }
        }
        IntentType::NetworkToggle => {
            if let Some(device) = raw_params.get("device").and_then(|v| v.as_str()) {
                final_args.insert("interface".to_string(), Value::String(device.to_string()));
                final_args.remove("device");
            }
            if let Some(state) = raw_params.get("state").and_then(|v| v.as_str()) {
                final_args.insert("enable".to_string(), Value::Bool(state == "on"));
                final_args.remove("state");
            }
        }
        IntentType::SystemInfo => {
            let mut include = Vec::new();
            if Regex::new(r"(?i)\b(cpu|processor)\b").unwrap().is_match(&text_lower) {
                include.push(Value::String("cpu".to_string()));
            }
            if Regex::new(r"(?i)\b(ram|memory)\b").unwrap().is_match(&text_lower) {
                include.push(Value::String("ram".to_string()));
            }
            if Regex::new(r"(?i)\b(storage|disk|drive|ssd|space)\b").unwrap().is_match(&text_lower) {
                include.push(Value::String("storage".to_string()));
            }
            if Regex::new(r"(?i)\b(network|internet|wifi|wi-fi|connected|connection)\b").unwrap().is_match(&text_lower) {
                include.push(Value::String("network".to_string()));
            }
            if !include.is_empty() {
                // Deduplicate implicitly done via simple matching
                final_args.insert("include".to_string(), Value::Array(include));
            }
        }
        IntentType::DirectoryList => {
            let mut path_str = None;

            let drive_root_fn = |letter: &str, require_exists: bool| -> Option<String> {
                let candidate = format!("{}:\\", letter.to_ascii_uppercase());
                if cfg!(windows) {
                    if !require_exists || Path::new(&candidate).exists() {
                        Some(candidate)
                    } else {
                        None
                    }
                } else {
                    Some(candidate)
                }
            };

            // Explicit drive formats
            for pattern in &[r"(?i)\b([a-z])\s*:", r"(?i)\bdrive\s*([a-z])\b", r"(?i)\b([a-z])\s*drive\b"] {
                if let Some(caps) = Regex::new(pattern).unwrap().captures(&text_lower) {
                    if let Some(m) = caps.get(1) {
                        if let Some(drive) = drive_root_fn(m.as_str(), false) {
                            path_str = Some(drive);
                            break;
                        }
                    }
                }
            }

            if path_str.is_none() {
                if let Some(caps) = Regex::new(r"(?i)\b(?:inside|in)\s*(?:the\s*)?([a-z])\b").unwrap().captures(&text_lower) {
                    if let Some(m) = caps.get(1) {
                        if let Some(drive) = drive_root_fn(m.as_str(), true) {
                            path_str = Some(drive);
                        }
                    }
                }
            }

            if path_str.is_none() {
                if let Some(caps) = Regex::new(r"(?i)\bi\s+the\s+([a-z])\b").unwrap().captures(&text_lower) {
                    if let Some(m) = caps.get(1) {
                        if let Some(drive) = drive_root_fn(m.as_str(), true) {
                            path_str = Some(drive);
                        }
                    }
                }
            }

            if path_str.is_none() {
                if let Some(home) = dirs::home_dir() {
                    path_str = Some(home.to_string_lossy().to_string());
                } else {
                    path_str = Some(".".to_string());
                }
            }

            final_args.insert("path".to_string(), Value::String(path_str.unwrap()));
            
            let is_hidden = raw_params.get("include_hidden").and_then(|v| v.as_str()) == Some("true");
            final_args.insert("include_hidden".to_string(), Value::Bool(is_hidden));
            final_args.insert("max_entries".to_string(), Value::Number(200.into()));

            let has_folders = Regex::new(r"(?i)\bfolders?\b").unwrap().is_match(&text_lower);
            let has_files = Regex::new(r"(?i)\bfiles?\b").unwrap().is_match(&text_lower);

            final_args.insert("directories_only".to_string(), Value::Bool(has_folders && !has_files));
            final_args.insert("files_only".to_string(), Value::Bool(has_files && !has_folders));
        }
        IntentType::FileOrganization => {
            let folder_alias = raw_params.get("target_folder").and_then(|v| v.as_str()).unwrap_or("downloads");
            let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            
            let path_val = match folder_alias {
                "downloads" => home.join("Downloads"),
                "desktop" => home.join("Desktop"),
                "documents" => home.join("Documents"),
                _ => home.join("Downloads"),
            };
            final_args.insert("path".to_string(), Value::String(path_val.to_string_lossy().to_string()));
            
            if !final_args.contains_key("strategy") {
                final_args.insert("strategy".to_string(), Value::String("extension".to_string()));
            }

            let is_dry = raw_params.get("dry_run").and_then(|v| v.as_str()) != Some("false");
            final_args.insert("dry_run".to_string(), Value::Bool(is_dry));
        }
        _ => {}
    }

    let should_execute = confidence >= 0.85 && intent_type.tool_name().is_some();

    RouteDecision {
        intent: intent_type.as_str().to_string(),
        confidence,
        tool_name: intent_type.tool_name().map(|s| s.to_string()),
        arguments: final_args,
        should_execute,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_volume_control_intent() {
        let dec = route_intent("Volume up");
        assert_eq!(dec.intent, "VOLUME_CONTROL");
        assert!(dec.should_execute);
        assert_eq!(dec.tool_name.as_deref(), Some("control_volume"));
        assert_eq!(dec.arguments.get("action"), Some(&Value::String("up".to_string())));
    }

    #[test]
    fn test_system_info_intent() {
        let dec = route_intent("What is my CPU usage?");
        assert_eq!(dec.intent, "SYSTEM_INFO");
        assert!(dec.should_execute);
        assert_eq!(dec.tool_name.as_deref(), Some("get_system_info"));
        
        let arr = dec.arguments.get("include").unwrap().as_array().unwrap();
        assert_eq!(arr[0], Value::String("cpu".to_string()));
    }

    #[test]
    fn test_directory_list_intent() {
        let dec = route_intent("List folders in D drive");
        assert_eq!(dec.intent, "DIRECTORY_LIST");
        assert!(dec.should_execute);
        assert_eq!(dec.tool_name.as_deref(), Some("list_directory"));
    }
}
