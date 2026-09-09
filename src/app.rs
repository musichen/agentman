use crate::{Session, SessionAction, ranked_sessions};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAction {
    Trash(usize),
    Launch(usize, SessionAction),
    Rename(usize, String),
}

#[derive(Debug)]
pub struct App {
    pub sessions: Vec<Session>,
    pub filter: String,
    pub selected: usize,
    searching: bool,
    confirming_delete: bool,
    renaming: bool,
    rename_buffer: String,
    pending: Option<UiAction>,
}

impl App {
    #[must_use]
    pub fn new(sessions: Vec<Session>) -> Self {
        Self {
            sessions,
            filter: String::new(),
            selected: 0,
            searching: false,
            confirming_delete: false,
            renaming: false,
            rename_buffer: String::new(),
            pending: None,
        }
    }

    #[must_use]
    pub fn matching(&self) -> Vec<usize> {
        ranked_sessions(&self.sessions, &self.filter)
    }

    #[must_use]
    pub fn visible_titles(&self) -> Vec<&str> {
        self.matching()
            .into_iter()
            .map(|index| self.sessions[index].title.as_str())
            .collect()
    }

    #[must_use]
    pub const fn searching(&self) -> bool {
        self.searching
    }

    #[must_use]
    pub const fn confirming_delete(&self) -> bool {
        self.confirming_delete
    }

    #[must_use]
    pub const fn renaming(&self) -> bool {
        self.renaming
    }

    #[must_use]
    pub fn rename_buffer(&self) -> &str {
        &self.rename_buffer
    }

    pub fn take_action(&mut self) -> Option<UiAction> {
        self.pending.take()
    }

    pub fn key(&mut self, key: char) {
        if self.confirming_delete {
            if matches!(key, 'y' | 'Y')
                && let Some(index) = self.matching().get(self.selected)
            {
                self.pending = Some(UiAction::Trash(*index));
            }
            self.confirming_delete = false;
            return;
        }
        if self.renaming {
            match key {
                '\n' => {
                    if let Some(index) = self.matching().get(self.selected) {
                        self.pending = Some(UiAction::Rename(
                            *index,
                            self.rename_buffer.trim().to_owned(),
                        ));
                    }
                    self.renaming = false;
                }
                '\u{1b}' => self.renaming = false,
                '\u{8}' | '\u{7f}' => {
                    self.rename_buffer.pop();
                }
                _ if !key.is_control() => self.rename_buffer.push(key),
                _ => {}
            }
            return;
        }
        if self.searching {
            match key {
                '\u{1b}' => self.searching = false,
                '\u{8}' | '\u{7f}' => {
                    self.filter.pop();
                    self.selected = 0;
                }
                _ if !key.is_control() => {
                    self.filter.push(key);
                    self.selected = 0;
                }
                _ => {}
            }
            return;
        }
        match key {
            '/' => self.searching = true,
            'j' => self.move_selection(1),
            'k' => self.move_selection(-1),
            'd' if !self.matching().is_empty() => self.confirming_delete = true,
            'r' if !self.matching().is_empty() => {
                self.rename_buffer = self
                    .matching()
                    .get(self.selected)
                    .map_or_else(String::new, |index| self.sessions[*index].title.clone());
                self.renaming = true;
            }
            '\n' => self.emit_launch(SessionAction::Resume),
            'f' => self.emit_launch(SessionAction::Fork),
            'y' => self.emit_launch(SessionAction::YoloResume),
            'Y' => self.emit_launch(SessionAction::YoloFork),
            _ => {}
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        let count = self.matching().len();
        if count == 0 {
            self.selected = 0;
            return;
        }
        self.selected = self.selected.saturating_add_signed(delta).min(count - 1);
    }

    fn emit_launch(&mut self, action: SessionAction) {
        if let Some(index) = self.matching().get(self.selected) {
            self.pending = Some(UiAction::Launch(*index, action));
        }
    }
}
