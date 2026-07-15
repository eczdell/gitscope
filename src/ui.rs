use std::time::Duration;

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

// ═══════════════════════════════════════════════════════════════════════════
// ANSI <-> Ratatui Conversion
// ═══════════════════════════════════════════════════════════════════════════

fn ansi_num_to_color(n: u8) -> Color {
    match n {
        31 => Color::Red,
        32 => Color::Green,
        33 => Color::Yellow,
        34 => Color::Blue,
        35 => Color::Magenta,
        36 => Color::Cyan,
        37 => Color::White,
        90 => Color::DarkGray,
        91 => Color::LightRed,
        92 => Color::LightGreen,
        93 => Color::LightYellow,
        94 => Color::LightBlue,
        95 => Color::LightMagenta,
        96 => Color::LightCyan,
        _ => Color::White,
    }
}

/// Convert an ANSI-escape-code string to a styled Ratatui Line by parsing
/// escape sequences into styled Spans.
fn ansi_to_line(s: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_style = Style::default();
    let mut buf = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                // Flush current buffer
                if !buf.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut buf), current_style));
                }
                chars.next(); // consume '['

                // Parse the ANSI code
                let mut code_str = String::new();
                for ec in &mut chars {
                    if ec.is_ascii_alphabetic() {
                        break;
                    }
                    code_str.push(ec);
                }

                // Apply the ANSI code
                if code_str == "0" {
                    current_style = Style::default();
                } else if code_str == "1" {
                    current_style = current_style.add_modifier(Modifier::BOLD);
                } else if code_str == "2" {
                    current_style = current_style.add_modifier(Modifier::DIM);
                } else if let Ok(n) = code_str.parse::<u8>() {
                    current_style = current_style.fg(ansi_num_to_color(n));
                }
            }
        } else {
            buf.push(c);
        }
    }

    if !buf.is_empty() {
        spans.push(Span::styled(buf, current_style));
    }

    Line::from(spans)
}

/// Convert a vector of ANSI-escape-code strings to styled Ratatui Text
fn ansi_lines_to_text(lines: &[String]) -> Text<'static> {
    let ratatui_lines: Vec<Line<'static>> = lines.iter().map(|s| ansi_to_line(s)).collect();
    Text::from(ratatui_lines)
}

// ═══════════════════════════════════════════════════════════════════════════
// Status Bar
// ═══════════════════════════════════════════════════════════════════════════

fn build_status_bar_styled(app: &AppState) -> Line<'static> {
    let mode_label = match app.mode {
        crate::app::AppMode::Diff => " diff ",
        crate::app::AppMode::Files => " files ",
        crate::app::AppMode::Report | crate::app::AppMode::ReportFilter => " report ",
        crate::app::AppMode::Help => " help ",
        crate::app::AppMode::TreeFilter => " filter ",
        crate::app::AppMode::SidebarFocus => " sidebar ",
        crate::app::AppMode::Issues => " issues ",
        crate::app::AppMode::IssueCreate => " issue create ",
        crate::app::AppMode::IssueDetail => " issue detail ",
        crate::app::AppMode::IssueEdit => " issue edit ",
        _ => " tree ",
    };

    let mut spans: Vec<Span<'static>> = Vec::new();

    // Spacer
    spans.push(Span::raw(" "));

    // Repo name pill
    spans.push(Span::styled(
        format!(" {} ", app.repo_name),
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));

    // Separator
    spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));

    // Branch pill
    if app.current_branch == "detached" {
        spans.push(Span::styled(
            format!(" {} ", app.current_branch),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::styled(
            format!(" {} ", app.current_branch),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Separator
    spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));

    // Commit count
    spans.push(Span::styled(
        format!(" {} commits ", app.total),
        Style::default().fg(Color::White),
    ));

    // Separator
    spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));

    // Mode pill
    spans.push(Span::styled(
        mode_label.to_string(),
        Style::default()
            .fg(Color::Black)
            .bg(Color::LightCyan)
            .add_modifier(Modifier::BOLD),
    ));

    // Filter indicators
    if !app.filter_text.is_empty() {
        spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));
        spans.push(Span::styled(
            format!(" /{} ", app.filter_text),
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if !app.date_from.is_empty() || !app.date_to.is_empty() {
        spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));
        spans.push(Span::styled(
            format!(" \u{1F4C5}{}→{} ", app.date_from, app.date_to),
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if !app.descendant_filter.is_empty() {
        spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));
        spans.push(Span::styled(
            format!(
                " \u{25BC}zoom:{} ",
                &app.descendant_filter[..7.min(app.descendant_filter.len())]
            ),
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Toggle flags as small pills
    if app.show_all {
        spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));
        spans.push(Span::styled(
            " ALL ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if app.show_meta {
        spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));
        spans.push(Span::styled(
            " META ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if app.compact {
        spans.push(Span::styled(" │ ", Style::default().add_modifier(Modifier::DIM)));
        spans.push(Span::styled(
            " CMP ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        ));
    }

    Line::from(spans)
}

fn render_status_bar(app: &AppState) -> Paragraph<'static> {
    let line = build_status_bar_styled(app);
    Paragraph::new(Text::from(vec![line])).style(Style::default().bg(Color::Black))
}

fn render_command_bar(app: &AppState) -> Paragraph<'static> {
    let text = match app.mode {
        crate::app::AppMode::SidebarFocus => format!(
            "  {}j/k{} ↕ select  {}Enter{} confirm  {}0{} reset  {}Esc/b{} cancel{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::RST
        ),
        crate::app::AppMode::IssueDetail => format!(
            "  {}j/k{} ↕ scroll  {}J/K{} page  {}Esc/q{} back{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::RST
        ),
        crate::app::AppMode::Issues => format!(
            "  {}j/k{} ↕  {}Enter{} open  {}x{} close  {}e{} edit  {}c{} create  {}s{} state  {}q{} quit{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST,
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::RST
        ),
        _ => format!(
            "  {}j/k{} ↕  {}d{} diff  {}f{} files  {}y{} copy  {}D{} date  {}R{} report  {}/{} filter  {}?{} help  {}q{} quit{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST,
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST,
            ansi::DIM, ansi::RST, ansi::RST
        ),
    };
    let line = ansi_to_line(&text);
    Paragraph::new(Text::from(vec![line])).style(Style::default().bg(Color::Black))
}

// ═══════════════════════════════════════════════════════════════════════════
// Tree View
// ═══════════════════════════════════════════════════════════════════════════

fn render_tree_view(frame: &mut Frame, area: Rect, app: &AppState) {
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

fn render_tree_content(frame: &mut Frame, area: Rect, app: &AppState) {
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

// ═══════════════════════════════════════════════════════════════════════════
// Diff View
// ═══════════════════════════════════════════════════════════════════════════

fn render_diff_view(frame: &mut Frame, area: Rect, app: &AppState) {
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

// ═══════════════════════════════════════════════════════════════════════════
// Files View
// ═══════════════════════════════════════════════════════════════════════════

fn render_files_view(frame: &mut Frame, area: Rect, app: &AppState) {
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

// ═══════════════════════════════════════════════════════════════════════════
// Report View
// ═══════════════════════════════════════════════════════════════════════════

fn render_report_view(frame: &mut Frame, area: Rect, app: &AppState) {
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

// ═══════════════════════════════════════════════════════════════════════════
// Issues View
// ═══════════════════════════════════════════════════════════════════════════

fn render_issues_view(frame: &mut Frame, area: Rect, app: &AppState) {
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

fn render_issue_create_view(frame: &mut Frame, area: Rect, app: &AppState) {
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
    lines.push(ansi_to_line(&format!(
        "  {}Title:{} {}{}",
        ansi::BLD, ansi::RST,
        app.issue_create_title,
        if app.issue_create_focus_title { "█" } else { "" }
    )));

    // Body input
    lines.push(Line::raw(""));
    let body_label = if app.issue_create_focus_title {
        format!("  {}Body:{} (optional){}", ansi::BLD, ansi::DIM, ansi::RST)
    } else {
        format!("  {}Body:{} {}", ansi::BLD, ansi::RST, "█")
    };
    lines.push(ansi_to_line(&body_label));
    if !app.issue_create_body.is_empty() {
        for body_line in app.issue_create_body.lines().take(5) {
            lines.push(ansi_to_line(&format!("  {}", body_line)));
        }
    } else if !app.issue_create_focus_title {
        lines.push(ansi_to_line(&format!(
            "  {}  {}",
            ansi::DIM, ansi::RST
        )));
    }

    lines.push(Line::raw(""));
    lines.push(ansi_to_line(&format!(
        "  {}Tab{} = switch field  {}{}Enter{} = submit  {}Esc{} = cancel{}",
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

fn render_issue_edit_view(frame: &mut Frame, area: Rect, app: &AppState) {
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
    let title_cursor = if app.issue_edit_focus_title { "█" } else { " " };
    lines.push(ansi_to_line(&format!(
        "  {}Title:{} {}{}{}",
        ansi::BLD, ansi::RST,
        app.issue_edit_title,
        if app.issue_edit_focus_title { "█" } else { "" },
        if app.issue_edit_focus_title { "" } else { title_cursor }
    )));
    if !app.issue_edit_focus_title {
        // Show first few lines of body when body is focused
        lines.push(Line::raw(""));
        lines.push(ansi_to_line(&format!(
            "  {}Body:{} {}{}",
            ansi::BLD, ansi::RST,
            app.issue_edit_body,
            if !app.issue_edit_focus_title { "█" } else { "" }
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
        lines.push(Line::raw(""));
        lines.push(ansi_to_line(&format!(
            "  {}Body:{} (unchanged){}{}",
            ansi::BLD, ansi::DIM, ansi::RST,
            if !app.issue_edit_focus_title { " █" } else { "" }
        )));
        if !app.issue_edit_body.is_empty() {
            for body_line in app.issue_edit_body.lines().take(3) {
                lines.push(ansi_to_line(&format!("  {}", body_line)));
            }
        }
    }

    lines.push(Line::raw(""));
    lines.push(ansi_to_line(&format!(
        "  {}Tab{} = switch field  {}{}Enter{} = submit  {}Esc{} = cancel{}",
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

fn render_help_view(frame: &mut Frame, area: Rect, _app: &AppState) {
    let hx = ((area.width as usize).saturating_sub(54)) / 2;
    let hx = hx.max(2);
    let pfx = " ".repeat(hx);

    // Build help text with ANSI codes
    let mut help_lines: Vec<String> = Vec::new();

    help_lines.push(String::new());
    help_lines.push(format!(
        "{}{}╔════════════════════════════════════════════════════╗{}",
        pfx, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}  {}gitscope - Interactive Git Tree Viewer{}          {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::BLD, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}╠════════════════════════════════════════════════════╣{}",
        pfx, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}  {}Navigation{}                                      {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::BLD, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}j{} / {}↓{}       Move cursor down                  {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LGR, ansi::RST, ansi::LGR, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}k{} / {}↑{}       Move cursor up                    {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LGR, ansi::RST, ansi::LGR, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}J{} / {}K{}       Move 5 lines                     {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LGR, ansi::RST, ansi::LGR, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}g{} / {}G{}       Top / Bottom                      {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LGR, ansi::RST, ansi::LGR, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}PgUp/PgDn{}   Page up/down                      {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LGR, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}╠════════════════════════════════════════════════════╣{}",
        pfx, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}  {}Actions{}                                          {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::BLD, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}Enter/o{}     Zoom into commit descendants      {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}d{}           View commit diff                  {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}f{}           View changed files                 {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}y{}           Copy commit hash                   {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}/{}           Filter commits (regex)            {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}D{}           Toggle last-7-day date filter      {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}R{}           Report: files by author            {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}r{}           Refresh data                      {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}╠════════════════════════════════════════════════════╣{}",
        pfx, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}  {}Display{}                                          {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::BLD, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}a{}           Toggle all branches                {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}m{}           Toggle commit metadata             {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}c{}           Toggle compact mode                {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}+{} / {}-{}         ±10 commits                       {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LYL, ansi::RST, ansi::LYL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}  {}Sidebar{}                                           {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::BLD, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}b{}           Focus branch picker (sidebar)     {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LBL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}0{}           Reset all filters                 {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LBL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}ESC{}          Clear zoom filter                  {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LBL, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}╠════════════════════════════════════════════════════╣{}",
        pfx, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}?{}           Toggle this help                  {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LRE, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}║{}    {}q{} / {}Ctrl-C{}    Quit                              {}{}║{}",
        pfx, ansi::CYN, ansi::RST, ansi::LRE, ansi::RST, ansi::LRE, ansi::RST, ansi::BLD, ansi::CYN, ansi::RST
    ));
    help_lines.push(format!(
        "{}{}╚════════════════════════════════════════════════════╝{}",
        pfx, ansi::CYN, ansi::RST
    ));

    let vis = (area.height as usize).max(1);
    let remain = vis.saturating_sub(help_lines.len());
    for _ in 0..remain {
        help_lines.push(String::new());
    }

    let text = ansi_lines_to_text(&help_lines);
    let paragraph = Paragraph::new(text).style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

// ═══════════════════════════════════════════════════════════════════════════
// Overlay Renders
// ═══════════════════════════════════════════════════════════════════════════

fn render_filter_input(app: &AppState) -> Option<Line<'static>> {
    if app.mode == crate::app::AppMode::TreeFilter {
        Some(ansi_to_line(&format!(
            "  {}{}{}{}{}█ Esc to cancel{}",
            ansi::BLD, ansi::LGR, app.filter_input, ansi::RST, ansi::DIM, ansi::RST
        )))
    } else if app.mode == crate::app::AppMode::IssuesFilter {
        Some(ansi_to_line(&format!(
            "  {}{}{}{}{}█ Esc to cancel  Enter to apply{}",
            ansi::BLD, ansi::LGR, app.issues_filter_input, ansi::RST, ansi::DIM, ansi::RST
        )))
    } else {
        None
    }
}

fn render_report_filter(app: &AppState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if app.mode != crate::app::AppMode::ReportFilter {
        return lines;
    }

    lines.push(ansi_to_line(&format!(
        "  {}{}{}{}{}█ Esc to cancel  Tab to select{}",
        ansi::BLD, ansi::LGR, app.report_email_input, ansi::RST, ansi::DIM, ansi::RST
    )));

    let ac_total = app.report_ac_list.len();
    if ac_total > 0 {
        lines.push(ansi_to_line(&format!(
            "  {}{} match(es){}",
            ansi::DIM, ac_total, ansi::RST
        )));
        for (idx, sug) in app.report_ac_list.iter().enumerate() {
            let prefix_len = sug
                .to_lowercase()
                .find(&app.report_email_input.to_lowercase())
                .unwrap_or(0);
            let prefix = &sug[..prefix_len];
            let match_str = &sug[prefix_len..prefix_len + app.report_email_input.len()];
            let rest = &sug[prefix_len + app.report_email_input.len()..];
            let marker = if idx == app.report_ac_idx {
                format!("{}{}▸{} ", ansi::BLD, ansi::LGR, ansi::RST)
            } else {
                "  ".to_string()
            };
            lines.push(ansi_to_line(&format!(
                "  {}{}{}{}{}{}{}",
                marker, ansi::DIM, prefix, ansi::RST, ansi::BLD, ansi::LGR, match_str,
            )));
            if !rest.is_empty() {
                lines.push(ansi_to_line(&format!("{}{}{}", ansi::DIM, rest, ansi::RST)));
            }
        }
    } else if !app.report_email_input.is_empty() {
        lines.push(ansi_to_line(&format!(
            "  {}  no matches{}",
            ansi::DIM, ansi::RST
        )));
    }

    lines
}

fn render_message(app: &AppState) -> Option<Line<'static>> {
    if let Some(ref msg_time) = app.msg_time {
        if !app.msg.is_empty() && msg_time.elapsed() < Duration::from_secs(2) {
            return Some(ansi_to_line(&format!(
                "  {}{}{}{}",
                ansi::BLD, ansi::LGR, app.msg, ansi::RST
            )));
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════════
// Issue Detail View
// ═══════════════════════════════════════════════════════════════════════════

fn render_issue_detail_view(frame: &mut Frame, area: Rect, app: &AppState) {
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

// ═══════════════════════════════════════════════════════════════════════════
// Main UI Entry Point
// ═══════════════════════════════════════════════════════════════════════════

pub fn ui(frame: &mut Frame, app: &AppState) {
    // Get terminal area
    let area = frame.area();

    // Calculate vertical layout: status bar, filter area (optional), content, message (optional), command bar
    let mut constraints = vec![Constraint::Length(1)]; // status bar

    // Filter input line (TreeFilter or IssuesFilter mode)
    if app.mode == crate::app::AppMode::TreeFilter || app.mode == crate::app::AppMode::IssuesFilter {
        constraints.push(Constraint::Length(1));
    }

    // Report filter lines
    if app.mode == crate::app::AppMode::ReportFilter {
        let extra = if app.report_ac_list.is_empty() && app.report_email_input.is_empty() {
            1
        } else {
            2 + app.report_ac_list.len().min(5)
        };
        constraints.push(Constraint::Length(extra as u16));
    }

    // Message line
    let has_msg = app.msg_time.as_ref().map_or(false, |t| {
        !app.msg.is_empty() && t.elapsed() < Duration::from_secs(2)
    });
    if has_msg {
        constraints.push(Constraint::Length(1));
    }

    // Content area (fills remaining space)
    constraints.push(Constraint::Min(0));

    // Command bar
    constraints.push(Constraint::Length(1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut chunk_idx = 0;

    // Status bar
    frame.render_widget(render_status_bar(app), chunks[chunk_idx]);
    chunk_idx += 1;

    // Filter input
    if app.mode == crate::app::AppMode::TreeFilter || app.mode == crate::app::AppMode::IssuesFilter {
        if let Some(line) = render_filter_input(app) {
            let paragraph = Paragraph::new(Text::from(vec![line]))
                .style(Style::default().bg(Color::Black));
            frame.render_widget(paragraph, chunks[chunk_idx]);
        }
        chunk_idx += 1;
    }

    // Report filter
    if app.mode == crate::app::AppMode::ReportFilter {
        let filter_lines = render_report_filter(app);
        let paragraph = Paragraph::new(filter_lines).style(Style::default().bg(Color::Black));
        frame.render_widget(paragraph, chunks[chunk_idx]);
        chunk_idx += 1;
    }

    // Message
    if has_msg {
        if let Some(line) = render_message(app) {
            let paragraph = Paragraph::new(Text::from(vec![line]))
                .style(Style::default().bg(Color::Black));
            frame.render_widget(paragraph, chunks[chunk_idx]);
        }
        chunk_idx += 1;
    }

    // Content area
    match app.mode {
        crate::app::AppMode::Tree
        | crate::app::AppMode::TreeFilter
        | crate::app::AppMode::SidebarFocus
        | crate::app::AppMode::Help => {
            if app.mode == crate::app::AppMode::Help {
                render_help_view(frame, chunks[chunk_idx], app);
            } else {
                render_tree_view(frame, chunks[chunk_idx], app);
            }
        }
        crate::app::AppMode::Diff => render_diff_view(frame, chunks[chunk_idx], app),
        crate::app::AppMode::Files => render_files_view(frame, chunks[chunk_idx], app),
        crate::app::AppMode::Issues | crate::app::AppMode::IssuesFilter => {
            render_issues_view(frame, chunks[chunk_idx], app)
        }
        crate::app::AppMode::IssueCreate => render_issue_create_view(frame, chunks[chunk_idx], app),
        crate::app::AppMode::IssueEdit => render_issue_edit_view(frame, chunks[chunk_idx], app),
        crate::app::AppMode::IssueDetail => {
            render_issue_detail_view(frame, chunks[chunk_idx], app)
        }
        crate::app::AppMode::Report | crate::app::AppMode::ReportFilter => {
            render_report_view(frame, chunks[chunk_idx], app)
        }
    }
    chunk_idx += 1;

    // Command bar
    frame.render_widget(render_command_bar(app), chunks[chunk_idx]);
}

