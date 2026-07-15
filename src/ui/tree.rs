use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ansi;
use crate::app::AppState;
use crate::render::build_sidebar;
use crate::ui::{ansi_lines_to_text, ansi_to_line};

// ═══════════════════════════════════════════════════════════════════════════
// Tree View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_tree_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let sidebar_w = if app.term_w >= 80 { 28 } else { 0 };

    if sidebar_w > 0 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(sidebar_w),
            ])
            .split(area);

        // Tree content
        render_tree_content(frame, chunks[0], app);

        // Vertical separator
        let sep_line = format!("{}│{}", ansi::DIM, ansi::RST);
        let sep_para = Paragraph::new(Text::from(vec![ansi_to_line(&sep_line)]))
            .style(Style::default().bg(Color::Black));
        frame.render_widget(sep_para, chunks[1]);

        // Sidebar with block border
        let sidebar_area = chunks[2];
        let sidebar_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM));
        let inner_area = sidebar_block.inner(sidebar_area);
        frame.render_widget(sidebar_block, sidebar_area);

        let vis = (inner_area.height as usize).max(1);
        let sidebar_lines = build_sidebar(app, vis);
        let sidebar_text = ansi_lines_to_text(&sidebar_lines);
        let sidebar_para = Paragraph::new(sidebar_text).style(Style::default().bg(Color::Black));
        frame.render_widget(sidebar_para, inner_area);
    } else {
        render_tree_content(frame, area, app);
    }
}

pub(crate) fn render_tree_content(frame: &mut Frame, area: Rect, app: &AppState) {
    let render_n = app.render_lines.len();
    let vis = (area.height as usize).max(1);
    let max_scroll = render_n.saturating_sub(vis);

    // Auto-scroll to keep cursor visible
    let mut scroll = app.scroll;
    if app.cursor >= 0 {
        if (app.cursor as usize) < scroll {
            scroll = app.cursor as usize;
        }
        if app.cursor as usize >= scroll + vis {
            scroll = (app.cursor as usize).saturating_sub(vis - 1);
        }
    }
    scroll = scroll.min(max_scroll);

    let tree_w = area.width as usize;
    let cursor_bg = Style::default().bg(Color::DarkGray);

    // Build styled lines for rendering
    let mut ratatui_lines: Vec<Line<'static>> = Vec::new();
    let mut drawn = 0;

    // Scroll up indicator
    if scroll > 0 {
        let remaining = scroll;
        let indicator = format!("  {}↑ {} more{}", ansi::DIM, remaining, ansi::RST);
        ratatui_lines.push(ansi_to_line(&indicator));
        drawn += 1;
    }

    for idx in scroll..render_n {
        if drawn >= vis {
            break;
        }
        let content = &app.render_lines[idx].content;
        let truncated = ansi::truncate_vis(content, tree_w);
        let mut line = ansi_to_line(&truncated);

        if idx == app.cursor as usize {
            // Full line cursor highlighting with background color
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

        ratatui_lines.push(line);
        drawn += 1;
    }

    // Scroll down indicator
    if scroll + drawn < render_n {
        let remaining = render_n - scroll - drawn;
        let indicator = format!("  {}↓ {} more{}", ansi::DIM, remaining, ansi::RST);
        ratatui_lines.push(ansi_to_line(&indicator));
        drawn += 1;
    }

    // Fill remaining area with empty lines
    for _ in drawn..vis {
        ratatui_lines.push(Line::raw(""));
    }

    let text = Text::from(ratatui_lines);
    let paragraph = Paragraph::new(text).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

