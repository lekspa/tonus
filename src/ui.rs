use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::Style,
    symbols,
    text::Span,
    widgets::{
        Block, BorderType, Gauge, HighlightSpacing, LineGauge, Padding, Paragraph, Row, Table,
    },
};

use crate::colors;
use crate::format;
use crate::player::PlayerStatus;
use crate::truncate::truncate;
use crate::{app::App, track::Track};

const PLAYBACK_HEIGHT: u16 = 4;
const VOLUME_WIDTH: u16 = 24;
const TRACK_DURATION_WIDTH: u16 = 7;
const LIBRARY_STATS_HEIGHT: u16 = 1;

const APP_CONSTRAINTS: [Constraint; 3] = [
    Constraint::Length(LIBRARY_STATS_HEIGHT),
    Constraint::Min(0),
    Constraint::Length(PLAYBACK_HEIGHT),
];
const LIBRARY_CONSTRAINTS: [Constraint; 4] = [
    Constraint::Min(24),
    Constraint::Min(18),
    Constraint::Min(18),
    Constraint::Length(TRACK_DURATION_WIDTH),
];
const PLAYER_CONSTRAINTS: [Constraint; 2] = [Constraint::Ratio(1, 2); 2];
const CURRENT_TRACK_CONSTRAINTS: [Constraint; 2] =
    [Constraint::Fill(1), Constraint::Length(VOLUME_WIDTH)];

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
    let rects = Layout::vertical(APP_CONSTRAINTS).split(frame.area());
    let block = Block::new().style(Style::new().bg(colors::PRIMARY_BG));
    // Render an empty block to apply a global background
    frame.render_widget(block, frame.area());

    render_library_stats(frame, app, rects[0]);
    render_library(app, frame, rects[1]);
    render_player(app, frame, rects[2]);
}

fn render_library_stats(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::new().padding(Padding::horizontal(2));
    let total = format::total_duration(app.library.total_duration);
    let text = format!("Library: {} tracks • {} total", app.library.len(), total);

    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn table_row<'a>(index: usize, track: &Track, app: &App) -> Row<'a> {
    let cells = [
        truncate(&track.title, app.column_widths[0]),
        truncate(&track.artist, app.column_widths[1]),
        truncate(&track.album, app.column_widths[2]),
        format!("{:>6}", format::duration(track.duration)),
    ];

    let style = match app.player.current_track_id {
        Some(id) if index == id => Style::new().fg(colors::FIERY_ORANGE),
        Some(_) | None => Style::new(),
    };

    Row::new(cells).style(style)
}

fn render_library(app: &mut App, frame: &mut Frame, area: Rect) {
    let header_style = Style::new().bold().bg(colors::HEADER_BG);
    let header = Row::new(["Title", "Artist", "Album", "Length"]).style(header_style);

    let highlight_fg = if app.table_state.selected() == app.player.current_track_id {
        colors::FIERY_ORANGE
    } else {
        colors::HIGHLIGHT_FG
    };

    let rows = app
        .library
        .tracks
        .iter()
        .enumerate()
        .map(|(index, track)| table_row(index, track, app));

    let table = Table::new(rows, LIBRARY_CONSTRAINTS)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .header(header)
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(Style::new().bg(colors::HIGHLIGHT_BG).fg(highlight_fg));

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
    let (ratio, progress_label) = match track {
        Some(t) => (
            app.player.progress(t.duration),
            format::progress(app.player.timer.elapsed().into(), t.duration),
        ),
        None => (0.0, String::from("0:00/0:00")),
    };

    let style = Style::new().fg(colors::HIGHLIGHT_BG).bold();

    let progress = Gauge::default()
        .ratio(ratio)
        .use_unicode(true)
        .label(Span::styled(progress_label, style))
        .gauge_style(Style::new().fg(colors::FIERY_ORANGE).bg(colors::HEADER_BG));

    frame.render_widget(progress, area);
}

fn render_current_track(track: Option<&Track>, app: &App, frame: &mut Frame, area: Rect) {
    let rects = Layout::horizontal(CURRENT_TRACK_CONSTRAINTS)
        .flex(Flex::SpaceBetween)
        .split(area);

    let status = Paragraph::new(current_track_text(app.player.status, track));

    let volume = LineGauge::default()
        .label(format!("Volume {}%", app.player.volume()))
        .filled_style(Style::new().fg(colors::LIGHTNING_YELLOW))
        .filled_symbol(symbols::line::HORIZONTAL)
        .ratio(f64::from(app.player.volume()) / 100.0);

    frame.render_widget(status, rects[0]);
    frame.render_widget(volume, rects[1]);
}

fn current_track_text(status: PlayerStatus, track: Option<&Track>) -> String {
    match track {
        Some(t) => format!("{} {} - {}", status, t.title, t.artist),
        None => String::from("○ Select a track to play"),
    }
}
