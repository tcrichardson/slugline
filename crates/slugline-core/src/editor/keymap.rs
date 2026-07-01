#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInput {
    pub key: String,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEffect {
    Goto(String),
    Today,
    Tab(String),
    Close,
    Save,
    PrevDay,
    NextDay,
    TabNext,
    TabPrev,
    Theme(String),
}

pub struct KeyResult {
    pub state: crate::editor::state::EditorState,
    pub effect: Option<AppEffect>,
}

pub fn handle_key(
    _state: &crate::editor::state::EditorState,
    _key: &KeyInput,
) -> KeyResult {
    todo!()
}
