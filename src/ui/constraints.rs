use ratatui::layout::Constraint;

const PLAYER_HEIGHT: u16 = 4;
const LIBRARY_STATS_HEIGHT: u16 = 1;

const VOLUME_WIDTH: u16 = 24;
const SHUFFLE_WIDTH: u16 = 15;
const REPEAT_WIDTH: u16 = 14;
const TRACK_TITLE_WIDTH: u16 = 24;
const TRACK_ARTIST_WIDTH: u16 = 18;
const TRACK_ALBUM_WIDTH: u16 = 18;
const TRACK_DURATION_WIDTH: u16 = 7;
const KEYMAPS_HINT_WIDTH: u16 = 10;

pub const APP: [Constraint; 3] = [
    Constraint::Length(LIBRARY_STATS_HEIGHT),
    Constraint::Fill(1),
    Constraint::Length(PLAYER_HEIGHT),
];
pub const LIBRARY: [Constraint; 4] = [
    Constraint::Min(TRACK_TITLE_WIDTH),
    Constraint::Min(TRACK_ARTIST_WIDTH),
    Constraint::Min(TRACK_ALBUM_WIDTH),
    Constraint::Length(TRACK_DURATION_WIDTH),
];
pub const PLAYER: [Constraint; 2] = [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)];
pub const CURRENT_TRACK: [Constraint; 4] = [
    Constraint::Fill(1),
    Constraint::Length(REPEAT_WIDTH),
    Constraint::Length(SHUFFLE_WIDTH),
    Constraint::Length(VOLUME_WIDTH),
];
pub const STATUS_LINE: [Constraint; 2] =
    [Constraint::Fill(1), Constraint::Length(KEYMAPS_HINT_WIDTH)];
