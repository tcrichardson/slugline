use chrono::Local;

/// Today's date in the local timezone, formatted `YYYY-MM-DD`.
pub fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
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
}
