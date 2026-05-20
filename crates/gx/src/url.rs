use std::sync::OnceLock;

use regex::Regex;

use crate::errors::{GxError, GxResult};
use crate::types::ParsedRepo;

fn re_https() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^https?://([^/]+)/(.+?)(?:\.git)?/?$").unwrap())
}

fn re_ssh() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^(?:ssh://)?[^@]+@([^/:]+)(?::\d+)?[:/](.+?)(?:\.git)?/?$").unwrap()
    })
}

fn re_git() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^git://([^/]+)/(.+?)(?:\.git)?/?$").unwrap())
}

fn re_shorthand() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^([a-zA-Z0-9_.\-]+)/([a-zA-Z0-9_.\-][a-zA-Z0-9_.\-/]*)$").unwrap()
    })
}

fn re_bare_name() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^[a-zA-Z0-9_.\-]+$").unwrap())
}

fn validate_segments(owner: &str, repo: &str) -> GxResult<()> {
    let segs = owner.split('/').chain(std::iter::once(repo));
    for seg in segs {
        if seg == ".." || seg == "." || seg.is_empty() {
            return Err(GxError::command("Path traversal detected"));
        }
        if seg.contains('\\') || seg.contains('\0') {
            return Err(GxError::command("Invalid characters in repository path"));
        }
    }
    Ok(())
}

fn strip_git_suffix(s: &str) -> String {
    s.strip_suffix(".git").unwrap_or(s).to_string()
}

fn build_parsed(host: &str, path: &str, original_url: &str) -> GxResult<ParsedRepo> {
    let mut segments: Vec<&str> = path.split('/').collect();
    if segments.len() < 2 {
        return Err(GxError::command(format!("Invalid repository path: {path}")));
    }
    let repo = segments.pop().unwrap().to_string();
    let owner = segments.join("/");
    validate_segments(&owner, &repo)?;
    Ok(ParsedRepo {
        host: host.to_string(),
        owner,
        repo,
        original_url: original_url.to_string(),
    })
}

pub fn parse_url(input: &str, default_host: &str, default_owner: &str) -> GxResult<ParsedRepo> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(GxError::command("Empty repository URL"));
    }

    if let Some(c) = re_https().captures(trimmed) {
        return build_parsed(
            c.get(1).map(|m| m.as_str()).unwrap_or(""),
            c.get(2).map(|m| m.as_str()).unwrap_or(""),
            trimmed,
        );
    }
    if let Some(c) = re_git().captures(trimmed) {
        return build_parsed(
            c.get(1).map(|m| m.as_str()).unwrap_or(""),
            c.get(2).map(|m| m.as_str()).unwrap_or(""),
            trimmed,
        );
    }
    if let Some(c) = re_ssh().captures(trimmed) {
        return build_parsed(
            c.get(1).map(|m| m.as_str()).unwrap_or(""),
            c.get(2).map(|m| m.as_str()).unwrap_or(""),
            trimmed,
        );
    }
    if let Some(c) = re_shorthand().captures(trimmed) {
        let owner = c.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let raw_repo = c.get(2).map(|m| m.as_str()).unwrap_or("");
        let repo = raw_repo.trim_end_matches('/').to_string();
        validate_segments(&owner, &repo)?;
        return Ok(ParsedRepo {
            host: default_host.to_string(),
            owner: owner.clone(),
            repo,
            original_url: format!("https://{default_host}/{owner}/{raw_repo}"),
        });
    }
    if re_bare_name().is_match(trimmed) {
        if default_owner.is_empty() {
            return Err(GxError::command(format!(
                "Bare repo name \"{trimmed}\" requires defaultOwner — run: gx config set defaultOwner <owner>"
            )));
        }
        let repo = strip_git_suffix(trimmed);
        validate_segments(default_owner, &repo)?;
        return Ok(ParsedRepo {
            host: default_host.to_string(),
            owner: default_owner.to_string(),
            repo: repo.clone(),
            original_url: format!("https://{default_host}/{default_owner}/{repo}"),
        });
    }

    Err(GxError::command(format!(
        "Cannot parse repository URL: {trimmed}"
    )))
}

pub fn to_clone_url(parsed: &ParsedRepo) -> String {
    if parsed.original_url.starts_with("git@") || parsed.original_url.starts_with("ssh://") {
        parsed.original_url.clone()
    } else {
        format!("https://{}/{}/{}.git", parsed.host, parsed.owner, parsed.repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> ParsedRepo {
        parse_url(s, "github.com", "").expect("parse ok")
    }

    #[test]
    fn shorthand_user_repo() {
        let r = parse("juev/gclone");
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "juev");
        assert_eq!(r.repo, "gclone");
    }

    #[test]
    fn https_with_git_suffix() {
        let r = parse("https://github.com/juev/gclone.git");
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "juev");
        assert_eq!(r.repo, "gclone");
    }

    #[test]
    fn https_without_git_suffix() {
        let r = parse("https://github.com/juev/gclone");
        assert_eq!(r.repo, "gclone");
    }

    #[test]
    fn https_with_trailing_slash() {
        let r = parse("https://github.com/juev/gclone/");
        assert_eq!(r.repo, "gclone");
    }

    #[test]
    fn ssh_git_at_host() {
        let r = parse("git@github.com:juev/gclone.git");
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "juev");
        assert_eq!(r.repo, "gclone");
    }

    #[test]
    fn ssh_without_git_suffix() {
        let r = parse("git@gitlab.com:company/project");
        assert_eq!(r.host, "gitlab.com");
        assert_eq!(r.owner, "company");
        assert_eq!(r.repo, "project");
    }

    #[test]
    fn git_protocol() {
        let r = parse("git://github.com/juev/gclone.git");
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "juev");
        assert_eq!(r.repo, "gclone");
    }

    #[test]
    fn nested_path_gitlab_groups() {
        let r = parse("https://gitlab.com/group/subgroup/repo.git");
        assert_eq!(r.host, "gitlab.com");
        assert_eq!(r.owner, "group/subgroup");
        assert_eq!(r.repo, "repo");
    }

    #[test]
    fn bare_name_with_default_owner() {
        let r = parse_url("anvil", "github.com", "eddacraft").unwrap();
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "eddacraft");
        assert_eq!(r.repo, "anvil");
        assert_eq!(r.original_url, "https://github.com/eddacraft/anvil");
    }

    #[test]
    fn explicit_owner_overrides_default() {
        let r = parse_url("joshuaboys/thing", "github.com", "eddacraft").unwrap();
        assert_eq!(r.owner, "joshuaboys");
        assert_eq!(r.repo, "thing");
    }

    #[test]
    fn bare_strips_git_suffix() {
        let r = parse_url("anvil.git", "github.com", "eddacraft").unwrap();
        assert_eq!(r.repo, "anvil");
        assert_eq!(r.original_url, "https://github.com/eddacraft/anvil");
    }

    #[test]
    fn bare_without_default_owner_errors() {
        let err = parse_url("anvil", "github.com", "").unwrap_err();
        assert!(err.to_string().contains("defaultOwner"));
    }

    #[test]
    fn bare_with_empty_default_owner_errors() {
        let err = parse_url("anvil", "github.com", "").unwrap_err();
        assert!(err.to_string().contains("defaultOwner"));
    }

    #[test]
    fn empty_input_errors() {
        assert!(parse_url("", "github.com", "").is_err());
    }

    #[test]
    fn invalid_url_errors() {
        assert!(parse_url("not a url at all", "github.com", "").is_err());
    }

    #[test]
    fn path_traversal_rejected() {
        assert!(parse_url("https://github.com/../etc/passwd", "github.com", "").is_err());
    }

    #[test]
    fn custom_default_host_for_shorthand() {
        let r = parse_url("juev/gclone", "gitlab.com", "").unwrap();
        assert_eq!(r.host, "gitlab.com");
    }

    #[test]
    fn trims_whitespace() {
        let r = parse_url("  juev/gclone  ", "github.com", "").unwrap();
        assert_eq!(r.repo, "gclone");
    }

    #[test]
    fn dots_in_repo_names_allowed() {
        let r = parse_url("user/my.cool.repo", "github.com", "").unwrap();
        assert_eq!(r.repo, "my.cool.repo");
    }

    #[test]
    fn clone_url_https() {
        let p = parse("https://github.com/juev/gclone.git");
        assert_eq!(to_clone_url(&p), "https://github.com/juev/gclone.git");
    }

    #[test]
    fn clone_url_ssh_at_preserved() {
        let p = parse("git@github.com:juev/gclone.git");
        assert_eq!(to_clone_url(&p), "git@github.com:juev/gclone.git");
    }

    #[test]
    fn clone_url_ssh_proto_preserved() {
        let p = parse("ssh://git@github.com/juev/gclone.git");
        assert_eq!(to_clone_url(&p), "ssh://git@github.com/juev/gclone.git");
    }

    #[test]
    fn clone_url_shorthand_appends_git() {
        let p = parse("juev/gclone");
        assert_eq!(to_clone_url(&p), "https://github.com/juev/gclone.git");
    }
}
