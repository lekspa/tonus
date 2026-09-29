use color_eyre::eyre;
use ratatui::crossterm;
use ratatui::crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, widgets::TableState};
use std::time::Duration;

use crate::config::Config;
use crate::library::Library;
use crate::player::Player;
use crate::track::{Track, TrackId};
use crate::ui;

pub struct App {
    pub should_quit: bool,
    pub config: Config,
    pub library: Library,
    pub player: Player,
    pub table_state: TableState,
    pub selected_track_id: Option<TrackId>,
    pub column_widths: Vec<usize>,
}

impl App {
    pub fn new() -> eyre::Result<Self> {
        Ok(Self {
            should_quit: false,
            config: Config::default(),
            library: Library::new(),
            player: Player::new()?,
            table_state: TableState::new(),
            selected_track_id: None,
            column_widths: ui::compute_column_widths(Self::terminal_width()),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, config: Config) -> eyre::Result {
        self.config = config;

        if let Some(directory) = &self.config.music_directory {
            self.library.load(directory);

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

    pub fn current_track(&self) -> Option<&Track> {
        self.player
            .current_track_id
            .and_then(|id| self.library.tracks.get(&id))
    }

    pub fn track_position(&self, track_id: Option<TrackId>) -> Option<usize> {
        track_id.and_then(|id| self.library.tracks_ids.iter().position(|&x| x == id))
    }

    pub fn current_track_position(&self) -> Option<usize> {
        self.track_position(self.player.current_track_id)
    }

    fn terminal_width() -> u16 {
        crossterm::terminal::size().ok().map_or(80, |(w, _)| w)
    }

    fn handle_events(&mut self) -> eyre::Result {
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') => self.quit(),
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
                KeyCode::Char('n') => self.play_next(),
                KeyCode::Char('p') => self.play_prev(),
                KeyCode::Char(' ') => self.player.toggle(),
                KeyCode::Char('+') => self.player.volume_up(),
                KeyCode::Char('-') => self.player.volume_down(),
                KeyCode::Enter => self.play(self.selected_track_id),
                _ => {}
            }
        }

        Ok(())
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn select_next(&mut self) {
        let next = next_index(self.table_state.selected(), self.library.len());
        let track_id = self.library.tracks_ids[next];

        self.table_state.select(Some(next));
        self.selected_track_id = Some(track_id);
    }

    fn select_prev(&mut self) {
        let prev = prev_index(self.table_state.selected(), self.library.len());
        let track_id = self.library.tracks_ids[prev];

        self.table_state.select(Some(prev));
        self.selected_track_id = Some(track_id);
    }

    fn play(&mut self, track_id: Option<TrackId>) {
        let Some(id) = track_id else {
            return;
        };
        let Some(track) = self.library.tracks.get(&id) else {
            return;
        };
        //TODO: handle error
        let _ = self.player.play(&track.path);
        self.player.set_track(Some(track.id));
    }

    fn play_next(&mut self) {
        let track_id = self.player.current_track_id.or(self.selected_track_id);
        let next = next_index(self.track_position(track_id), self.library.len());
        let next_track_id = self.library.tracks_ids[next];

        self.play(Some(next_track_id));
        self.table_state.select(Some(next));
        self.selected_track_id = Some(next_track_id);
    }

    fn play_prev(&mut self) {
        let track_id = self.player.current_track_id.or(self.selected_track_id);
        let prev = prev_index(self.track_position(track_id), self.library.len());
        let prev_track_id = self.library.tracks_ids[prev];

        self.play(Some(prev_track_id));
        self.table_state.select(Some(prev));
        self.selected_track_id = Some(prev_track_id);
    }
}

// TODO: caller should guard againt empty library
fn next_index(current: Option<usize>, len: usize) -> usize {
    current.map_or(0, |index| (index + 1) % len)
}
// TODO: caller should guard againt empty library
fn prev_index(current: Option<usize>, len: usize) -> usize {
    current.map_or_else(|| len.saturating_sub(1), |index| (index + len - 1) % len)
}
