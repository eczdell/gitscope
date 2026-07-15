use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

use crate::ansi;
use crate::app::AppState;
use crate::ui::ansi_to_line;

// ═══════════════════════════════════════════════════════════════════════════
// Diff View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_diff_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let vis = (area.height as usize).max(1);
    let total = app.diff_lines.len();
    let max_s = total.saturating_sub(vis);
    let scroll = app.diff_scroll.min(max_s);

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
        let line_str = &app.diff_lines[idx];
        let styled_line = if line_str.contains("diff --git") {
            ansi_to_line(&format!("{}{}{}", ansi::BLD, ansi::LGR, line_str))
        } else if line_str.contains("@@") {
            ansi_to_line(&format!("{}{}{}", ansi::BLD, ansi::CYN, line_str))
        } else if (line_str.contains('+') && !line_str.contains("+++"))
            || (line_str.starts_with('+') && !line_str.starts_with("+++"))
        {
            ansi_to_line(&format!("{}{}{}", ansi::LGR, line_str, ansi::RST))
        } else if (line_str.contains('-') && !line_str.contains("---"))
            || (line_str.starts_with('-') && !line_str.starts_with("---"))
        {
            ansi_to_line(&format!("{}{}{}", ansi::LRE, line_str, ansi::RST))
        } else {
            ansi_to_line(line_str)
        };
        lines.push(styled_line);
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

