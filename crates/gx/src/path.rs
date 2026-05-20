use std::path::PathBuf;

use crate::config::effective_project_dir_for;
use crate::types::{Config, ParsedRepo, Structure};

/// Pure path mapping: takes an explicit agent (instead of reading `GX_AGENT`)
/// so tests don't have to mutate global env state.
pub fn to_path_for(parsed: &ParsedRepo, config: &Config, agent: Option<&str>) -> PathBuf {
    let base = effective_project_dir_for(config, agent);
    match config.structure {
        Structure::Host => base.join(&parsed.host).join(&parsed.owner).join(&parsed.repo),
        Structure::Flat => base.join(&parsed.repo),
        Structure::Owner => base.join(&parsed.owner).join(&parsed.repo),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Config, Structure};

    fn repo() -> ParsedRepo {
        ParsedRepo {
            host: "github.com".into(),
            owner: "juev".into(),
            repo: "gclone".into(),
            original_url: "https://github.com/juev/gclone".into(),
        }
    }

    #[test]
    fn flat_structure() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            structure: Structure::Flat,
            ..Config::default()
        };
        assert_eq!(
            to_path_for(&repo(), &cfg, None),
            PathBuf::from("/home/user/src/gclone")
        );
    }

    #[test]
    fn owner_structure() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            ..Config::default()
        };
        assert_eq!(
            to_path_for(&repo(), &cfg, None),
            PathBuf::from("/home/user/src/juev/gclone")
        );
    }

    #[test]
    fn host_structure() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            structure: Structure::Host,
            ..Config::default()
        };
        assert_eq!(
            to_path_for(&repo(), &cfg, None),
            PathBuf::from("/home/user/src/github.com/juev/gclone")
        );
    }

    #[test]
    fn nested_owner_preserves_path() {
        let nested = ParsedRepo {
            owner: "group/subgroup".into(),
            repo: "project".into(),
            ..repo()
        };
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            ..Config::default()
        };
        assert_eq!(
            to_path_for(&nested, &cfg, None),
            PathBuf::from("/home/user/src/group/subgroup/project")
        );
    }

    #[test]
    fn tilde_expansion() {
        let cfg = Config {
            project_dir: "~/src".into(),
            ..Config::default()
        };
        let result = to_path_for(&repo(), &cfg, None);
        let s = result.to_string_lossy();
        assert!(!s.contains('~'));
        assert!(s.ends_with("/juev/gclone"), "got {s}");
    }

    #[test]
    fn owner_with_agent_routes_to_dotdir() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            ..Config::default()
        };
        assert_eq!(
            to_path_for(&repo(), &cfg, Some("morgan")),
            PathBuf::from("/home/user/src/.morgan/juev/gclone")
        );
    }

    #[test]
    fn flat_with_agent() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            structure: Structure::Flat,
            ..Config::default()
        };
        assert_eq!(
            to_path_for(&repo(), &cfg, Some("morgan")),
            PathBuf::from("/home/user/src/.morgan/gclone")
        );
    }

    #[test]
    fn host_with_agent() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            structure: Structure::Host,
            ..Config::default()
        };
        assert_eq!(
            to_path_for(&repo(), &cfg, Some("morgan")),
            PathBuf::from("/home/user/src/.morgan/github.com/juev/gclone")
        );
    }

    #[test]
    fn no_agent_normal_path() {
        let cfg = Config {
            project_dir: "/home/user/src".into(),
            ..Config::default()
        };
        assert_eq!(
            to_path_for(&repo(), &cfg, None),
            PathBuf::from("/home/user/src/juev/gclone")
        );
    }
}
