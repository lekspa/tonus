use color_eyre::eyre;
use ratatui::crossterm;
use ratatui::crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, widgets::TableState};
use std::time::Duration;

use crate::config::Config;
use crate::library::Library;
use crate::player::Player;
use crate::track::Track;
use crate::ui;

pub struct App {
    pub should_quit: bool,
    pub config: Config,
    pub library: Library,
    pub player: Player,
    pub table_state: TableState,
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
            }
        }

        Ok(())
    }

    fn terminal_width() -> u16 {
        crossterm::terminal::size().ok().map_or(80, |(w, _)| w)
    }

    fn handle_events(&mut self) -> eyre::Result {
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') => self.quit(),
                KeyCode::Char('j') | KeyCode::Down => self.table_state.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.table_state.select_previous(),
                KeyCode::Char(' ') => self.player.toggle(),
                KeyCode::Char('+') => self.player.volume_up(),
                KeyCode::Char('-') => self.player.volume_down(),
                KeyCode::Enter => self.play(self.table_state.selected()),
                _ => {}
            }
        }

        Ok(())
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn play(&mut self, selected_row: Option<usize>) {
        if let Some(index) = selected_row {
            let track = &self.library.tracks[index];
            let _ = self.player.play(&track.path);
            self.player.set_track(Some(index));
        }
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.player
            .current_track_id
            .map(|id| &self.library.tracks[id])
    }
}
