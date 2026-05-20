use std::time::{SystemTime, UNIX_EPOCH};

/// Pure relative-time formatter. The `now_ms` parameter lets callers (and
/// tests) inject a deterministic clock; production callers use [`relative_time`].
pub fn relative_time_at(iso: &str, now_ms: i64) -> String {
    if iso.is_empty() {
        return String::new();
    }
    let Some(t_ms) = parse_iso_ms(iso) else {
        return String::new();
    };
    let diff = now_ms - t_ms;
    if diff < 0 {
        return String::new();
    }
    let seconds = diff / 1000;
    if seconds < 60 {
        return "just now".into();
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!(
            "{minutes} {unit} ago",
            unit = if minutes == 1 { "minute" } else { "minutes" }
        );
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!(
            "{hours} {unit} ago",
            unit = if hours == 1 { "hour" } else { "hours" }
        );
    }
    let days = hours / 24;
    if days < 14 {
        return format!(
            "{days} {unit} ago",
            unit = if days == 1 { "day" } else { "days" }
        );
    }
    let weeks = days / 7;
    if days < 60 {
        return format!(
            "{weeks} {unit} ago",
            unit = if weeks == 1 { "week" } else { "weeks" }
        );
    }
    let months = days / 30;
    format!(
        "{months} {unit} ago",
        unit = if months == 1 { "month" } else { "months" }
    )
}

pub fn relative_time(iso: &str) -> String {
    relative_time_at(iso, now_ms())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Minimal ISO-8601 parser sufficient for the index format
/// (`YYYY-MM-DDTHH:MM:SS[.fff]Z` or `±HH:MM`). Returns milliseconds since
/// the Unix epoch.
fn parse_iso_ms(iso: &str) -> Option<i64> {
    let bytes = iso.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let year: i64 = std::str::from_utf8(&bytes[0..4]).ok()?.parse().ok()?;
    if bytes[4] != b'-' {
        return None;
    }
    let month: u32 = std::str::from_utf8(&bytes[5..7]).ok()?.parse().ok()?;
    if bytes[7] != b'-' {
        return None;
    }
    let day: u32 = std::str::from_utf8(&bytes[8..10]).ok()?.parse().ok()?;
    if bytes[10] != b'T' && bytes[10] != b' ' {
        return None;
    }
    let hour: u32 = std::str::from_utf8(&bytes[11..13]).ok()?.parse().ok()?;
    if bytes[13] != b':' {
        return None;
    }
    let minute: u32 = std::str::from_utf8(&bytes[14..16]).ok()?.parse().ok()?;
    if bytes[16] != b':' {
        return None;
    }
    let second: u32 = std::str::from_utf8(&bytes[17..19]).ok()?.parse().ok()?;

    let mut frac_ms: i64 = 0;
    let mut idx = 19usize;
    if idx < bytes.len() && bytes[idx] == b'.' {
        idx += 1;
        let frac_start = idx;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
        let frac_str = std::str::from_utf8(&bytes[frac_start..idx]).ok()?;
        if !frac_str.is_empty() {
            let frac: i64 = frac_str.parse().ok()?;
            let scale = 10_i64.pow(frac_str.len() as u32);
            frac_ms = (frac * 1000) / scale;
        }
    }

    let mut tz_offset_minutes: i64 = 0;
    if idx < bytes.len() {
        match bytes[idx] {
            b'Z' => {}
            b'+' | b'-' => {
                let sign: i64 = if bytes[idx] == b'+' { 1 } else { -1 };
                if idx + 6 > bytes.len() || bytes[idx + 3] != b':' {
                    return None;
                }
                let oh: i64 = std::str::from_utf8(&bytes[idx + 1..idx + 3])
                    .ok()?
                    .parse()
                    .ok()?;
                let om: i64 = std::str::from_utf8(&bytes[idx + 4..idx + 6])
                    .ok()?
                    .parse()
                    .ok()?;
                tz_offset_minutes = sign * (oh * 60 + om);
            }
            _ => return None,
        }
    }

    let days = days_from_civil(year, month as i64, day as i64);
    let seconds_utc = days * 86_400
        + (hour as i64) * 3600
        + (minute as i64) * 60
        + second as i64
        - tz_offset_minutes * 60;
    Some(seconds_utc * 1000 + frac_ms)
}

/// Howard Hinnant's `days_from_civil` — days from 1970-01-01 to (y, m, d).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW_MS: i64 = 1_700_000_000_000; // 2023-11-14T22:13:20Z

    fn iso_at(now: i64, offset_ms: i64) -> String {
        let t = now - offset_ms;
        // Build an ISO-8601 string the parser round-trips on. Cheap path: use
        // chrono's logic inline via a tiny civil-from-days inverse. We only
        // need to feed parse_iso_ms valid strings, so reuse parse_iso_ms's
        // counterpart by constructing strings manually for the cases we test.
        // Here we craft a known-good ISO via a separate helper.
        let s = ms_to_iso(t);
        // sanity: roundtrip
        assert_eq!(parse_iso_ms(&s).unwrap(), t, "roundtrip {s}");
        s
    }

    fn ms_to_iso(ms: i64) -> String {
        let seconds = ms.div_euclid(1000);
        let frac_ms = ms.rem_euclid(1000);
        let days = seconds.div_euclid(86_400);
        let sod = seconds.rem_euclid(86_400);
        let (y, mo, d) = civil_from_days(days);
        let h = sod / 3600;
        let mi = (sod % 3600) / 60;
        let s = sod % 60;
        format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{frac_ms:03}Z")
    }

    fn civil_from_days(z: i64) -> (i64, i64, i64) {
        let z = z + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        (if m <= 2 { y + 1 } else { y }, m, d)
    }

    #[test]
    fn just_now_under_60_seconds() {
        let s = iso_at(NOW_MS, 30_000);
        assert_eq!(relative_time_at(&s, NOW_MS), "just now");
    }

    #[test]
    fn minutes_plural() {
        let s = iso_at(NOW_MS, 5 * 60_000);
        assert_eq!(relative_time_at(&s, NOW_MS), "5 minutes ago");
    }

    #[test]
    fn minute_singular() {
        let s = iso_at(NOW_MS, 90_000);
        assert_eq!(relative_time_at(&s, NOW_MS), "1 minute ago");
    }

    #[test]
    fn hours_plural() {
        let s = iso_at(NOW_MS, 3 * 3_600_000);
        assert_eq!(relative_time_at(&s, NOW_MS), "3 hours ago");
    }

    #[test]
    fn days_plural() {
        let s = iso_at(NOW_MS, 2 * 86_400_000);
        assert_eq!(relative_time_at(&s, NOW_MS), "2 days ago");
    }

    #[test]
    fn weeks_plural() {
        let s = iso_at(NOW_MS, 14 * 86_400_000);
        assert_eq!(relative_time_at(&s, NOW_MS), "2 weeks ago");
    }

    #[test]
    fn months_plural() {
        let s = iso_at(NOW_MS, 60 * 86_400_000);
        assert_eq!(relative_time_at(&s, NOW_MS), "2 months ago");
    }

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(relative_time_at("", NOW_MS), "");
    }

    #[test]
    fn unparseable_returns_empty() {
        assert_eq!(relative_time_at("not-an-iso", NOW_MS), "");
    }

    #[test]
    fn future_returns_empty() {
        let s = iso_at(NOW_MS, -1_000);
        assert_eq!(relative_time_at(&s, NOW_MS), "");
    }
}
