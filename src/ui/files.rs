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
// Files View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_files_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let vis = (area.height as usize).max(1);
    let total = app.files_lines.len();
    let max_s = total.saturating_sub(vis);
    let scroll = app.files_scroll.min(max_s);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut drawn = 0;
    for idx in scroll..total {
        if drawn >= vis {
            break;
        }
        let line_str = &app.files_lines[idx];
        let styled_line = if line_str.contains('│') {
            ansi_to_line(&format!("  {}{}{}", ansi::DIM, line_str, ansi::RST))
        } else if line_str.contains("file changed") || line_str.contains("files changed") {
            ansi_to_line(&format!("{}{}{}", ansi::BLD, line_str, ansi::RST))
        } else {
            ansi_to_line(&format!("  {}", line_str))
        };
        lines.push(styled_line);
        drawn += 1;
    }
    for _ in drawn..vis {
        lines.push(Line::raw(""));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

