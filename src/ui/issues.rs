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
// Issues View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_issues_view(frame: &mut Frame, area: Rect, app: &AppState) {
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

        // Issues content
        render_issues_content(frame, chunks[0], app);

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
        render_issues_content(frame, area, app);
    }
}

pub(crate) fn render_issues_content(frame: &mut Frame, area: Rect, app: &AppState) {
    let vis = (area.height as usize).max(1);
    let total = app.issues_lines.len();
    let max_s = total.saturating_sub(vis);

    // Auto-scroll to keep cursor visible
    let mut scroll = app.issues_scroll;
    if app.issues_cursor < scroll {
        scroll = app.issues_cursor;
    }
    if app.issues_cursor >= scroll + vis {
        scroll = app.issues_cursor.saturating_sub(vis - 1);
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
        let mut line = ansi_to_line(&app.issues_lines[idx]);
        if idx == app.issues_cursor {
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
// Issue Create View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_issue_create_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Header
    lines.push(ansi_to_line(&format!(
        "  {}Create a new issue   {}({}){}",
        ansi::BLD, ansi::DIM, app.repo_name, ansi::RST
    )));
    lines.push(ansi_to_line(&format!(
        "  {}─────────────────────{}",
        ansi::DIM, ansi::RST
    )));
    lines.push(Line::raw(""));

    // Title input
    let title_cursor = if app.issue_create_focus == 0 { "█" } else { " " };
    lines.push(ansi_to_line(&format!(
        "  {}Title:{} {} {}",
        ansi::BLD, ansi::RST,
        app.issue_create_title, title_cursor,
    )));
    if app.issue_create_focus == 0 {
        lines.push(ansi_to_line(&format!(
            "  {}  {}",
            ansi::DIM, ansi::RST
        )));
    }

    // Body input
    lines.push(Line::raw(""));
    if app.issue_create_focus == 1 {
        lines.push(ansi_to_line(&format!(
            "  {}Body:{} (optional){}",
            ansi::BLD, ansi::DIM, ansi::RST
        )));
        if !app.issue_create_body.is_empty() {
            for body_line in app.issue_create_body.lines().take(5) {
                lines.push(ansi_to_line(&format!("  {}", body_line)));
            }
        } else {
            lines.push(ansi_to_line(&format!(
                "  {}  {}",
                ansi::DIM, ansi::RST
            )));
        }
    } else {
        lines.push(ansi_to_line(&format!(
            "  {}Body:{} {}",
            ansi::BLD, ansi::RST,
            if app.issue_create_body.is_empty() { "(optional)".to_string() } else { app.issue_create_body.clone() }
        )));
    }

    // Labels input
    lines.push(Line::raw(""));
    let labels_display = if app.issue_create_labels_input.is_empty() {
        "(comma-separated)".to_string()
    } else {
        app.issue_create_labels_input.clone()
    };
    let labels_cursor = if app.issue_create_focus == 2 { "█" } else { "" };
    lines.push(ansi_to_line(&format!(
        "  {}Labels:{} {}{}",
        ansi::BLD, ansi::RST,
        labels_display,
        labels_cursor
    )));
    if !app.issue_create_labels_input.is_empty() {
        let parsed: Vec<&str> = app.issue_create_labels_input.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if !parsed.is_empty() {
            lines.push(ansi_to_line(&format!(
                "  {}  → will apply: [{}]{}",
                ansi::DIM,
                parsed.join("]["),
                ansi::RST
            )));
        }
    }

    // Label autocomplete suggestions
    if !app.label_ac_list.is_empty() {
        lines.push(Line::raw(""));
        lines.push(ansi_to_line(&format!(
            "  {}{} label(s) found{}",
            ansi::DIM, app.label_ac_list.len(), ansi::RST
        )));
        for (idx, label) in app.label_ac_list.iter().enumerate() {
            let marker = if idx == app.label_ac_idx {
                format!("{}{}▸{} ", ansi::BLD, ansi::LGR, ansi::RST)
            } else {
                "  ".to_string()
            };
            lines.push(ansi_to_line(&format!(
                "  {}{}{}",
                marker, ansi::LBL, label,
            )));
        }
    }

    lines.push(Line::raw(""));
    lines.push(ansi_to_line(&format!(
        "  {}Tab{} = switch/cycle  {}{}Enter{} = submit  {}Esc{} = cancel{}",
        ansi::BLD, ansi::RST,
        ansi::BLD, ansi::LGR, ansi::RST,
        ansi::BLD, ansi::LRE, ansi::RST
    )));

    let vis = (area.height as usize).max(1);
    while lines.len() < vis {
        lines.push(Line::raw(""));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════
// Issue Edit View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_issue_edit_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Header
    lines.push(ansi_to_line(&format!(
        "  {}Edit issue #{}   {}(from {}){}",
        ansi::BLD, app.issue_edit_number, ansi::DIM,
        app.repo_name, ansi::RST
    )));
    lines.push(ansi_to_line(&format!(
        "  {}─────────────────────{}",
        ansi::DIM, ansi::RST
    )));
    lines.push(Line::raw(""));

    // Title input
    let title_cursor = if app.issue_edit_focus == 0 { "█" } else { " " };
    lines.push(ansi_to_line(&format!(
        "  {}Title:{} {} {}",
        ansi::BLD, ansi::RST,
        app.issue_edit_title, title_cursor,
    )));
    if app.issue_edit_focus == 0 {
        lines.push(ansi_to_line(&format!(
            "  {}  {}",
            ansi::DIM, ansi::RST
        )));
    }

    // Body input
    lines.push(Line::raw(""));
    if app.issue_edit_focus == 1 {
        lines.push(ansi_to_line(&format!(
            "  {}Body:{} {}",
            ansi::BLD, ansi::RST,
            app.issue_edit_body,
        )));
        if app.issue_edit_body.is_empty() {
            lines.push(ansi_to_line(&format!(
                "  {}  {}",
                ansi::DIM, ansi::RST
            )));
        } else {
            for body_line in app.issue_edit_body.lines().take(5) {
                lines.push(ansi_to_line(&format!("  {}", body_line)));
            }
        }
    } else {
        lines.push(ansi_to_line(&format!(
            "  {}Body:{} (type to update){}",
            ansi::BLD, ansi::DIM, ansi::RST
        )));
        if !app.issue_edit_body.is_empty() {
            for body_line in app.issue_edit_body.lines().take(3) {
                lines.push(ansi_to_line(&format!("  {}", body_line)));
            }
        }
    }

    // Labels input
    lines.push(Line::raw(""));
    if app.issue_edit_focus == 2 {
        let labels_display = if app.issue_edit_labels_input.is_empty() {
            "(comma-separated)".to_string()
        } else {
            app.issue_edit_labels_input.clone()
        };
        lines.push(ansi_to_line(&format!(
            "  {}Labels:{} {}█",
            ansi::BLD, ansi::RST,
            labels_display,
        )));
        if !app.issue_edit_labels_input.is_empty() {
            let parsed: Vec<&str> = app.issue_edit_labels_input.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if !parsed.is_empty() {
                lines.push(ansi_to_line(&format!(
                    "  {}  → will apply: [{}]{}",
                    ansi::DIM,
                    parsed.join("]["),
                    ansi::RST
                )));
            }
        }
    } else {
        let labels_display = if app.issue_edit_labels_input.is_empty() {
            "(unchanged)".to_string()
        } else {
            app.issue_edit_labels_input.clone()
        };
        lines.push(ansi_to_line(&format!(
            "  {}Labels:{} {}",
            ansi::BLD, ansi::RST,
            labels_display,
        )));
    }

    // Label autocomplete suggestions
    if !app.label_ac_list.is_empty() {
        lines.push(Line::raw(""));
        lines.push(ansi_to_line(&format!(
            "  {}{} label(s) found{}",
            ansi::DIM, app.label_ac_list.len(), ansi::RST
        )));
        for (idx, label) in app.label_ac_list.iter().enumerate() {
            let marker = if idx == app.label_ac_idx {
                format!("{}{}▸{} ", ansi::BLD, ansi::LGR, ansi::RST)
            } else {
                "  ".to_string()
            };
            lines.push(ansi_to_line(&format!(
                "  {}{}{}",
                marker, ansi::LBL, label,
            )));
        }
    }

    lines.push(Line::raw(""));
    lines.push(ansi_to_line(&format!(
        "  {}Tab{} = switch/cycle  {}{}Enter{} = submit  {}Esc{} = cancel{}",
        ansi::BLD, ansi::RST,
        ansi::BLD, ansi::LGR, ansi::RST,
        ansi::BLD, ansi::LRE, ansi::RST
    )));

    let vis = (area.height as usize).max(1);
    while lines.len() < vis {
        lines.push(Line::raw(""));
    }

    let paragraph = Paragraph::new(Text::from(lines)).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════
// Issue Detail View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_issue_detail_view(frame: &mut Frame, area: Rect, app: &AppState) {
    let vis = (area.height as usize).max(1);
    let total = app.issue_detail_lines.len();
    let max_s = total.saturating_sub(vis);

    // Auto-scroll to keep cursor visible
    let mut scroll = app.issue_detail_scroll;
    if app.issue_detail_cursor < scroll {
        scroll = app.issue_detail_cursor;
    }
    if app.issue_detail_cursor >= scroll + vis {
        scroll = app.issue_detail_cursor.saturating_sub(vis - 1);
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
        let mut line = ansi_to_line(&app.issue_detail_lines[idx]);
        if idx == app.issue_detail_cursor {
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

