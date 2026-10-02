use ratatui::crossterm;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Default)]
pub struct Search {
    input: Input,
    active: bool,
}

impl Search {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_empty(&self) -> bool {
        self.input.value().is_empty()
    }

    pub fn query(&self) -> &str {
        self.input.value()
    }

    pub fn visual_cursor(&self) -> usize {
        self.input.visual_cursor()
    }

    pub fn visual_scroll(&self, width: usize) -> usize {
        self.input.visual_scroll(width)
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.input = Input::default();
    }

    pub fn set_active(&mut self, value: bool) {
        self.active = value;
    }

    pub fn handle_event(&mut self, event: &crossterm::event::Event) {
        self.input.handle_event(event);
    }
}
