//! `gx config` / `gx config set <key> <value>` — show or mutate config.
//! Ported from `src/commands/config.ts`.

use std::path::Path;

use serde::Serialize;

use crate::config::{effective_project_dir, get_agent, load_config, save_config};
use crate::errors::{GxError, GxResult};
use crate::types::{Config, Structure};

/// The order matches the TS `Object.keys(config)` order, used both for the
/// "Unknown config key" error and the `config set` type dispatch.
const CONFIG_KEYS: &str =
    "projectDir, defaultHost, defaultOwner, structure, shallow, similarityThreshold, editor";

/// Serialisable view of the config for `gx config`. When an agent is active,
/// `agent` and `effectiveProjectDir` are appended after the config fields —
/// `#[serde(flatten)]` preserves that insertion order (serde_json writes
/// struct fields in declaration order, unlike `Value::Object`).
#[derive(Serialize)]
struct ConfigView<'a> {
    #[serde(flatten)]
    config: &'a Config,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    #[serde(
        rename = "effectiveProjectDir",
        skip_serializing_if = "Option::is_none"
    )]
    effective_project_dir: Option<String>,
}

pub fn show_config(config_path: &Path) -> GxResult<()> {
    let config = load_config(config_path);
    let agent = get_agent()?;

    let view = if agent.is_some() {
        let eff = effective_project_dir(&config)?
            .to_string_lossy()
            .into_owned();
        ConfigView {
            config: &config,
            agent,
            effective_project_dir: Some(eff),
        }
    } else {
        ConfigView {
            config: &config,
            agent: None,
            effective_project_dir: None,
        }
    };

    let json = serde_json::to_string_pretty(&view)
        .map_err(|e| GxError::Other(format!("serialise config: {e}")))?;
    println!("{json}");
    Ok(())
}

pub fn set_config(config_path: &Path, key: &str, value: &str) -> GxResult<()> {
    let mut config = load_config(config_path);

    match key {
        "projectDir" => config.project_dir = value.to_string(),
        "defaultHost" => config.default_host = value.to_string(),
        "defaultOwner" => config.default_owner = value.to_string(),
        "editor" => config.editor = value.to_string(),
        "structure" => {
            config.structure = Structure::from_str_opt(value).ok_or_else(|| {
                GxError::command(format!(
                    "Invalid structure value: {value}\nValid values: flat, owner, host"
                ))
            })?;
        }
        "shallow" => config.shallow = value == "true",
        "similarityThreshold" => {
            config.similarity_threshold = parse_js_number(value)
                .ok_or_else(|| GxError::command(format!("Value for '{key}' must be a number")))?;
        }
        _ => {
            return Err(GxError::command(format!(
                "Unknown config key: {key}\nValid keys: {CONFIG_KEYS}"
            )));
        }
    }

    save_config(config_path, &config)?;
    eprintln!("Set {key} = {value}");
    Ok(())
}

/// Mirror JS `Number(value)` for the cases reachable here: a trimmed empty
/// string is `0`, otherwise parse as f64 and reject NaN. (The dispatcher only
/// calls `set` when the value arg is non-empty, so the empty case is for
/// whitespace-only input parity.)
fn parse_js_number(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Some(0.0);
    }
    trimmed.parse::<f64>().ok().filter(|n| !n.is_nan())
}
