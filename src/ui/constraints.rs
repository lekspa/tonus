use ratatui::layout::Constraint;

const PLAYER_HEIGHT: u16 = 4;
const STATUS_LINE_HEIGHT: u16 = 1;

const TRACK_TITLE_WIDTH: u16 = 24;
const TRACK_ARTIST_WIDTH: u16 = 18;
const TRACK_ALBUM_WIDTH: u16 = 18;
const TRACK_DURATION_WIDTH: u16 = 7;
const KEYMAPS_HINT_WIDTH: u16 = 10;
const PLAYBACK_CONTROLS_WIDTH: u16 = 18;

pub const APP: [Constraint; 3] = [
    Constraint::Length(STATUS_LINE_HEIGHT),
    Constraint::Fill(1),
    Constraint::Length(PLAYER_HEIGHT),
];
pub const LIBRARY_FULL: [Constraint; 4] = [
    Constraint::Min(TRACK_TITLE_WIDTH),
    Constraint::Min(TRACK_ARTIST_WIDTH),
    Constraint::Min(TRACK_ALBUM_WIDTH),
    Constraint::Length(TRACK_DURATION_WIDTH),
];
pub const LIBRARY_MEDIUM: [Constraint; 4] = [
    Constraint::Fill(4),
    Constraint::Fill(3),
    Constraint::Length(0),
    Constraint::Length(TRACK_DURATION_WIDTH),
];
pub const PLAYBACK: [Constraint; 2] = [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)];
pub const PLAYBACK_DATA: [Constraint; 2] = [
    Constraint::Fill(1),
    Constraint::Length(PLAYBACK_CONTROLS_WIDTH),
];
pub const STATUS_LINE: [Constraint; 2] =
    [Constraint::Fill(1), Constraint::Length(KEYMAPS_HINT_WIDTH)];
