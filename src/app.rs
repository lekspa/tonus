use color_eyre::eyre::Result;
use ratatui::crossterm;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::TableState};
use std::time::Duration;
use tracing::{trace, warn};

use crate::cli::Cli;
use crate::config::Config;
use crate::filter::Filter;
use crate::library::Library;
use crate::player::{Player, PlayerStatus, Repeat};
use crate::search::Search;
use crate::track::{Track, TrackId};
use crate::ui;

pub struct App {
    pub should_quit: bool,
    #[allow(dead_code)]
    pub cli: Cli,
    pub config: Config,
    pub library: Library,
    pub player: Player,
    pub search: Search,
    pub table_state: TableState,
    pub selected_track_id: Option<TrackId>,
    pub column_widths: Vec<usize>,
    filter: Filter,
}

impl App {
    pub fn new(config: Config, cli: Cli) -> Result<Self> {
        Ok(Self {
            should_quit: false,
            cli,
            config,
            library: Library::new(),
            player: Player::new()?,
            search: Search::new(),
            table_state: TableState::new(),
            selected_track_id: None,
            column_widths: ui::compute_column_widths(Self::terminal_width()),
            filter: Filter::default(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        if let Some(directory) = &self.config.music_directory {
            self.library.load(directory);
            self.apply_filter();

            while !self.should_quit {
                terminal.draw(|frame| ui::render(frame, self))?;

                if event::poll(Duration::from_millis(16))? {
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

    // TODO: Recompute on terminal resize
    fn terminal_width() -> u16 {
        crossterm::terminal::size().ok().map_or(80, |(w, _)| w)
    }

    fn handle_events(&mut self) -> Result<()> {
        let event = event::read()?;
        trace!(?event);
        match event {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if self.search.is_active() {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.search.reset();
                            self.apply_filter_if_stale();
                            self.reshuffle();
                        }
                        KeyCode::Enter => {
                            self.search.set_active(false);
                            self.apply_filter_if_stale();
                            self.reshuffle();
                        }
                        _ => {
                            self.search.handle_event(&event);
                            self.apply_filter_if_stale();
                        }
                    }
                } else {
                    match key_event.code {
                        KeyCode::Char('q') => self.quit(),
                        KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                        KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
                        KeyCode::Char('n') => self.play_next(),
                        KeyCode::Char('p') => self.play_prev(),
                        KeyCode::Char(' ') => self.toggle(),
                        KeyCode::Char('s') => self.toggle_shuffle(),
                        KeyCode::Char('r') => self.player.toggle_repeat(),
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

    fn welcome_screen(&self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(ui::render_welcome_screen)?;

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
    fn toggle(&mut self) {
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

/// Returns index of element in a given slice.
fn index_of<T: PartialEq>(element: Option<T>, slice: &[T]) -> Option<usize> {
    element.and_then(|e| slice.iter().position(|x| *x == e))
}
