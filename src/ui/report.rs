use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};
use crate::app::AppState;
use crate::ui::ansi_to_line;

// ═══════════════════════════════════════════════════════════════════════════
// Report View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_report_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let vis = (area.height as usize).max(1);
    let total = app.report_lines.len();
    let max_s = total.saturating_sub(vis);
    let scroll = app.report_scroll.min(max_s);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut drawn = 0;
    for idx in scroll..total {
        if drawn >= vis {
            break;
        }
        lines.push(ansi_to_line(&app.report_lines[idx]));
        drawn += 1;
    }
    for _ in drawn..vis {
        lines.push(Line::raw(""));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

