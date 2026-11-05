use color_eyre::eyre::Result;
use rand::seq::SliceRandom;
use rodio::source::EmptyCallback;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink};
use std::fmt::Display;
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::warn;

use crate::timer::Timer;
use crate::track::{Track, TrackId};

const SEEK_STEP: u64 = 10;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayerStatus {
    Paused,
    Playing,
    Stopped,
}

pub enum Repeat {
    Off,
    All,
    One,
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

impl Display for Repeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off | Self::All => write!(f, "R "),
            Self::One => write!(f, "R1"),
        }
    }
}

pub struct Player {
    pub status: PlayerStatus,
    pub current_track_id: Option<TrackId>,
    pub shuffled_track_ids: Vec<TrackId>,
    pub timer: Timer,
    pub shuffle: bool,
    pub repeat: Repeat,
    playback_finished: Arc<AtomicBool>,
    player: rodio::Player,
    _handle: MixerDeviceSink,
}

impl Player {
    pub fn new() -> Result<Self> {
        let handle = DeviceSinkBuilder::open_default_sink()?;
        let player = rodio::Player::connect_new(handle.mixer());

        Ok(Self {
            status: PlayerStatus::Stopped,
            current_track_id: None,
            shuffled_track_ids: Vec::new(),
            timer: Timer::new(),
            shuffle: false,
            repeat: Repeat::Off,
            playback_finished: Arc::new(AtomicBool::new(false)),
            player,
            _handle: handle,
        })
    }

    pub fn is_playback_finished(&self) -> bool {
        self.playback_finished.load(Ordering::Relaxed)
    }

    pub fn play(&mut self, track: &Track) -> Result<()> {
        let file = File::open(&track.path)?;
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

        self.current_track_id = Some(track.id);
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
    pub fn progress(&self, duration: Duration) -> f64 {
        if duration.is_zero() {
            return 0.0;
        }
        (self.timer.elapsed().as_secs_f64() / duration.as_secs_f64()).clamp(0.0, 1.0)
    }

    pub fn on_playback_completion(&mut self) {
        self.playback_finished.store(false, Ordering::Release);
        self.timer.stop();
        self.timer.reset();
        self.status = PlayerStatus::Stopped;
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

    pub fn toggle_repeat(&mut self) {
        self.repeat = match self.repeat {
            Repeat::Off => Repeat::All,
            Repeat::All => Repeat::One,
            Repeat::One => Repeat::Off,
        }
    }

    /// Attempts to seek forward by 10s in `track`.
    pub fn seek_forward(&mut self, track: &Track) {
        self.seek(track, SEEK_STEP.cast_signed());
    }

    /// Attempts to seek backward by 10s in `track`.
    pub fn seek_backward(&mut self, track: &Track) {
        self.seek(track, -(SEEK_STEP.cast_signed()));
    }

    fn seek(&mut self, track: &Track, offset: i64) {
        if self.status != PlayerStatus::Playing {
            return;
        }

        let elapsed = self.timer.elapsed();
        let position = (elapsed.as_secs().cast_signed() + offset)
            .clamp(0, track.duration.as_secs().cast_signed())
            .cast_unsigned();

        match self.player.try_seek(Duration::from_secs(position)) {
            Ok(()) => self.timer.set_accumulated(Duration::from_secs(position)),
            Err(e) => warn!("failed seeking to {position}: {e:?}"),
        }
    }
}
