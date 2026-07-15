use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

use crate::ansi;
use crate::app::AppState;
use crate::ui::ansi_lines_to_text;

// ═══════════════════════════════════════════════════════════════════════════
// Help View
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn render_help_view(frame: &mut Frame, area: Rect, _app: &AppState) {
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

