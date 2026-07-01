use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::date::{is_valid_date, weekday_abbr};

/// The built-in daily note template (materialized on open).
pub fn daily_template(date: &str) -> String {
    format!(
        "# {date}-{wd}\n\n## To Do\n\n## Meetings\n\n## Notes\n",
        date = date,
        wd = weekday_abbr(date),
    )
}

/// Create `dir` if missing and verify it is writable (used for startup validation).
pub fn ensure_writable_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let probe = dir.join(".slugline-write-probe");
    fs::write(&probe, b"")?;
    fs::remove_file(&probe)?;
    Ok(())
}

#[derive(Clone)]
pub struct NotesStore {
    notes_dir: PathBuf,
}

impl NotesStore {
    pub fn new(notes_dir: PathBuf) -> Self {
        Self { notes_dir }
    }

    pub fn notes_dir(&self) -> &Path {
        &self.notes_dir
    }

    /// Resolve the on-disk path for a date, rejecting anything that is not a valid `YYYY-MM-DD`.
    pub fn path_for(&self, date: &str) -> Option<PathBuf> {
        if !is_valid_date(date) {
            return None;
        }
        Some(self.notes_dir.join(format!("{date}.md")))
    }

    /// List dates (`YYYY-MM-DD`) that have note files, sorted ascending.
    pub fn list_dates(&self) -> io::Result<Vec<String>> {
        let mut out = Vec::new();
        let rd = match fs::read_dir(&self.notes_dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(e),
        };
        for entry in rd {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if let Some(stem) = name.strip_suffix(".md")
                && is_valid_date(stem)
            {
                out.push(stem.to_string());
            }
        }
        out.sort();
        Ok(out)
    }

    /// Read a note, materializing it from the template if it does not yet exist.
    pub fn read_or_create(&self, date: &str) -> io::Result<String> {
        let path = self
            .path_for(date)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid date"))?;
        match fs::read_to_string(&path) {
            Ok(content) => Ok(content),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let content = daily_template(date);
                self.write(date, &content)?;
                Ok(content)
            }
            Err(e) => Err(e),
        }
    }

    /// Atomically write a note (temp file + rename), ensuring a trailing newline.
    pub fn write(&self, date: &str, content: &str) -> io::Result<()> {
        let path = self
            .path_for(date)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid date"))?;
        fs::create_dir_all(&self.notes_dir)?;

        let mut body = content.to_string();
        if !body.ends_with('\n') {
            body.push('\n');
        }

        let tmp = self.notes_dir.join(format!(".{date}.md.tmp"));
        fs::write(&tmp, body.as_bytes())?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn template_has_title_with_weekday_and_sections() {
        let t = daily_template("2026-06-23");
        assert!(t.starts_with("# 2026-06-23-TUE\n"));
        assert!(t.contains("## To Do"));
        assert!(t.contains("## Meetings"));
        assert!(t.contains("## Notes"));
        assert!(t.ends_with('\n'));
    }

    #[test]
    fn path_for_rejects_invalid_dates() {
        let store = NotesStore::new(tempdir().unwrap().path().to_path_buf());
        assert!(store.path_for("2026-06-23").is_some());
        assert!(store.path_for("../secret").is_none());
        assert!(store.path_for("2026-13-01").is_none());
    }

    #[test]
    fn read_or_create_materializes_then_reuses() {
        let dir = tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        let first = store.read_or_create("2026-06-23").unwrap();
        assert!(first.starts_with("# 2026-06-23-TUE"));
        assert!(dir.path().join("2026-06-23.md").exists());

        // Mutate on disk, then ensure read returns the existing content (not a fresh template).
        store.write("2026-06-23", "# edited").unwrap();
        let second = store.read_or_create("2026-06-23").unwrap();
        assert_eq!(second, "# edited\n");
    }

    #[test]
    fn write_ensures_trailing_newline_atomically() {
        let dir = tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        store.write("2026-06-23", "no newline").unwrap();
        let content = std::fs::read_to_string(dir.path().join("2026-06-23.md")).unwrap();
        assert_eq!(content, "no newline\n");
        // No leftover temp file.
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn list_dates_filters_and_sorts() {
        let dir = tempdir().unwrap();
        let store = NotesStore::new(dir.path().to_path_buf());
        store.write("2026-06-23", "a").unwrap();
        store.write("2026-06-21", "b").unwrap();
        std::fs::write(dir.path().join("README.md"), "x").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "x").unwrap();
        assert_eq!(
            store.list_dates().unwrap(),
            vec!["2026-06-21", "2026-06-23"]
        );
    }

    #[test]
    fn ensure_writable_dir_creates_missing() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("notes");
        ensure_writable_dir(&nested).unwrap();
        assert!(nested.is_dir());
    }
}
