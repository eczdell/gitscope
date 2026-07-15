use std::time::Duration;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

use crate::ansi;
use crate::app::{AppMode, AppState};

mod diff;
mod files;
mod gists;
mod help;
mod issues;
mod repos;
mod report;
mod tree;

pub(crate) use diff::render_diff_view;
pub(crate) use files::render_files_view;
pub(crate) use gists::{render_gist_content_view, render_gists_view};
pub(crate) use help::render_help_view;
pub(crate) use issues::{
    render_issue_create_view, render_issue_detail_view, render_issue_edit_view,
    render_issues_view,
};
pub(crate) use repos::{render_repos_add_view, render_repos_view};
pub(crate) use report::render_report_view;
pub(crate) use tree::render_tree_view;

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
pub(crate) fn ansi_to_line(s: &str) -> Line<'static> {
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
pub(crate) fn ansi_lines_to_text(lines: &[String]) -> Text<'static> {
    let ratatui_lines: Vec<Line<'static>> = lines.iter().map(|s| ansi_to_line(s)).collect();
    Text::from(ratatui_lines)
}

// ═══════════════════════════════════════════════════════════════════════════
// Status Bar
// ═══════════════════════════════════════════════════════════════════════════

fn build_status_bar_styled(app: &AppState) -> Line<'static> {
    let mode_label = match app.mode {
        AppMode::Diff => " diff ",
        AppMode::Files => " files ",
        AppMode::Report | AppMode::ReportFilter => " report ",
        AppMode::Help => " help ",
        AppMode::TreeFilter => " filter ",
        AppMode::SidebarFocus => " sidebar ",
        AppMode::Issues => " issues ",
        AppMode::IssueCreate => " issue create ",
        AppMode::IssueDetail => " issue detail ",
        AppMode::IssueEdit => " issue edit ",
        AppMode::Gists | AppMode::GistsFilter => " gists ",
        AppMode::GistContent => " gist content ",
        AppMode::Repos | AppMode::ReposAdd => " repos ",
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
            format!(" 📅{}→{} ", app.date_from, app.date_to),
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
                " ▼zoom:{} ",
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
        AppMode::SidebarFocus => format!(
            "  {}j/k{} ↕ select  {}Enter{} confirm  {}0{} reset  {}Esc/b{} cancel{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::RST
        ),
        AppMode::IssueDetail => format!(
            "  {}j/k{} ↕ scroll  {}J/K{} page  {}o{} open image  {}Esc/q{} back{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::RST
        ),
        AppMode::Issues => format!(
            "  {}j/k{} ↕  {}Enter{} open  {}x{} close  {}e{} edit  {}c{} create  {}s{} state  {}q{} quit{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST,
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::RST
        ),
        AppMode::Repos => format!(
            "  {}j/k{} ↕  {}Enter{} switch  {}a{} add  {}s{} scan  {}x{} remove  {}q{} quit{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST,
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST,
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::RST
        ),
        _ => format!(
            "  {}j/k{} ↕  {}d{} diff  {}f{} files  {}y{} copy  {}D{} date  {}R{} report  {}/{} filter  {}?{} help  {}P{} repos  {}\\\\'{} gists  {}q{} quit{}",
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST,
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST,
            ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::DIM, ansi::RST, ansi::RST
        ),
    };
    let line = ansi_to_line(&text);
    Paragraph::new(Text::from(vec![line])).style(Style::default().bg(Color::Black))
}

// ═══════════════════════════════════════════════════════════════════════════
// Overlay Renders
// ═══════════════════════════════════════════════════════════════════════════

fn render_filter_input(app: &AppState) -> Option<Line<'static>> {
    if app.mode == AppMode::TreeFilter {
        Some(ansi_to_line(&format!(
            "  {}{}{}{}{}█ Esc to cancel{}",
            ansi::BLD, ansi::LGR, app.filter_input, ansi::RST, ansi::DIM, ansi::RST
        )))
    } else if app.mode == AppMode::IssuesFilter {
        Some(ansi_to_line(&format!(
            "  {}{}{}{}{}█ Esc to cancel  Enter to apply{}",
            ansi::BLD, ansi::LGR, app.issues_filter_input, ansi::RST, ansi::DIM, ansi::RST
        )))
    } else if app.mode == AppMode::IssuesLabelFilter {
        Some(ansi_to_line(&format!(
            "  {}{}{}{}{}█ Esc to cancel  Enter to apply  (label filter){}",
            ansi::BLD, ansi::LGR, app.issues_label_filter_input, ansi::RST, ansi::DIM, ansi::RST
        )))
    } else {
        None
    }
}

fn render_report_filter(app: &AppState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if app.mode != AppMode::ReportFilter {
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
// Main UI Entry Point
// ═══════════════════════════════════════════════════════════════════════════

pub fn ui(frame: &mut Frame, app: &AppState) {
    // Get terminal area
    let area = frame.area();

    // Calculate vertical layout: status bar, filter area (optional), content, message (optional), command bar
    let mut constraints = vec![Constraint::Length(1)]; // status bar

    // Filter input line (TreeFilter or IssuesFilter or IssuesLabelFilter mode)
    if app.mode == AppMode::TreeFilter
        || app.mode == AppMode::IssuesFilter
        || app.mode == AppMode::IssuesLabelFilter
    {
        constraints.push(Constraint::Length(1));
    }

    // Report filter lines
    if app.mode == AppMode::ReportFilter {
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
    if app.mode == AppMode::TreeFilter
        || app.mode == AppMode::IssuesFilter
        || app.mode == AppMode::IssuesLabelFilter
    {
        if let Some(line) = render_filter_input(app) {
            let paragraph = Paragraph::new(Text::from(vec![line]))
                .style(Style::default().bg(Color::Black));
            frame.render_widget(paragraph, chunks[chunk_idx]);
        }
        chunk_idx += 1;
    }

    // Report filter
    if app.mode == AppMode::ReportFilter {
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
        AppMode::Tree
        | AppMode::TreeFilter
        | AppMode::SidebarFocus
        | AppMode::Help => {
            if app.mode == AppMode::Help {
                render_help_view(frame, chunks[chunk_idx], app);
            } else {
                render_tree_view(frame, chunks[chunk_idx], app);
            }
        }
        AppMode::Diff => render_diff_view(frame, chunks[chunk_idx], app),
        AppMode::Files => render_files_view(frame, chunks[chunk_idx], app),
        AppMode::Issues | AppMode::IssuesFilter | AppMode::IssuesLabelFilter => {
            render_issues_view(frame, chunks[chunk_idx], app)
        }
        AppMode::IssueCreate => render_issue_create_view(frame, chunks[chunk_idx], app),
        AppMode::IssueEdit => render_issue_edit_view(frame, chunks[chunk_idx], app),
        AppMode::IssueDetail => {
            render_issue_detail_view(frame, chunks[chunk_idx], app)
        }
        AppMode::Report | AppMode::ReportFilter => {
            render_report_view(frame, chunks[chunk_idx], app)
        }
        AppMode::Repos => render_repos_view(frame, chunks[chunk_idx], app),
        AppMode::ReposAdd => render_repos_add_view(frame, chunks[chunk_idx], app),
        AppMode::Gists | AppMode::GistsFilter => {
            render_gists_view(frame, chunks[chunk_idx], app)
        }
        AppMode::GistContent => {
            render_gist_content_view(frame, chunks[chunk_idx], app)
        }
    }
    chunk_idx += 1;

    // Command bar
    frame.render_widget(render_command_bar(app), chunks[chunk_idx]);
}

