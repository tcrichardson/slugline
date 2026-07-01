use iced::widget::{column, container, scrollable, text};
use iced::{Element, Font, Length, Task};

use slugline_core::store::NotesStore;

pub struct App {
    date: String,
    lines: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl App {
    /// Build the app by reading (or materializing) the note for `date`.
    pub fn new(store: &NotesStore, date: String) -> Self {
        match store.read_or_create(&date) {
            Ok(content) => Self {
                date,
                lines: content.lines().map(str::to_string).collect(),
                error: None,
            },
            Err(e) => Self {
                date,
                lines: Vec::new(),
                error: Some(format!("Failed to load note: {e}")),
            },
        }
    }

    pub fn title(&self) -> String {
        format!("Slugline — {}", self.date)
    }

    pub fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut col = column![].spacing(2).padding(16);
        if let Some(err) = &self.error {
            col = col.push(text(err.clone()));
        }
        for line in &self.lines {
            // Render each raw line in monospace. Empty lines get a space so they keep height.
            let display = if line.is_empty() { " ".to_string() } else { line.clone() };
            col = col.push(text(display).font(Font::MONOSPACE));
        }
        scrollable(container(col).width(Length::Fill)).into()
    }
}
