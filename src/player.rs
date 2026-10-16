use color_eyre::eyre;
use rand::seq::SliceRandom;
use rodio::source::EmptyCallback;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink};
use std::fmt::Display;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::timer::Timer;
use crate::track::TrackId;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

impl Display for PlayerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Playing => write!(f, "⏸"),
            Self::Paused => write!(f, "►"),
            Self::Stopped => write!(f, "⏹"),
        }
    }
}

pub struct Player {
    pub status: PlayerStatus,
    pub current_track_id: Option<TrackId>,
    pub shuffled_track_ids: Vec<TrackId>,
    pub timer: Timer,
    pub shuffle: bool,
    playback_finished: Arc<AtomicBool>,
    player: rodio::Player,
    _handle: MixerDeviceSink,
}

impl Player {
    pub fn new() -> eyre::Result<Self> {
        let handle = DeviceSinkBuilder::open_default_sink()?;
        let player = rodio::Player::connect_new(handle.mixer());

        Ok(Self {
            status: PlayerStatus::Stopped,
            current_track_id: None,
            shuffled_track_ids: Vec::new(),
            timer: Timer::new(),
            shuffle: false,
            playback_finished: Arc::new(AtomicBool::new(false)),
            player,
            _handle: handle,
        })
    }

    pub fn play(&mut self, path: &Path) -> eyre::Result<()> {
        let file = File::open(path)?;
        let source = Decoder::try_from(file)?;

        let playback_finished = Arc::clone(&self.playback_finished);
        let empty_callback = EmptyCallback::new(Box::new(move || {
            playback_finished.store(true, Ordering::Release);
        }));

        self.playback_finished.store(false, Ordering::Relaxed);
        self.timer.reset();

        self.player.clear();
        self.player.append(source);
        self.player.append(empty_callback);
        self.player.play();

        self.timer.start();

        self.status = PlayerStatus::Playing;

        Ok(())
    }

    pub fn toggle(&mut self) {
        if self.player.is_paused() {
            self.player.play();
            self.timer.start();
            self.status = PlayerStatus::Playing;
        } else {
            self.player.pause();
            self.timer.stop();
            self.status = PlayerStatus::Paused;
        }
    }

    pub fn set_track(&mut self, id: Option<TrackId>) {
        self.current_track_id = id;
    }

    pub fn volume(&self) -> f32 {
        (self.player.volume() * 100.0).round()
    }

    pub fn volume_up(&self) {
        let value = (self.player.volume() + 0.05).clamp(0.0, 1.0);
        self.player.set_volume(value);
    }

    pub fn volume_down(&self) {
        let value = (self.player.volume() - 0.05).clamp(0.0, 1.0);
        self.player.set_volume(value);
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn progress(&self, duration: u64) -> f64 {
        (f64::from(self.timer.elapsed()) / duration as f64).clamp(0.0, 1.0)
    }

    pub fn check_playback_completion(&mut self) {
        if self.playback_finished.load(Ordering::Acquire) {
            self.timer.stop();
            self.status = PlayerStatus::Stopped;
        }
    }

    pub fn shuffle_track_ids(&mut self, track_ids: &[TrackId]) {
        let mut rng = rand::rng();
        let mut track_ids = track_ids.to_owned();
        track_ids.shuffle(&mut rng);
        self.shuffled_track_ids = track_ids;
    }

    pub fn set_shuffle(&mut self, value: bool) {
        self.shuffle = value;
    }
}
