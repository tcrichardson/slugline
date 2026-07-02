use chrono::{Datelike, Days, Local, NaiveDate};

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

/// A single day cell in a month grid: its ISO date and whether it belongs to the
/// requested month (vs. a leading/trailing day borrowed from an adjacent month).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthCell {
    pub date: String,
    pub in_month: bool,
}

/// A calendar year/month pair (`month` is 1-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearMonth {
    pub year: i32,
    pub month: u32,
}

/// A 6x7 grid (weeks start Sunday) covering `month` (1-12) of `year`.
pub fn month_grid(year: i32, month: u32) -> Vec<Vec<MonthCell>> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    let offset = first.weekday().num_days_from_sunday() as u64;
    let mut cursor = first - Days::new(offset);

    let mut weeks = Vec::with_capacity(6);
    for _ in 0..6 {
        let mut row = Vec::with_capacity(7);
        for _ in 0..7 {
            row.push(MonthCell {
                date: cursor.format("%Y-%m-%d").to_string(),
                in_month: cursor.month() == month,
            });
            cursor = cursor + Days::new(1);
        }
        weeks.push(row);
    }
    weeks
}

/// Extract the year/month of an ISO `YYYY-MM-DD` date.
pub fn year_month(date: &str) -> YearMonth {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| YearMonth {
            year: d.year(),
            month: d.month(),
        })
        .unwrap_or(YearMonth {
            year: 1970,
            month: 1,
        })
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

    #[test]
    fn builds_a_6x7_month_grid_with_first_of_month_and_out_of_month_days() {
        let g = month_grid(2026, 6);
        assert_eq!(g.len(), 6);
        assert_eq!(g[0].len(), 7);
        let flat: Vec<&MonthCell> = g.iter().flatten().collect();
        assert!(
            flat.iter()
                .find(|c| c.date == "2026-06-01")
                .unwrap()
                .in_month
        );
        assert!(flat.iter().any(|c| !c.in_month));
    }

    #[test]
    fn extracts_year_and_month() {
        assert_eq!(
            year_month("2026-06-23"),
            YearMonth {
                year: 2026,
                month: 6
            }
        );
    }
}
