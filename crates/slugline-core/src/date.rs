/// Validate a strict `YYYY-MM-DD` string that is also a real calendar date.
pub fn is_valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = b[0..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..10].iter().all(u8::is_ascii_digit);
    if !digits {
        return false;
    }
    let y: i32 = s[0..4].parse().unwrap();
    let m: u32 = s[5..7].parse().unwrap();
    let d: u32 = s[8..10].parse().unwrap();
    if !(1..=12).contains(&m) || d < 1 {
        return false;
    }
    d <= days_in_month(y, m)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Uppercase 3-letter weekday (e.g. "TUE") for a valid `YYYY-MM-DD` date.
/// Uses Sakamoto's algorithm. The caller must pass a date that `is_valid_date` accepts.
pub fn weekday_abbr(s: &str) -> &'static str {
    let y: i32 = s[0..4].parse().unwrap();
    let m: i32 = s[5..7].parse().unwrap();
    let d: i32 = s[8..10].parse().unwrap();
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    let idx = (yy + yy / 4 - yy / 100 + yy / 400 + T[(m - 1) as usize] + d).rem_euclid(7);
    const NAMES: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
    NAMES[idx as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_real_dates_and_rejects_impossible_ones() {
        assert!(is_valid_date("2026-06-23"));
        assert!(is_valid_date("2024-02-29")); // leap year
        assert!(!is_valid_date("2026-02-30"));
        assert!(!is_valid_date("2026-13-01"));
        assert!(!is_valid_date("2026-00-10"));
        assert!(!is_valid_date("2026-6-23")); // not zero-padded
        assert!(!is_valid_date("not-a-date"));
        assert!(!is_valid_date("2026-06-23/../etc")); // traversal attempt
    }

    #[test]
    fn computes_weekday_abbreviation() {
        assert_eq!(weekday_abbr("2026-06-23"), "TUE");
        assert_eq!(weekday_abbr("2000-01-01"), "SAT");
    }
}
