use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

use crate::ansi;
use crate::app::AppState;
use crate::ui::ansi_to_line;

// ═══════════════════════════════════════════════════════════════════════════
// Gists View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_gists_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let vis = (area.height as usize).max(1);
    let total = app.gists_lines.len();
    let max_s = total.saturating_sub(vis);

    // Auto-scroll to keep cursor visible
    let mut scroll = app.gists_scroll;
    if app.gists_cursor < scroll {
        scroll = app.gists_cursor;
    }
    if app.gists_cursor >= scroll + vis {
        scroll = app.gists_cursor.saturating_sub(vis - 1);
    }
    scroll = scroll.min(max_s);

    let cursor_bg = Style::default().bg(Color::DarkGray);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut drawn = 0;

    // Scroll up indicator
    if scroll > 0 {
        let indicator = format!("  {}↑ {} more{}", ansi::DIM, scroll, ansi::RST);
        lines.push(ansi_to_line(&indicator));
        drawn += 1;
    }

    for idx in scroll..total {
        if drawn >= vis {
            break;
        }
        let mut line = ansi_to_line(&app.gists_lines[idx]);
        if idx == app.gists_cursor {
            // Cursor highlighting with background color
            let mut new_spans = vec![Span::styled(
                "▸ ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )];
            for span in line.spans.iter() {
                new_spans.push(Span::styled(
                    span.content.clone(),
                    span.style.patch(cursor_bg),
                ));
            }
            line = Line::from(new_spans);
        }
        lines.push(line);
        drawn += 1;
    }

    // Scroll down indicator
    if scroll + drawn < total {
        let remaining = total - scroll - drawn;
        let indicator = format!("  {}↓ {} more{}", ansi::DIM, remaining, ansi::RST);
        lines.push(ansi_to_line(&indicator));
        drawn += 1;
    }

    for _ in drawn..vis {
        lines.push(Line::raw(""));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════
// Gist Content View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_gist_content_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let vis = (area.height as usize).max(1);
    let total = app.gist_content_lines.len();
    let max_s = total.saturating_sub(vis);
    let scroll = app.gist_content_scroll.min(max_s);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut drawn = 0;

    // Scroll up indicator
    if scroll > 0 {
        let indicator = format!("  {}↑ {} more{}", ansi::DIM, scroll, ansi::RST);
        lines.push(ansi_to_line(&indicator));
        drawn += 1;
    }

    for idx in scroll..total {
        if drawn >= vis {
            break;
        }
        lines.push(ansi_to_line(&app.gist_content_lines[idx]));
        drawn += 1;
    }

    // Scroll down indicator
    if scroll + drawn < total {
        let remaining = total - scroll - drawn;
        let indicator = format!("  {}↓ {} more{}", ansi::DIM, remaining, ansi::RST);
        lines.push(ansi_to_line(&indicator));
        drawn += 1;
    }

    for _ in drawn..vis {
        lines.push(Line::raw(""));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

