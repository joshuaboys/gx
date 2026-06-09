//! Project-name resolution — the layer above the index that turns a raw
//! query into an exact hit, a high-confidence auto-jump, an ambiguous set of
//! suggestions, or a miss. Ported from `src/lib/resolve-name.ts` under strict
//! behaviour parity.

use crate::fuzzy::{fuzzy_match, FuzzyEntry, FuzzyResult, AUTO_JUMP_THRESHOLD};
use crate::index_store::ProjectIndex;
use crate::types::Config;

/// Outcome of resolving a query against the project index. Mirrors the TS
/// `ResolveResult` discriminated union.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolveResult {
    /// The query is an exact index key. `name` is the query verbatim.
    Exact { name: String, path: String },
    /// A single fuzzy match scored at or above `AUTO_JUMP_THRESHOLD`.
    Auto {
        name: String,
        path: String,
        query: String,
        score: f64,
    },
    /// Multiple candidates, or a single candidate below the auto threshold.
    Ambiguous {
        query: String,
        matches: Vec<FuzzyResult>,
    },
    /// No candidate cleared the similarity threshold.
    Missing { query: String },
}

/// Resolve `query` against `idx`, using `config.similarity_threshold` for the
/// fuzzy pass. Exact index keys win immediately; otherwise a single
/// high-confidence match auto-jumps and anything else is reported as
/// ambiguous or missing.
pub fn resolve_project_name(query: &str, idx: &ProjectIndex, config: &Config) -> ResolveResult {
    if let Some(path) = idx.resolve(query) {
        return ResolveResult::Exact {
            name: query.to_string(),
            path: path.to_string(),
        };
    }

    let entries: Vec<FuzzyEntry> = idx
        .list()
        .into_iter()
        .map(|(name, entry)| FuzzyEntry {
            name,
            path: entry.path,
        })
        .collect();
    let matches = fuzzy_match(query, &entries, config.similarity_threshold);

    if matches.is_empty() {
        return ResolveResult::Missing {
            query: query.to_string(),
        };
    }

    let first = &matches[0];
    if matches.len() == 1 && first.score >= AUTO_JUMP_THRESHOLD {
        return ResolveResult::Auto {
            name: first.name.clone(),
            path: first.path.clone(),
            query: query.to_string(),
            score: first.score,
        };
    }

    ResolveResult::Ambiguous {
        query: query.to_string(),
        matches,
    }
}

/// Render the "did you mean" suggestion list for an ambiguous resolve.
pub fn format_ambiguous(query: &str, matches: &[FuzzyResult]) -> String {
    let mut lines = vec![format!("No exact match for '{query}'. Did you mean:")];
    for (i, m) in matches.iter().enumerate() {
        lines.push(format!(
            "  {}. {} ({}%)",
            i + 1,
            m.name,
            format_pct(m.score),
        ));
    }
    lines.join("\n")
}

/// Render the one-line notice emitted when a query auto-jumps to a match.
pub fn format_auto_match(query: &str, name: &str, score: f64) -> String {
    format!(
        "Fuzzy match: '{query}' -> '{name}' ({}%)",
        format_pct(score)
    )
}

/// Match TS `(score * 100).toFixed(0)`: scale to a percentage and round to a
/// whole number.
fn format_pct(score: f64) -> String {
    format!("{:.0}", score * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IndexEntry;
    use std::path::Path;

    fn entry(path: &str) -> IndexEntry {
        IndexEntry {
            path: path.to_string(),
            url: String::new(),
            cloned_at: String::new(),
            last_visited: None,
        }
    }

    fn fixture_index() -> ProjectIndex {
        let mut idx = ProjectIndex::load(Path::new("/nonexistent-resolve-name-fixture"));
        idx.add("myproject", entry("/tmp/myproject"));
        idx.add("myapp", entry("/tmp/myapp"));
        idx.add("gx", entry("/tmp/gx"));
        idx.add("unrelated", entry("/tmp/unrelated"));
        idx
    }

    #[test]
    fn exact_match_returns_exact() {
        let idx = fixture_index();
        let result = resolve_project_name("gx", &idx, &Config::default());
        assert_eq!(
            result,
            ResolveResult::Exact {
                name: "gx".into(),
                path: "/tmp/gx".into(),
            }
        );
    }

    #[test]
    fn existing_name_is_exact_not_fuzzy() {
        let idx = fixture_index();
        let result = resolve_project_name("myapp", &idx, &Config::default());
        assert!(matches!(result, ResolveResult::Exact { .. }));
    }

    #[test]
    fn auto_match_for_single_high_confidence_result() {
        let idx = fixture_index();
        // "unrelate" is very close to "unrelated" and matches nothing else.
        let result = resolve_project_name("unrelate", &idx, &Config::default());
        match result {
            ResolveResult::Auto {
                name,
                path,
                query,
                score,
            } => {
                assert_eq!(name, "unrelated");
                assert_eq!(path, "/tmp/unrelated");
                assert_eq!(query, "unrelate");
                assert!(score >= AUTO_JUMP_THRESHOLD);
            }
            other => panic!("expected Auto, got {other:?}"),
        }
    }

    #[test]
    fn multiple_fuzzy_matches_are_auto_or_ambiguous() {
        let idx = fixture_index();
        // "my" is a prefix of both "myproject" and "myapp".
        let result = resolve_project_name("my", &idx, &Config::default());
        match result {
            ResolveResult::Ambiguous { query, matches } => {
                assert_eq!(query, "my");
                assert!(matches.len() > 1);
            }
            // Scores may push this to a single auto-jump; both are valid.
            ResolveResult::Auto { .. } => {}
            other => panic!("expected Ambiguous or Auto, got {other:?}"),
        }
    }

    #[test]
    fn missing_when_no_matches() {
        let idx = fixture_index();
        let result = resolve_project_name("zzzznothing", &idx, &Config::default());
        assert_eq!(
            result,
            ResolveResult::Missing {
                query: "zzzznothing".into(),
            }
        );
    }

    #[test]
    fn format_ambiguous_numbers_suggestions() {
        let matches = vec![
            FuzzyResult {
                name: "myproject".into(),
                score: 0.82,
                path: "/tmp/myproject".into(),
            },
            FuzzyResult {
                name: "myapp".into(),
                score: 0.75,
                path: "/tmp/myapp".into(),
            },
        ];
        let output = format_ambiguous("my", &matches);
        assert!(output.contains("No exact match for 'my'. Did you mean:"));
        assert!(output.contains("1. myproject (82%)"));
        assert!(output.contains("2. myapp (75%)"));
    }

    #[test]
    fn format_auto_match_message() {
        let output = format_auto_match("myap", "myapp", 0.92);
        assert_eq!(output, "Fuzzy match: 'myap' -> 'myapp' (92%)");
    }
}
