pub const AUTO_JUMP_THRESHOLD: f64 = 0.85;

#[derive(Debug, Clone, PartialEq)]
pub struct FuzzyResult {
    pub name: String,
    pub score: f64,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct FuzzyEntry {
    pub name: String,
    pub path: String,
}

fn jaro(a: &[char], b: &[char]) -> f64 {
    if a == b {
        return 1.0;
    }
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 || b_len == 0 {
        return 0.0;
    }

    let match_dist = (a_len.max(b_len) / 2).saturating_sub(1);

    let mut a_matches = vec![false; a_len];
    let mut b_matches = vec![false; b_len];

    let mut matches: usize = 0;
    let mut transpositions: usize = 0;

    for i in 0..a_len {
        let start = i.saturating_sub(match_dist);
        let end = (i + match_dist + 1).min(b_len);
        for j in start..end {
            if b_matches[j] || a[i] != b[j] {
                continue;
            }
            a_matches[i] = true;
            b_matches[j] = true;
            matches += 1;
            break;
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut k = 0usize;
    for i in 0..a_len {
        if !a_matches[i] {
            continue;
        }
        while !b_matches[k] {
            k += 1;
        }
        if a[i] != b[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    (m / a_len as f64 + m / b_len as f64 + (m - (transpositions as f64) / 2.0) / m) / 3.0
}

pub fn jaro_winkler(a: &str, b: &str) -> f64 {
    let al: Vec<char> = a.to_lowercase().chars().collect();
    let bl: Vec<char> = b.to_lowercase().chars().collect();

    let jaro_score = jaro(&al, &bl);

    let max_prefix = 4.min(al.len()).min(bl.len());
    let mut prefix_len = 0usize;
    for i in 0..max_prefix {
        if al[i] == bl[i] {
            prefix_len += 1;
        } else {
            break;
        }
    }
    let p = 0.1;
    jaro_score + (prefix_len as f64) * p * (1.0 - jaro_score)
}

pub fn fuzzy_match(query: &str, entries: &[FuzzyEntry], threshold: f64) -> Vec<FuzzyResult> {
    if query.is_empty() || entries.is_empty() {
        return Vec::new();
    }
    let mut results: Vec<FuzzyResult> = entries
        .iter()
        .map(|e| FuzzyResult {
            name: e.name.clone(),
            score: jaro_winkler(query, &e.name),
            path: e.path.clone(),
        })
        .filter(|r| r.score >= threshold)
        .collect();
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_returns_one() {
        assert_eq!(jaro_winkler("gx", "gx"), 1.0);
    }

    #[test]
    fn unrelated_low_score() {
        assert!(jaro_winkler("abc", "xyz") < 0.5);
    }

    #[test]
    fn typo_high_score() {
        assert!(jaro_winkler("gclone", "gclne") > 0.85);
    }

    #[test]
    fn both_empty_is_one() {
        assert_eq!(jaro_winkler("", ""), 1.0);
    }

    #[test]
    fn one_empty_is_zero() {
        assert_eq!(jaro_winkler("abc", ""), 0.0);
        assert_eq!(jaro_winkler("", "abc"), 0.0);
    }

    #[test]
    fn single_char_match() {
        assert_eq!(jaro_winkler("a", "a"), 1.0);
    }

    #[test]
    fn single_char_mismatch() {
        assert!(jaro_winkler("a", "b") < 0.5);
    }

    #[test]
    fn prefix_bonus_martha() {
        // Classic Jaro-Winkler test case.
        assert!(jaro_winkler("MARTHA", "MARHTA") > 0.9);
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(jaro_winkler("GClone", "gclone"), 1.0);
    }

    #[test]
    fn bounds_zero_to_one() {
        let s = jaro_winkler("hello", "world");
        assert!((0.0..=1.0).contains(&s));
    }

    fn make_entries() -> Vec<FuzzyEntry> {
        vec![
            FuzzyEntry {
                name: "gclone".into(),
                path: "/projects/gclone".into(),
            },
            FuzzyEntry {
                name: "cockpit".into(),
                path: "/projects/cockpit".into(),
            },
            FuzzyEntry {
                name: "gx".into(),
                path: "/projects/gx".into(),
            },
            FuzzyEntry {
                name: "dotfiles".into(),
                path: "/projects/dotfiles".into(),
            },
            FuzzyEntry {
                name: "next-app".into(),
                path: "/projects/next-app".into(),
            },
        ]
    }

    #[test]
    fn ranks_by_score() {
        let r = fuzzy_match("gc", &make_entries(), 0.7);
        assert!(!r.is_empty());
        assert_eq!(r[0].name, "gclone");
    }

    #[test]
    fn gclon_picks_gclone() {
        let r = fuzzy_match("gclon", &make_entries(), 0.7);
        assert!(!r.is_empty());
        assert_eq!(r[0].name, "gclone");
    }

    #[test]
    fn no_matches_above_threshold_returns_empty() {
        let r = fuzzy_match("xxx", &make_entries(), 0.7);
        assert!(r.is_empty());
    }

    #[test]
    fn cock_picks_cockpit() {
        let r = fuzzy_match("cock", &make_entries(), 0.7);
        assert!(!r.is_empty());
        assert_eq!(r[0].name, "cockpit");
    }

    #[test]
    fn custom_threshold_strict_and_loose() {
        let strict = fuzzy_match("gclon", &make_entries(), 0.99);
        assert!(strict.is_empty());
        let loose = fuzzy_match("gclon", &make_entries(), 0.1);
        assert!(!loose.is_empty());
    }

    #[test]
    fn sorted_descending() {
        let r = fuzzy_match("g", &make_entries(), 0.4);
        for w in r.windows(2) {
            assert!(w[0].score >= w[1].score);
        }
    }

    #[test]
    fn result_shape() {
        let r = fuzzy_match("gclone", &make_entries(), 0.7);
        assert!(!r.is_empty());
        let first = &r[0];
        assert!(!first.name.is_empty());
        assert!(!first.path.is_empty());
        assert!(first.score > 0.0);
    }

    #[test]
    fn empty_entries_returns_empty() {
        assert!(fuzzy_match("test", &[], 0.7).is_empty());
    }

    #[test]
    fn empty_query_returns_empty() {
        assert!(fuzzy_match("", &make_entries(), 0.7).is_empty());
    }
}
