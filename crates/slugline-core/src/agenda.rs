use crate::doc::{SectionKind, scan_document};

/// A scheduled meeting derived from the `## Meetings` section of a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgendaItem {
    pub time: String,
    pub name: String,
    pub heading_line_index: usize,
    pub started: Option<String>,
    pub ended: Option<String>,
}

/// Scheduled meetings for a note, sorted ascending by `HH:MM`. Meetings without a
/// scheduled time are omitted. Port of `web/src/lib/agenda.ts`.
pub fn derive_agenda(lines: &[String]) -> Vec<AgendaItem> {
    let model = scan_document(lines);
    let Some(meetings) = model
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Meetings)
    else {
        return Vec::new();
    };

    let mut items = Vec::new();
    for block in &meetings.blocks {
        let Some(scheduled) = block.meta.iter().find(|m| m.key == "scheduled") else {
            continue;
        };
        let time = scheduled.value.trim();
        if time.is_empty() {
            continue;
        }
        items.push(AgendaItem {
            time: time.to_string(),
            name: block.name.clone(),
            heading_line_index: block.heading_line_index,
            started: block
                .meta
                .iter()
                .find(|m| m.key == "started")
                .map(|m| m.value.trim().to_string()),
            ended: block
                .meta
                .iter()
                .find(|m| m.key == "ended")
                .map(|m| m.value.trim().to_string()),
        });
    }
    items.sort_by(|a, b| a.time.cmp(&b.time));
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_lines(name: &str) -> Vec<String> {
        let path = format!("{}/../../fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"))
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn lists_scheduled_meetings_sorted_by_time() {
        let items = derive_agenda(&fixture_lines("full-day.md"));
        let names: Vec<&str> = items.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["Standup", "Weekly Sync"]);
        assert_eq!(items[0].time, "09:00");
    }

    #[test]
    fn captures_started_ended_status_when_present() {
        let items = derive_agenda(&fixture_lines("full-day.md"));
        let sync = items.iter().find(|i| i.name == "Weekly Sync").unwrap();
        assert_eq!(sync.ended, Some("15:02".to_string()));
    }

    #[test]
    fn omits_meetings_without_a_scheduled_time() {
        let lines: Vec<String> = vec![
            "## Meetings".into(),
            "### A".into(),
            "meta:scheduled 10:00".into(),
            "### B".into(),
            "".into(),
        ];
        let names: Vec<String> = derive_agenda(&lines).into_iter().map(|i| i.name).collect();
        assert_eq!(names, vec!["A".to_string()]);
    }

    #[test]
    fn returns_empty_when_there_is_no_meetings_section() {
        let lines: Vec<String> = vec!["# T".into(), "".into(), "## Notes".into(), "".into()];
        assert_eq!(derive_agenda(&lines), Vec::new());
    }
}
