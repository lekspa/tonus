use color_eyre::eyre;
use ratatui::crossterm;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::TableState};
use std::time::Duration;

use crate::config::Config;
use crate::library::Library;
use crate::player::Player;
use crate::search::Search;
use crate::track::{Track, TrackId};
use crate::ui;

pub struct App {
    pub should_quit: bool,
    pub config: Config,
    pub library: Library,
    pub player: Player,
    pub search: Search,
    pub table_state: TableState,
    pub selected_track_id: Option<TrackId>,
    pub column_widths: Vec<usize>,
    filter: Option<Filter>,
}

struct Filter {
    query: String,
    result: Vec<TrackId>,
}

impl App {
    pub fn new() -> eyre::Result<Self> {
        Ok(Self {
            should_quit: false,
            config: Config::default(),
            library: Library::new(),
            player: Player::new()?,
            search: Search::new(),
            table_state: TableState::new(),
            selected_track_id: None,
            column_widths: ui::compute_column_widths(Self::terminal_width()),
            filter: None,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, config: Config) -> eyre::Result<()> {
        self.config = config;

        if let Some(directory) = &self.config.music_directory {
            self.library.load(directory);
            self.apply_filter();

            while !self.should_quit {
                terminal.draw(|frame| ui::render(frame, self))?;

                if event::poll(Duration::from_millis(16))? {
                    self.handle_events()?;
                }

                self.player.check_playback_completion();
            }
        }

        Ok(())
    }

    /// Returns all track IDs that are rendered inside the table.
    pub fn visible_track_ids(&self) -> &[TrackId] {
        match &self.filter {
            Some(state) => &state.result,
            None => &[],
        }
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.player
            .current_track_id
            .and_then(|id| self.library.get_track(id))
    }

    /// Returns the index of `track_id` in visible track list (filtered or all).
    pub fn track_index(&self, track_id: Option<TrackId>) -> Option<usize> {
        track_id.and_then(|id| self.visible_track_ids().iter().position(|&x| x == id))
    }

    // TODO: Recompute on terminal resize
    fn terminal_width() -> u16 {
        crossterm::terminal::size().ok().map_or(80, |(w, _)| w)
    }

    fn handle_events(&mut self) -> eyre::Result<()> {
        let event = event::read()?;

        match event {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if self.search.is_active() {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.search.reset();
                            self.refresh_filter_if_stale();
                        }
                        KeyCode::Enter => {
                            self.search.set_active(false);
                            self.refresh_filter_if_stale();
                        }
                        _ => {
                            self.search.handle_event(&event);
                            self.refresh_filter_if_stale();
                        }
                    }
                } else {
                    match key_event.code {
                        KeyCode::Char('q') => self.quit(),
                        KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                        KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
                        KeyCode::Char('n') => self.play_next(),
                        KeyCode::Char('p') => self.play_prev(),
                        KeyCode::Char(' ') => self.player.toggle(),
                        KeyCode::Char('+') => self.player.volume_up(),
                        KeyCode::Char('-') => self.player.volume_down(),
                        KeyCode::Char('/') => self.search.set_active(true),
                        KeyCode::Enter => self.play(self.selected_track_id),
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn select_next(&mut self) {
        let track_ids = self.visible_track_ids();
        if track_ids.is_empty() {
            return;
        }
        let index = next_index(self.table_state.selected(), track_ids.len());

        self.selected_track_id = Some(track_ids[index]);
        self.table_state.select(Some(index));
    }

    fn select_prev(&mut self) {
        let track_ids = self.visible_track_ids();
        if track_ids.is_empty() {
            return;
        }
        let index = prev_index(self.table_state.selected(), track_ids.len());

        self.selected_track_id = Some(track_ids[index]);
        self.table_state.select(Some(index));
    }

    fn play(&mut self, track_id: Option<TrackId>) {
        let Some(id) = track_id else {
            return;
        };
        let Some(track) = self.library.get_track(id) else {
            return;
        };
        //TODO: handle error
        let _ = self.player.play(&track.path);
        self.player.set_track(Some(track.id));
    }

    fn play_next(&mut self) {
        let track_ids = self.visible_track_ids();
        if track_ids.is_empty() {
            return;
        }
        let current = self.player.current_track_id.or(self.selected_track_id);
        let index = next_index(self.track_index(current), track_ids.len());
        let next = track_ids[index];

        self.play(Some(next));
        self.table_state.select(Some(index));
        self.selected_track_id = Some(next);
    }

    fn play_prev(&mut self) {
        let track_ids = self.visible_track_ids();
        if track_ids.is_empty() {
            return;
        }
        let current = self.player.current_track_id.or(self.selected_track_id);
        let index = prev_index(self.track_index(current), track_ids.len());
        let prev = track_ids[index];

        self.play(Some(prev));
        self.table_state.select(Some(index));
        self.selected_track_id = Some(prev);
    }

    fn apply_filter(&mut self) {
        let query = self.search.query().to_string();
        let result = self.library.filter(&query);
        self.filter = Some(Filter { query, result });
    }

    fn is_filter_stale(&self) -> bool {
        let query = self.search.query();
        self.filter
            .as_ref()
            .is_none_or(|state| state.query != query)
    }

    fn sync_selection(&mut self) {
        let track_ids = self.visible_track_ids();

        if track_ids.is_empty() {
            self.table_state.select(None);
            self.selected_track_id = None;
            return;
        }

        let track_id = self
            .selected_track_id
            .filter(|id| track_ids.contains(id))
            .unwrap_or_else(|| track_ids[0]);

        let index = self.track_index(Some(track_id)).unwrap_or(0);

        self.table_state.select(Some(index));
        self.selected_track_id = Some(track_id);
    }

    fn refresh_filter_if_stale(&mut self) {
        if self.is_filter_stale() {
            self.apply_filter();
            self.sync_selection();
        }
    }
}
/// Returns the next index from current position not exceeding the `len`.
/// Caller must guard against `len` being 0.
fn next_index(current: Option<usize>, len: usize) -> usize {
    current.map_or(0, |index| (index + 1) % len)
}

/// Returns the previous index from current position not exceeding the `len`.
/// Caller must guard against `len` being 0.
fn prev_index(current: Option<usize>, len: usize) -> usize {
    current.map_or_else(|| len.saturating_sub(1), |index| (index + len - 1) % len)
}
