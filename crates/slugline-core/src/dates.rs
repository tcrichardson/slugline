use chrono::{Days, Local, NaiveDate};

/// Today's date in the local timezone, formatted `YYYY-MM-DD`.
pub fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// Add `n` days (may be negative) to an ISO `YYYY-MM-DD` date, returning a new ISO date.
/// On any parse/overflow failure the input is returned unchanged (callers always pass
/// validated dates, so this is a safety net rather than an expected path).
pub fn add_days(date: &str, n: i64) -> String {
    let Ok(base) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
        return date.to_string();
    };
    let shifted = if n >= 0 {
        base.checked_add_days(Days::new(n as u64))
    } else {
        base.checked_sub_days(Days::new(n.unsigned_abs()))
    };
    shifted.map_or_else(|| date.to_string(), |d| d.format("%Y-%m-%d").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::is_valid_date;

    #[test]
    fn today_iso_is_a_valid_yyyy_mm_dd() {
        let t = today_iso();
        assert_eq!(t.len(), 10, "expected YYYY-MM-DD, got {t:?}");
        assert!(
            is_valid_date(&t),
            "today_iso() produced an invalid date: {t:?}"
        );
    }

    #[test]
    fn adds_days_across_month_and_year_boundaries() {
        assert_eq!(add_days("2026-12-31", 1), "2027-01-01");
        assert_eq!(add_days("2026-03-01", -1), "2026-02-28");
    }

    #[test]
    fn add_days_zero_is_identity_and_bad_input_is_unchanged() {
        assert_eq!(add_days("2026-06-23", 0), "2026-06-23");
        assert_eq!(add_days("not-a-date", 5), "not-a-date");
    }
}
