use color_eyre::eyre::Result;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::widgets::{ListState, TableState};
use ratatui::{DefaultTerminal, crossterm};
use std::time::Duration;
use tracing::{trace, warn};

use crate::cli::Cli;
use crate::config::Config;
use crate::filter::Filter;
use crate::indexing::{index_of, next_index, prev_index};
use crate::library::Library;
use crate::player::{Player, PlayerStatus, Repeat};
use crate::search::Search;
use crate::track::{Track, TrackId};
use crate::ui;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
}

#[derive(PartialEq, Eq)]
pub enum View {
    Main,
    Keymaps,
}

pub struct App {
    should_quit: bool,
    pub mode: Mode,
    pub view: View,
    #[allow(dead_code)]
    pub cli: Cli,
    pub config: Config,
    pub library: Library,
    pub player: Player,
    pub search: Search,
    pub table_state: TableState,
    pub selected_track_id: Option<TrackId>,
    pub keymaps_list_state: ListState,
    pub column_widths: Vec<usize>,
    filter: Filter,
}

impl App {
    pub fn new(config: Config, cli: Cli) -> Result<Self> {
        Ok(Self {
            should_quit: false,
            mode: Mode::Normal,
            view: View::Main,
            cli,
            config,
            library: Library::new(),
            player: Player::new()?,
            search: Search::new(),
            table_state: TableState::new(),
            selected_track_id: None,
            keymaps_list_state: ListState::default().with_selected(Some(0)),
            column_widths: ui::compute_column_widths(terminal_width()),
            filter: Filter::default(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        if let Some(dir) = self.config.music_dir() {
            self.library.load(dir);
            self.apply_filter();

            while !self.should_quit {
                terminal.draw(|frame| ui::render(frame, self))?;

                if event::poll(self.timeout())? {
                    self.handle_events()?;
                }

                if self.player.is_playback_finished() {
                    self.player.on_playback_completion();
                    self.advance_to_next_track();
                }
            }
        } else {
            self.welcome_screen(terminal)?;
        }

        Ok(())
    }

    fn advance_to_next_track(&mut self) {
        match self.player.repeat {
            Repeat::Off => {
                let queue = self.playback_queue();
                if queue.is_empty() {
                    return;
                }
                let current = self.player.current_track_id;
                // Should not advance to first track after playing the last
                if index_of(current, queue).unwrap_or(0) + 1 < queue.len() {
                    self.play_next();
                }
            }
            Repeat::All => self.play_next(),
            // TODO: avoid redecoding the track again
            Repeat::One => self.play(self.player.current_track_id),
        }
    }

    /// Returns all track IDs that are rendered inside the table.
    pub fn visible_track_ids(&self) -> &[TrackId] {
        self.filter.result()
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.player
            .current_track_id
            .and_then(|id| self.library.get_track(id))
    }

    /// Returns the index of track in visible track list (filtered or all).
    pub fn index_of_track(&self, track_id: Option<TrackId>) -> Option<usize> {
        index_of(track_id, self.filter.result())
    }

    /// Returns the timeout used for events polling.
    /// The smaller the number, the higher the CPU usage.
    fn timeout(&self) -> Duration {
        match self.player.status {
            PlayerStatus::Playing => Duration::from_millis(32),
            _ => Duration::from_millis(256),
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        let event = event::read()?;
        trace!(?event);
        match event {
            event::Event::Key(e) if e.kind == KeyEventKind::Press => match self.view {
                View::Main => match self.mode {
                    Mode::Normal => match e.code {
                        KeyCode::Char('q') => self.quit(),
                        KeyCode::Char('/') => self.show_search(),
                        KeyCode::Char('?') => self.toggle_keymaps_menu(),
                        KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                        KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
                        KeyCode::Enter => self.play(self.selected_track_id),
                        KeyCode::Char(' ') => self.pause_play(),
                        KeyCode::Char('n') => self.play_next(),
                        KeyCode::Char('p') => self.play_prev(),
                        KeyCode::Char('+' | '=') => self.player.volume_up(),
                        KeyCode::Char('-') => self.player.volume_down(),
                        KeyCode::Char('s') => self.toggle_shuffle(),
                        KeyCode::Char('r') => self.player.toggle_repeat(),
                        KeyCode::Right if e.modifiers == KeyModifiers::CONTROL => {
                            self.seek_forward();
                        }
                        KeyCode::Left if e.modifiers == KeyModifiers::CONTROL => {
                            self.seek_backwards();
                        }
                        _ => {}
                    },
                    Mode::Search => match e.code {
                        KeyCode::Esc => {
                            self.search.reset();
                            self.mode = Mode::Normal;
                            self.apply_filter_if_stale();
                            self.reshuffle();
                        }
                        KeyCode::Enter => {
                            self.mode = Mode::Normal;
                            self.apply_filter_if_stale();
                            self.reshuffle();
                        }
                        _ => {
                            self.search.handle_event(&event);
                            self.apply_filter_if_stale();
                        }
                    },
                },
                View::Keymaps => match e.code {
                    KeyCode::Esc | KeyCode::Char('?') => self.toggle_keymaps_menu(),
                    KeyCode::Char('j') | KeyCode::Down => self.keymaps_list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => self.keymaps_list_state.select_previous(),
                    _ => {}
                },
            },
            event::Event::Resize(columns, _) => {
                self.column_widths = ui::compute_column_widths(columns);
            }
            _ => {}
        }

        Ok(())
    }

    fn welcome_screen(&self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| ui::render_welcome_screen(frame, self))?;

            if let event::Event::Key(key) = crossterm::event::read()?
                && key.kind == event::KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn show_search(&mut self) {
        self.mode = Mode::Search;
    }

    fn toggle_keymaps_menu(&mut self) {
        self.view = match self.view {
            View::Keymaps => View::Main,
            View::Main => View::Keymaps,
        }
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
        if let Err(e) = self.player.play(track) {
            warn!("Failed to play {}: {e:?}", track.path.display());
        }
    }

    fn play_next(&mut self) {
        let queue = self.playback_queue();
        if queue.is_empty() {
            return;
        }
        let current = self.player.current_track_id.or(self.selected_track_id);
        let index = next_index(index_of(current, queue), queue.len());
        let next = Some(queue[index]);

        self.play(next);
        self.table_state.select(self.index_of_track(next));
        self.selected_track_id = next;
    }

    fn play_prev(&mut self) {
        let queue = self.playback_queue();
        if queue.is_empty() {
            return;
        }
        let current = self.player.current_track_id.or(self.selected_track_id);
        let index = prev_index(index_of(current, queue), queue.len());
        let prev = Some(queue[index]);

        self.play(prev);
        self.table_state.select(self.index_of_track(prev));
        self.selected_track_id = prev;
    }

    /// Returns the list of shuffled `track_ids` if shuffle is enabled.
    /// Otherwise, returns the list of `track_ids` rendered inside the table widget.
    fn playback_queue(&self) -> &[TrackId] {
        if self.player.shuffle {
            &self.player.shuffled_track_ids
        } else {
            self.filter.result()
        }
    }

    fn apply_filter(&mut self) {
        let query = self.search.query();
        let query_lower = query.to_lowercase();
        let track_ids = self.library.track_ids();
        let filter = |id: TrackId| {
            self.library
                .get_track(id)
                .is_some_and(|track| track.contains(&query_lower))
        };

        self.filter.apply(query, track_ids, filter);
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

        let index = self.index_of_track(Some(track_id)).unwrap_or(0);

        self.table_state.select(Some(index));
        self.selected_track_id = Some(track_id);
    }

    fn apply_filter_if_stale(&mut self) {
        let query = self.search.query();

        if self.filter.is_stale(query) {
            self.apply_filter();
            self.sync_selection();
        }
    }

    /// Restarts the current track if player is stopped, otherwise toggles the playback.
    fn pause_play(&mut self) {
        if self.player.status == PlayerStatus::Stopped {
            self.play(self.player.current_track_id);
        } else {
            self.player.toggle();
        }
    }

    fn toggle_shuffle(&mut self) {
        let track_ids = self.filter.result();

        if self.player.shuffle {
            self.player.set_shuffle(false);
            self.player.shuffled_track_ids = Vec::new();
        } else {
            self.player.shuffle_track_ids(track_ids);
            self.player.set_shuffle(true);
        }
    }

    fn reshuffle(&mut self) {
        if self.player.shuffle {
            self.player.shuffle_track_ids(self.filter.result());
        }
    }

    fn seek_forward(&mut self) {
        let Some(id) = self.player.current_track_id else {
            return;
        };
        let Some(track) = self.library.get_track(id) else {
            return;
        };
        self.player.seek_forward(track);
    }

    fn seek_backwards(&mut self) {
        let Some(id) = self.player.current_track_id else {
            return;
        };
        let Some(track) = self.library.get_track(id) else {
            return;
        };
        self.player.seek_backward(track);
    }
}

fn terminal_width() -> u16 {
    crossterm::terminal::size().ok().map_or(80, |(w, _)| w)
}
