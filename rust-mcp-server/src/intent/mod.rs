use serde_json::{Map, Value};

pub mod patterns;
pub mod mappings;

pub use patterns::ParamExtractor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntentType {
    DirectoryList,
    SystemInfo,
    VolumeControl,
    MusicControl,
    NetworkToggle,
    FileOrganization,
    BluetoothControl,
    PathResolve,
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
            Self::BluetoothControl => "BLUETOOTH_CONTROL",
            Self::PathResolve => "PATH_RESOLVE",
            Self::Unknown => "UNKNOWN",
            Self::GeneralQuery => "GENERAL_QUERY",
        }
    }

    pub fn tool_name(&self) -> Option<&'static str> {
        match self {
            Self::DirectoryList => Some("list_directory"),
            Self::SystemInfo => Some("get_system_info"),
            Self::VolumeControl => Some("control_volume"),
            Self::MusicControl => Some("playMusic"),
            Self::NetworkToggle => Some("toggle_network"),
            Self::FileOrganization => Some("organize_folder"),
            Self::BluetoothControl => Some("control_bluetooth_device"),
            Self::PathResolve => Some("resolve_path"),
            _ => None,
        }
    }
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
        IntentType::BluetoothControl,
        IntentType::PathResolve,
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

    mappings::map_params_to_args(&intent_type, &raw_params, &text_lower, confidence)
}
