use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Structure {
    Flat,
    Owner,
    Host,
}

impl Structure {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "flat" => Some(Structure::Flat),
            "owner" => Some(Structure::Owner),
            "host" => Some(Structure::Host),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Structure::Flat => "flat",
            Structure::Owner => "owner",
            Structure::Host => "host",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub project_dir: String,
    pub default_host: String,
    pub default_owner: String,
    pub structure: Structure,
    pub shallow: bool,
    pub similarity_threshold: f64,
    pub editor: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_dir: "~/Projects/src".into(),
            default_host: "github.com".into(),
            default_owner: String::new(),
            structure: Structure::Owner,
            shallow: false,
            similarity_threshold: 0.7,
            editor: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedRepo {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub original_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexEntry {
    pub path: String,
    pub url: String,
    pub cloned_at: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_visited: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Index {
    pub projects: IndexMap<String, IndexEntry>,
}
