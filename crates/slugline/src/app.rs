use std::time::{Duration, Instant};

use iced::{Element, Subscription, Task, keyboard, time, window};

use slugline_core::editor::{EditorState, KeyInput, create_editor_state, handle_key};
use slugline_core::store::NotesStore;

use crate::keys::key_string;
use crate::ui::editor_pane;

const SAVE_DEBOUNCE: Duration = Duration::from_millis(750);

pub struct App {
    store: NotesStore,
    date: String,
    editor: EditorState,
    last_saved: String,
    dirty_since: Option<Instant>,
    saving: bool,
    #[allow(dead_code)]
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Key(KeyInput),
    Tick,
    Saved { res: Result<String, String> },
    CloseRequested(window::Id),
}

impl App {
    pub fn new(store: NotesStore, date: String) -> Self {
        let (content, error) = match store.read_or_create(&date) {
            Ok(c) => (c, None),
            Err(e) => (String::new(), Some(format!("Failed to load note: {e}"))),
        };
        let editor = create_editor_state(content.lines().map(str::to_string).collect(), Vec::new());
        Self {
            store,
            date,
            editor,
            last_saved: content,
            dirty_since: None,
            saving: false,
            error,
        }
    }

    pub fn title(&self) -> String {
        format!("Slugline \u{2014} {}", self.date)
    }

    /// The buffer as file content, with a guaranteed trailing newline.
    fn content(&self) -> String {
        let body = self.editor.lines.join("\n");
        if body.ends_with('\n') {
            body
        } else {
            format!("{body}\n")
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Key(input) => {
                let before = self.editor.lines.clone();
                let result = handle_key(&self.editor, &input);
                self.editor = result.state;
                // (result.effect is always None in the walking skeleton; handled in Phase 2/5.)
                if self.editor.lines != before {
                    self.dirty_since = Some(Instant::now());
                }
                Task::none()
            }
            Message::Tick => {
                let idle = self
                    .dirty_since
                    .map(|t| t.elapsed() >= SAVE_DEBOUNCE)
                    .unwrap_or(false);
                if idle && !self.saving {
                    let content = self.content();
                    if content == self.last_saved {
                        self.dirty_since = None;
                        return Task::none();
                    }
                    self.saving = true;
                    let store = self.store.clone();
                    let date = self.date.clone();
                    let to_save = content.clone();
                    return Task::perform(
                        async move {
                            store
                                .write(&date, &to_save)
                                .map(|_| to_save)
                                .map_err(|e| e.to_string())
                        },
                        |res| Message::Saved { res },
                    );
                }
                Task::none()
            }
            Message::Saved { res } => {
                self.saving = false;
                match res {
                    Ok(content) => {
                        self.last_saved = content;
                        if self.content() == self.last_saved {
                            self.dirty_since = None;
                        }
                    }
                    Err(e) => {
                        self.error =
                            Some(format!("Save failed \u{2014} edits kept, will retry: {e}"));
                        // dirty_since stays set, so the next Tick retries.
                    }
                }
                Task::none()
            }
            Message::CloseRequested(id) => {
                // Final synchronous flush so nothing is lost on quit.
                let content = self.content();
                if content != self.last_saved {
                    let _ = self.store.write(&self.date, &content);
                }
                window::close(id)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        editor_pane::view(&self.editor)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::on_key_press(|key, mods| {
                key_string(&key).map(|k| {
                    Message::Key(KeyInput {
                        key: k,
                        ctrl: mods.control(),
                        meta: mods.logo(),
                        shift: mods.shift(),
                    })
                })
            }),
            time::every(Duration::from_millis(250)).map(|_| Message::Tick),
            window::close_requests().map(Message::CloseRequested),
        ])
    }
}
