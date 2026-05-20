use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

use serde_json::Value;

use crate::errors::{GxError, GxResult};
use crate::types::{Config, Structure};

pub fn get_config_path() -> PathBuf {
    home_dir().join(".config/gx/config.json")
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

/// Expand `~/...` and bare `~` against the current home directory. Other
/// inputs are returned verbatim, matching `src/lib/config.ts::expandTilde`.
pub fn expand_tilde(p: &str) -> PathBuf {
    if p == "~" {
        return home_dir();
    }
    if let Some(rest) = p.strip_prefix("~/") {
        return home_dir().join(rest);
    }
    PathBuf::from(p)
}

/// Pure validator for the `GX_AGENT` env value. Returns `Ok(None)` for
/// unset/empty/whitespace input, `Ok(Some(name))` for valid input, and an
/// error otherwise. Matches `getAgent` regex: `^[a-z0-9]([a-z0-9-]*[a-z0-9])?$`.
pub fn validate_agent(raw: Option<&str>) -> GxResult<Option<String>> {
    let Some(raw) = raw else { return Ok(None) };
    let trimmed = raw.trim().to_lowercase();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if !is_valid_agent_name(&trimmed) {
        return Err(GxError::Other(format!(
            "Invalid GX_AGENT: \"{trimmed}\" — use lowercase alphanumeric and hyphens"
        )));
    }
    Ok(Some(trimmed))
}

fn is_valid_agent_name(s: &str) -> bool {
    // ^[a-z0-9]([a-z0-9-]*[a-z0-9])?$
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    if !is_alnum(bytes[0]) {
        return false;
    }
    if bytes.len() == 1 {
        return true;
    }
    if !is_alnum(*bytes.last().unwrap()) {
        return false;
    }
    bytes[1..bytes.len() - 1]
        .iter()
        .all(|&b| is_alnum(b) || b == b'-')
}

fn is_alnum(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit()
}

/// Read `GX_AGENT` from the environment with validation. Tests should prefer
/// [`validate_agent`] so they don't depend on global env state.
pub fn get_agent() -> GxResult<Option<String>> {
    validate_agent(std::env::var("GX_AGENT").ok().as_deref())
}

/// Pure project-dir resolution: same as [`effective_project_dir`] but takes
/// the agent string explicitly.
pub fn effective_project_dir_for(config: &Config, agent: Option<&str>) -> PathBuf {
    let base = expand_tilde(&config.project_dir);
    match agent {
        Some(a) if !a.is_empty() => base.join(format!(".{a}")),
        _ => base,
    }
}

/// Env-aware project-dir resolution (matches the TS surface).
pub fn effective_project_dir(config: &Config) -> GxResult<PathBuf> {
    let agent = get_agent()?;
    Ok(effective_project_dir_for(config, agent.as_deref()))
}

/// Manual TS-parity validation: accept fields that pass type/value checks,
/// fall back to `Config::default()` for the rest. Matches the field-by-field
/// behaviour of `src/lib/config.ts::validateConfig`.
pub fn validate_config_value(raw: &Value) -> Config {
    let mut out = Config::default();
    let Value::Object(obj) = raw else {
        return out;
    };

    if let Some(Value::String(s)) = obj.get("projectDir") {
        if !s.is_empty() {
            out.project_dir = s.clone();
        }
    }
    if let Some(Value::String(s)) = obj.get("defaultHost") {
        if !s.is_empty() {
            out.default_host = s.clone();
        }
    }
    if let Some(Value::String(s)) = obj.get("defaultOwner") {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            out.default_owner = trimmed.to_string();
        }
    }
    if let Some(Value::String(s)) = obj.get("structure") {
        if let Some(st) = Structure::from_str_opt(s) {
            out.structure = st;
        }
    }
    if let Some(Value::Bool(b)) = obj.get("shallow") {
        out.shallow = *b;
    }
    if let Some(Value::Number(n)) = obj.get("similarityThreshold") {
        if let Some(f) = n.as_f64() {
            if (0.0..=1.0).contains(&f) {
                out.similarity_threshold = f;
            }
        }
    }
    if let Some(Value::String(s)) = obj.get("editor") {
        out.editor = s.clone();
    }

    out
}

pub fn load_config(path: &Path) -> Config {
    let Ok(raw) = fs::read_to_string(path) else {
        return Config::default();
    };
    let Ok(value) = serde_json::from_str::<Value>(&raw) else {
        return Config::default();
    };
    validate_config_value(&value)
}

/// Atomically serialise `config` as JSON to `path` (parent dirs created as
/// needed). Output is 2-space-indented with a trailing newline, matching
/// `JSON.stringify(.., null, 2) + "\n"` in the TS code.
pub fn save_config(path: &Path, config: &Config) -> GxResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| GxError::Other(format!("create {}: {e}", parent.display())))?;
    }
    let mut text = serde_json::to_string_pretty(config)
        .map_err(|e| GxError::Other(format!("serialise config: {e}")))?;
    text.push('\n');
    atomic_write(path, text.as_bytes())
}

pub(crate) fn atomic_write(path: &Path, contents: &[u8]) -> GxResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| GxError::Other(format!("path has no parent: {}", path.display())))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| GxError::Other(format!("path has no file name: {}", path.display())))?
        .to_string_lossy()
        .into_owned();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = process::id();
    let tmp = parent.join(format!("{file_name}.{pid}.{nanos}.tmp"));
    {
        let mut f = fs::File::create(&tmp)
            .map_err(|e| GxError::Other(format!("create {}: {e}", tmp.display())))?;
        f.write_all(contents)
            .map_err(|e| GxError::Other(format!("write {}: {e}", tmp.display())))?;
    }
    fs::rename(&tmp, path)
        .map_err(|e| GxError::Other(format!("rename {} -> {}: {e}", tmp.display(), path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    // --- expand_tilde -----------------------------------------------------

    #[test]
    fn expand_tilde_with_slash() {
        let home = home_dir();
        assert_eq!(expand_tilde("~/Projects/src"), home.join("Projects/src"));
    }

    #[test]
    fn expand_tilde_bare() {
        assert_eq!(expand_tilde("~"), home_dir());
    }

    #[test]
    fn expand_tilde_absolute_unchanged() {
        assert_eq!(expand_tilde("/usr/local/bin"), PathBuf::from("/usr/local/bin"));
    }

    #[test]
    fn expand_tilde_relative_unchanged() {
        assert_eq!(expand_tilde("relative/path"), PathBuf::from("relative/path"));
    }

    // --- validate_agent ---------------------------------------------------

    #[test]
    fn agent_unset_is_none() {
        assert_eq!(validate_agent(None).unwrap(), None);
    }

    #[test]
    fn agent_lowercased() {
        assert_eq!(validate_agent(Some("Morgan")).unwrap(), Some("morgan".into()));
    }

    #[test]
    fn agent_trimmed() {
        assert_eq!(validate_agent(Some("  morgan  ")).unwrap(), Some("morgan".into()));
    }

    #[test]
    fn agent_empty_is_none() {
        assert_eq!(validate_agent(Some("")).unwrap(), None);
    }

    #[test]
    fn agent_whitespace_is_none() {
        assert_eq!(validate_agent(Some("   ")).unwrap(), None);
    }

    #[test]
    fn agent_accepts_hyphen_in_middle() {
        assert_eq!(
            validate_agent(Some("agent-007")).unwrap(),
            Some("agent-007".into())
        );
    }

    #[test]
    fn agent_rejects_invalid_chars() {
        let err = validate_agent(Some("bad_agent!")).unwrap_err();
        assert!(err.to_string().contains("Invalid GX_AGENT"));
    }

    #[test]
    fn agent_rejects_leading_hyphen() {
        let err = validate_agent(Some("-morgan")).unwrap_err();
        assert!(err.to_string().contains("Invalid GX_AGENT"));
    }

    #[test]
    fn agent_rejects_trailing_hyphen() {
        let err = validate_agent(Some("morgan-")).unwrap_err();
        assert!(err.to_string().contains("Invalid GX_AGENT"));
    }

    // --- effective_project_dir_for ---------------------------------------

    #[test]
    fn effective_no_agent() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            ..Config::default()
        };
        assert_eq!(
            effective_project_dir_for(&cfg, None),
            PathBuf::from("/home/user/src")
        );
    }

    #[test]
    fn effective_with_agent_appends_dotdir() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            ..Config::default()
        };
        assert_eq!(
            effective_project_dir_for(&cfg, Some("morgan")),
            PathBuf::from("/home/user/src/.morgan")
        );
    }

    #[test]
    fn effective_expands_tilde() {
        let cfg = Config {
            project_dir: "~/src".into(),
            ..Config::default()
        };
        let result = effective_project_dir_for(&cfg, None);
        let result_s = result.to_string_lossy();
        assert!(!result_s.contains('~'));
        assert!(result_s.ends_with("/src"));
    }

    #[test]
    fn effective_expands_tilde_with_agent() {
        let cfg = Config {
            project_dir: "~/src".into(),
            ..Config::default()
        };
        let result = effective_project_dir_for(&cfg, Some("morgan"));
        let result_s = result.to_string_lossy();
        assert!(!result_s.contains('~'));
        assert!(result_s.ends_with("/src/.morgan"));
    }

    // --- get_config_path --------------------------------------------------

    #[test]
    fn config_path_under_config_gx() {
        let p = get_config_path();
        let s = p.to_string_lossy();
        assert!(s.contains(".config/gx/config.json"), "got {s}");
    }

    // --- load_config / save_config / validation -------------------------

    #[test]
    fn load_returns_defaults_when_missing() {
        let tmp = TempDir::new().unwrap();
        let cfg = load_config(&tmp.path().join("config.json"));
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn save_then_load_roundtrips() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        let custom = Config {
            structure: Structure::Host,
            ..Config::default()
        };
        save_config(&path, &custom).unwrap();
        let loaded = load_config(&path);
        assert_eq!(loaded.structure, Structure::Host);
    }

    #[test]
    fn load_merges_partial_with_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        fs::write(&path, r#"{"shallow": true}"#).unwrap();
        let cfg = load_config(&path);
        assert!(cfg.shallow);
        assert_eq!(cfg.default_host, "github.com");
    }

    #[test]
    fn load_invalid_json_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        fs::write(&path, "{ not valid json }").unwrap();
        let cfg = load_config(&path);
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn load_invalid_structure_falls_back() {
        let cfg = validate_config_value(&json!({"structure": "invalid"}));
        assert_eq!(cfg.structure, Structure::Owner);
    }

    #[test]
    fn load_non_number_threshold_falls_back() {
        let cfg = validate_config_value(&json!({"similarityThreshold": "not-a-number"}));
        assert_eq!(cfg.similarity_threshold, 0.7);
    }

    #[test]
    fn load_out_of_range_threshold_falls_back() {
        let cfg = validate_config_value(&json!({"similarityThreshold": 5.0}));
        assert_eq!(cfg.similarity_threshold, 0.7);
    }

    #[test]
    fn load_non_bool_shallow_falls_back() {
        let cfg = validate_config_value(&json!({"shallow": "yes"}));
        assert!(!cfg.shallow);
    }

    #[test]
    fn load_empty_project_dir_falls_back() {
        let cfg = validate_config_value(&json!({"projectDir": ""}));
        assert_eq!(cfg.project_dir, "~/Projects/src");
    }

    #[test]
    fn load_whitespace_default_owner_becomes_empty() {
        let cfg = validate_config_value(&json!({"defaultOwner": "  "}));
        assert_eq!(cfg.default_owner, "");
    }

    #[test]
    fn load_trims_surrounding_default_owner() {
        let cfg = validate_config_value(&json!({"defaultOwner": " eddacraft "}));
        assert_eq!(cfg.default_owner, "eddacraft");
    }

    #[test]
    fn load_accepts_full_config() {
        let value = json!({
            "projectDir": "/custom/path",
            "defaultHost": "gitlab.com",
            "defaultOwner": "",
            "structure": "host",
            "shallow": true,
            "similarityThreshold": 0.8,
            "editor": "vim",
        });
        let cfg = validate_config_value(&value);
        assert_eq!(cfg.project_dir, "/custom/path");
        assert_eq!(cfg.default_host, "gitlab.com");
        assert_eq!(cfg.default_owner, "");
        assert_eq!(cfg.structure, Structure::Host);
        assert!(cfg.shallow);
        assert!((cfg.similarity_threshold - 0.8).abs() < f64::EPSILON);
        assert_eq!(cfg.editor, "vim");
    }

    #[test]
    fn save_writes_two_space_indent_with_trailing_newline() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        save_config(&path, &Config::default()).unwrap();
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.ends_with('\n'), "no trailing newline");
        // 2-space indent: every non-empty line starts with even number of spaces.
        for line in written.lines().filter(|l| l.starts_with(' ')) {
            let leading = line.chars().take_while(|c| *c == ' ').count();
            assert!(leading % 2 == 0, "non-2-space indent in line: {line:?}");
        }
        assert!(written.contains("\"projectDir\""));
    }
}
