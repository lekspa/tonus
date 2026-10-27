use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Gauge, HighlightSpacing, LineGauge, Padding, Paragraph, Row, Table,
    },
};

use crate::config::config_file_path;
use crate::format;
use crate::player::PlayerStatus;
use crate::truncate::truncate;
use crate::{app::App, track::Track};

const PLAYBACK_HEIGHT: u16 = 4;
const VOLUME_WIDTH: u16 = 24;
const TRACK_DURATION_WIDTH: u16 = 7;
const LIBRARY_STATS_HEIGHT: u16 = 1;
const KEYMAPS_HEIGHT: u16 = 1;
const SHUFFLE_WIDTH: u16 = 15;
const REPEAT_WIDTH: u16 = 14;

const DELIMITER: &str = "•";

const APP_CONSTRAINTS: [Constraint; 4] = [
    Constraint::Length(LIBRARY_STATS_HEIGHT),
    Constraint::Fill(1),
    Constraint::Length(PLAYBACK_HEIGHT),
    Constraint::Length(KEYMAPS_HEIGHT),
];
const LIBRARY_CONSTRAINTS: [Constraint; 4] = [
    Constraint::Min(24),
    Constraint::Min(18),
    Constraint::Min(18),
    Constraint::Length(TRACK_DURATION_WIDTH),
];
const PLAYER_CONSTRAINTS: [Constraint; 2] = [Constraint::Ratio(1, 2); 2];
const CURRENT_TRACK_CONSTRAINTS: [Constraint; 4] = [
    Constraint::Fill(1),
    Constraint::Length(REPEAT_WIDTH),
    Constraint::Length(SHUFFLE_WIDTH),
    Constraint::Length(VOLUME_WIDTH),
];

const SEARCH_PREFIX: &str = "Search: ";

/// Computes track list table column widths exluding horizontal table borders(2 columns)
pub fn compute_column_widths(width: u16) -> Vec<usize> {
    let area = Rect::new(0, 0, width, 1);
    Layout::horizontal(LIBRARY_CONSTRAINTS)
        .split(area)
        .iter()
        .map(|rect| rect.width.saturating_sub(2) as usize)
        .collect()
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let theme = app.config.theme();
    let rects = Layout::vertical(APP_CONSTRAINTS).split(frame.area());
    let block = Block::new()
        .style(Style::new().bg(theme.background))
        .fg(theme.foreground);
    // Render an empty block to apply a global background
    frame.render_widget(block, frame.area());

    if app.search.is_active() {
        render_search(app, frame, rects[0]);
    } else {
        render_library_stats(app, frame, rects[0]);
    }

    if app.library.is_empty() {
        render_library_placeholder(app, frame, rects[1]);
    } else {
        render_library(app, frame, rects[1]);
    }

    render_player(app, frame, rects[2]);
    render_keymaps(app, frame, rects[3]);
}

pub fn render_welcome_screen(frame: &mut Frame, app: &App) {
    let theme = app.config.theme();
    let area = frame.area();
    let centered_area = area.centered_vertically(Constraint::Length(7));
    let block = Block::bordered().border_type(BorderType::Rounded);
    let config_file = config_file_path().unwrap_or_else(|| "~/.config/sonus/config.toml".into());

    let lines = vec![
        Line::from(Span::styled(
            "Welcome to Sonus!",
            Style::new().bold().fg(theme.accent),
        )),
        Line::from(""),
        Line::from("Player could not find your music directory."),
        Line::from(vec![
            Span::raw("Add this to your config: "),
            Span::styled(
                "music_directory = \"/path/to/music\"",
                Style::new().bold().fg(theme.accent),
            ),
            Span::raw("."),
        ]),
        Line::from(vec![
            Span::raw("Config file: "),
            Span::styled(config_file.display().to_string(), Style::new().italic()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[q]", Style::new().bold()),
            Span::raw(" Quit"),
        ]),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);

    frame.render_widget(block, area);
    frame.render_widget(paragraph, centered_area);
}

fn render_library_stats(app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();
    let block = Block::new().padding(Padding::horizontal(2));
    let total = format::total_duration(app.library.total_duration);
    let len = app.library.len();
    let visible_len = app.visible_track_ids().len();

    let mut spans = Vec::new();

    spans.push(Span::styled(
        "Library: ",
        Style::new().bold().fg(theme.accent),
    ));
    spans.push(Span::raw(format!("{len} tracks {DELIMITER} {total} total")));

    if let Some(s) = app.table_state.selected() {
        spans.push(Span::raw(format!(" {DELIMITER} ")));
        spans.push(Span::raw(format!("selected {}/{}", s + 1, visible_len)));
    }

    if !app.search.is_empty() {
        spans.push(Span::raw(format!(" {DELIMITER} ")));
        spans.push(Span::raw(format!("searching: \"{}\"", app.search.query())));
    }

    let text = Line::from(spans);
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn table_row<'a>(index: usize, track: &Track, app: &App) -> Row<'a> {
    let theme = app.config.theme();
    let cells = [
        truncate(&track.title, app.column_widths[0]),
        truncate(&track.artist, app.column_widths[1]),
        truncate(&track.album, app.column_widths[2]),
        format!("{:>6}", format::duration(track.duration)),
    ];

    let style = match app.index_of_track(app.player.current_track_id) {
        Some(position) if index == position => Style::new().fg(theme.accent),
        _ => Style::new(),
    };

    Row::new(cells).style(style)
}

fn render_library_placeholder(_app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::bordered().border_type(BorderType::Rounded);
    let centered_area = area.centered_vertically(Constraint::Length(2));

    let lines = vec![
        Line::from("No supported audio files were found in configured music directory."),
        Line::from("Supported formats: mp3, mp4, wav, ogg, flac."),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center).bold();

    frame.render_widget(block, area);
    frame.render_widget(paragraph, centered_area);
}

fn render_library(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();
    let header_style = Style::new().bold().bg(theme.surface);
    let header = Row::new(["Title", "Artist", "Album", "Length"]).style(header_style);

    let highlight_fg = if app.selected_track_id == app.player.current_track_id {
        theme.accent
    } else {
        theme.selection_foreground
    };

    let rows = app
        .visible_track_ids()
        .iter()
        .enumerate()
        .filter_map(|(idx, &id)| app.library.get_track(id).map(|t| (idx, t)))
        .map(|(index, track)| table_row(index, track, app));

    let table = Table::new(rows, LIBRARY_CONSTRAINTS)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .header(header)
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(Style::new().bg(theme.selection_background).fg(highlight_fg));

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_player(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1));
    let rects = Layout::vertical(PLAYER_CONSTRAINTS).split(block.inner(area));
    let track = app.current_track();

    frame.render_widget(block, area);

    render_progress(track, app, frame, rects[0]);
    render_current_track(track, app, frame, rects[1]);
}

fn render_progress(track: Option<&Track>, app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();
    let elapsed = app.player.timer.elapsed();
    let (ratio, label) = track.map_or_else(
        || (0.0, String::from("0:00/0:00")),
        |t| {
            (
                app.player.progress(t.duration),
                format::progress(elapsed, t.duration),
            )
        },
    );

    let style = Style::new().fg(theme.on_accent).bold();

    let progress = Gauge::default()
        .ratio(ratio)
        .use_unicode(true)
        .label(Span::styled(label, style))
        .gauge_style(Style::new().fg(theme.accent).bg(theme.surface));

    frame.render_widget(progress, area);
}

fn render_current_track(track: Option<&Track>, app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();
    let rects = Layout::horizontal(CURRENT_TRACK_CONSTRAINTS)
        .flex(Flex::SpaceBetween)
        .split(area);

    let status = Paragraph::new(current_track_text(app.player.status, track));

    let repeat = Span::raw(format!("Repeat: {} {DELIMITER} ", app.player.repeat));

    let shuffle = Span::raw(format!(
        "Shuffle: {} {DELIMITER}",
        if app.player.shuffle { "on " } else { "off" }
    ));

    let volume = LineGauge::default()
        .label(format!("Volume: {}%", app.player.volume()))
        .filled_style(Style::new().fg(theme.accent))
        .filled_symbol(symbols::line::HORIZONTAL)
        .ratio(f64::from(app.player.volume()) / 100.0);

    frame.render_widget(status, rects[0]);
    frame.render_widget(repeat, rects[1]);
    frame.render_widget(shuffle, rects[2]);
    frame.render_widget(volume, rects[3]);
}

fn render_keymaps(app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();
    let general_keymaps = vec![
        ("j/k", "Navigate"),
        ("n/p", "Next/Prev"),
        ("Enter", "Play"),
        ("Space", "Toggle"),
        ("+/-", "Volume"),
        ("s", "Shuffle"),
        ("r", "Repeat"),
        ("←/→", "Seek"),
        ("/", "Search"),
        ("q", "Quit"),
    ];

    let search_keymaps = vec![
        ("Esc", "Exit"),
        ("Enter", "Confirm"),
        ("←/→", "Move cursor"),
    ];

    let block = Block::new().padding(Padding::horizontal(2));

    let keymaps = if app.search.is_active() {
        search_keymaps
    } else {
        general_keymaps
    };

    let mut spans = Vec::new();

    for (index, &(keys, label)) in keymaps.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" | "));
        }
        spans.push(Span::styled("[", Style::new().dim()));
        spans.push(Span::styled(keys, Style::new().fg(theme.accent).bold()));
        spans.push(Span::styled("] ", Style::new().dim()));
        spans.push(Span::raw(label));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

fn render_search(app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();
    // Terminal dimensions never exceed u16::MAX (65535 pixels)
    let block_padding = 2;
    #[allow(clippy::cast_possible_truncation)]
    let offset = (SEARCH_PREFIX.len() as u16) + block_padding;
    let width = area.width.saturating_sub(offset + block_padding);
    let scroll = app.search.visual_scroll(width as usize);

    let text = Line::from(vec![
        Span::styled(SEARCH_PREFIX, Style::new().bold().fg(theme.accent)),
        Span::raw(app.search.query()),
    ]);

    let block = Block::new().padding(Padding::horizontal(2));
    // Scroll is bounded by terminal width, which fits in u16
    #[allow(clippy::cast_possible_truncation)]
    let input = Paragraph::new(text).scroll((0, scroll as u16)).block(block);

    frame.render_widget(input, area);

    // Position the cursor past the end of the input text
    let x = app.search.visual_cursor().saturating_sub(scroll) + offset as usize;
    // Cursor coordinate fits in u16 (same bound as terminal area.x)
    #[allow(clippy::cast_possible_truncation)]
    frame.set_cursor_position((area.x + x as u16, area.y));
}

fn current_track_text(status: PlayerStatus, track: Option<&Track>) -> String {
    track.map_or_else(
        || String::from("○ Select a track to play"),
        |t| format!("{} {} - {}", status, t.title, t.artist),
    )
}
