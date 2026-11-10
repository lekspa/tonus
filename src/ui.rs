pub mod color;
pub mod constraints;
pub mod format;
mod keymaps;
pub mod theme;
pub mod truncate;

use crate::config::DEFAULT_CONFIG_LOCATION;
use crate::library::SUPPORTED_CODECS;
use crate::player::Repeat;
use crate::ui::truncate::truncate;
use crate::{app::App, track::Track};
use crate::{app::Mode, player::PlayerStatus};
use crate::{app::View, config::config_file_path};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Clear, Gauge, HighlightSpacing, List, Padding, Paragraph, Row, Table,
    },
};
use unicode_width::UnicodeWidthStr;

/// Minimum terminal height to render all widgets
const MIN_HEIGHT: u16 = 24;
/// Minimum terminal width to render all widgets
const MIN_WIDTH: u16 = 80;
/// Minimum terminal width to render the full layout
const BREAKPOINT: u16 = 100;
/// Default block padding
const PADDING: u16 = 2;

const DELIMITER: &str = "•";
const SEARCH_PREFIX: &str = "/";
// Last label should be shifted to right respecting the padding
const TABLE_HEADER: [&str; 4] = ["Title", "Artist", "Album", "  Time"];

/// Returns a list containing width for every library widget column.
/// Block padding (2 cells) and column padding (2 cells) are excluded.
pub fn compute_column_widths(width: u16) -> Vec<usize> {
    let area = Rect::new(0, 0, width - PADDING, 1);
    let constraints = library_constraints(width);
    Layout::horizontal(constraints)
        .split(area)
        .iter()
        .map(|rect| rect.width.saturating_sub(2) as usize)
        .collect()
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let theme = app.config.theme();
    let area = frame.area();
    let rects = Layout::vertical(constraints::APP).split(area);
    let mut block = Block::new().bg(theme.background).fg(theme.foreground);

    if app.view == View::Keymaps {
        block = block.add_modifier(Modifier::DIM);
    }

    frame.render_widget(block, area);

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_terminal_size(frame, app);
        return;
    }

    if app.mode == Mode::Search {
        render_search(app, frame, rects[0]);
    } else {
        render_status_line(app, frame, rects[0]);
    }

    if app.library.is_empty() {
        render_placeholder(frame, rects[1]);
    } else {
        render_library(app, frame, rects[1]);
    }

    if app.view == View::Keymaps {
        render_keymaps(app, frame);
    }

    render_playback(app, frame, rects[2]);
}

pub fn render_welcome_screen(frame: &mut Frame, app: &App) {
    let theme = app.config.theme();
    let area = frame.area();
    let centered_area = area.centered_vertically(Constraint::Length(6));
    let block = Block::new().bg(theme.background).fg(theme.foreground);

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_terminal_size(frame, app);
        return;
    }

    let path = config_file_path().unwrap_or_else(|| DEFAULT_CONFIG_LOCATION.into());

    let lines = vec![
        Line::from("Music directory is not available").bold(),
        Line::from("Configure it to point to an existing path:").bold(),
        Line::from("music_directory = \"/path/to/music\"".fg(theme.accent)).bold(),
        Line::from(path.display().to_string()),
        Line::from(""),
        Line::from(vec![
            "[".dim(),
            "q".fg(theme.accent).bold(),
            "]".dim(),
            " Quit".bold(),
        ]),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);

    frame.render_widget(block, area);
    frame.render_widget(paragraph, centered_area);
}

fn render_status_line(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::new().padding(Padding::horizontal(PADDING));
    let rects = Layout::horizontal(constraints::STATUS_LINE).split(block.inner(area));

    let theme = app.config.theme();

    let duration = if area.width < BREAKPOINT {
        format::total_duration_compact(app.library.total_duration)
    } else {
        format::total_duration(app.library.total_duration)
    };

    let len = app.library.len();
    let visible_len = app.visible_track_ids().len();

    let mut spans = Vec::new();

    spans.push(Span::styled(
        "Library: ",
        Style::new().bold().fg(theme.accent),
    ));
    spans.push(Span::raw(format!("{len} tracks {DELIMITER} {duration}")));

    if let Some(s) = app.table_state.selected() {
        spans.push(Span::raw(format!(" {DELIMITER} ")));
        spans.push(Span::raw(format!("{} of {}", s + 1, visible_len)));
    }

    if !app.search.is_empty() {
        spans.push(Span::raw(format!(" {DELIMITER} ")));
        // Compute available space for rendering the search query.
        // Truncate the string if there is not enough space.
        let spacing = 1;
        let width: usize = spans.iter().map(UnicodeWidthStr::width).sum();
        let free_space = rects[0].width as usize - width - spacing;

        spans.push(Span::raw(truncate(
            &format!("{}{}", SEARCH_PREFIX, app.search.query()),
            free_space,
        )));
    }

    let line = Line::from(spans);
    let keymaps = Span::styled("Keymaps: ?", Style::new().fg(theme.accent).bold());

    frame.render_widget(Paragraph::new(line), rects[0]);
    frame.render_widget(keymaps, rects[1]);
}

fn table_row<'a>(index: usize, track: &Track, app: &App) -> Row<'a> {
    let theme = app.config.theme();
    let cells = [
        truncate(&track.title, app.column_widths[0]),
        truncate(&track.artist, app.column_widths[1]),
        truncate(&track.album, app.column_widths[2]),
        format!("{:>6}", format::track_duration(track.duration)),
    ];

    let style = match app.index_of_track(app.player.current_track_id) {
        Some(position) if index == position => Style::new().fg(theme.accent),
        _ => Style::new(),
    };

    Row::new(cells).style(style)
}

fn render_placeholder(frame: &mut Frame, area: Rect) {
    let block = Block::bordered().border_type(BorderType::Rounded);
    let centered_area = area.centered_vertically(Constraint::Length(2));

    let lines = vec![
        Line::from("No supported audio files were found in configured music directory."),
        Line::from(vec![
            "Supported formats: ".into(),
            SUPPORTED_CODECS.join(", ").bold(),
            ".".into(),
        ]),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);

    frame.render_widget(block, area);
    frame.render_widget(paragraph, centered_area);
}

fn render_library(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();

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

    let table = Table::new(rows, library_constraints(area.width))
        .block(Block::bordered().border_type(BorderType::Rounded))
        .header(Row::new(TABLE_HEADER).bold().bg(theme.surface))
        .highlight_symbol(">")
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(Style::new().bg(theme.selection_background).fg(highlight_fg));

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_playback(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1));
    let rects = Layout::vertical(constraints::PLAYBACK).split(block.inner(area));
    let track = app.current_track();

    frame.render_widget(block, area);

    render_progress_gauge(track, app, frame, rects[0]);
    render_playback_data(track, app, frame, rects[1]);
}

fn render_progress_gauge(track: Option<&Track>, app: &App, frame: &mut Frame, area: Rect) {
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

fn render_playback_data(track: Option<&Track>, app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();
    let rects = Layout::horizontal(constraints::PLAYBACK_DATA).split(area);

    let status = Paragraph::new(current_track_text(app.player.status, track));

    let repeat_style = match app.player.repeat {
        Repeat::Off => Style::new().dim(),
        _ => Style::new().fg(theme.accent),
    };

    let shuffle_style = if app.player.shuffle {
        Style::new().fg(theme.accent)
    } else {
        Style::new().dim()
    };

    let line = Line::from(vec![
        Span::styled(format!("{}", app.player.repeat), repeat_style),
        Span::raw(" "),
        Span::styled("S", shuffle_style),
        Span::raw(format!(" {DELIMITER} ")),
        Span::raw(format!("Volume:{:>3}%", app.player.volume())),
    ]);

    frame.render_widget(status, rects[0]);
    frame.render_widget(line, rects[1]);
}

fn render_keymaps(app: &mut App, frame: &mut Frame) {
    let theme = app.config.theme();
    let area = frame.area();
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" Keymaps ".bold())
        .bg(theme.background)
        .padding(Padding::horizontal(1));
    let area = area.centered(Constraint::Length(60), Constraint::Percentage(50));

    frame.render_widget(Clear, area);

    let mut lines = Vec::<Line>::new();

    lines.push(Line::from(vec![
        "             ".into(),
        "─── ".fg(theme.accent),
        "General".bold(),
    ]));

    for (key, desc) in keymaps::GENERAL {
        lines.push(Line::from(vec![
            key.fg(theme.accent),
            " ".into(),
            desc.into(),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        "             ".into(),
        "─── ".fg(theme.accent),
        "Playback".bold(),
    ]));

    for (key, desc) in keymaps::PLAYBACK {
        lines.push(Line::from(vec![
            key.fg(theme.accent),
            " ".into(),
            desc.into(),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        "             ".into(),
        "─── ".fg(theme.accent),
        "Search".bold(),
    ]));

    for (key, desc) in keymaps::SEARCH {
        lines.push(Line::from(vec![
            key.fg(theme.accent),
            " ".into(),
            desc.into(),
        ]));
    }

    let style = Style::new()
        .bg(theme.selection_background)
        .fg(theme.selection_foreground);

    let list = List::new(lines).block(block).highlight_style(style);

    frame.render_stateful_widget(list, area, &mut app.keymaps_list_state);
}

fn render_search(app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.config.theme();
    // Terminal dimensions never exceed u16::MAX (65535 pixels)
    #[allow(clippy::cast_possible_truncation)]
    let offset = (SEARCH_PREFIX.len() as u16) + PADDING;
    let width = area.width.saturating_sub(offset + PADDING);
    let scroll = app.search.visual_scroll(width as usize);

    let text = Line::from(vec![
        Span::styled(SEARCH_PREFIX, Style::new().bold().fg(theme.accent)),
        Span::raw(app.search.query()),
    ]);

    let block = Block::new().padding(Padding::horizontal(PADDING));
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

fn render_terminal_size(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let centered_area = area.centered_vertically(Constraint::Length(5));

    let theme = app.config.theme();
    let status_color = |success: bool| {
        if success {
            theme.success
        } else {
            theme.failure
        }
    };

    let lines = vec![
        Line::from("Terminal is too small:"),
        Line::from(vec![
            "Width = ".into(),
            area.width.fg(status_color(area.width > MIN_WIDTH)),
            " ".into(),
            "Height = ".into(),
            area.height.fg(status_color(area.height > MIN_HEIGHT)),
        ]),
        Line::from(""),
        Line::from("Minimum required terminal size:"),
        Line::from(format!("Width = {MIN_WIDTH} Height = {MIN_HEIGHT}")),
    ];
    let paragraph = Paragraph::new(lines).alignment(Alignment::Center).bold();

    frame.render_widget(paragraph, centered_area);
}

fn library_constraints(width: u16) -> [Constraint; 4] {
    if width < BREAKPOINT {
        constraints::LIBRARY_MEDIUM
    } else {
        constraints::LIBRARY_FULL
    }
}
