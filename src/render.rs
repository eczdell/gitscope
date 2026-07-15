use crate::ansi::{self, vis_len};
use crate::app::AppState;

// ═══════════════════════════════════════════════════════════════════════════
// Render Buffer (produces ANSI-styled strings)
// ═══════════════════════════════════════════════════════════════════════════

pub fn build_render_buffer(app: &mut AppState) {
    app.render_lines.clear();

    if app.commits.is_empty() {
        app.render_lines.push(crate::app::RenderLine {
            content: format!("  {}No commits to display{}", ansi::DIM, ansi::RST),
            commit_idx: None,
        });
        return;
    }

    let tree_w = compute_tree_w(app);
    let ga_w = app.nlanes * 2 + 4;
    let prefix_w = 4 + ga_w + 1;
    let hash_w = 8;
    let auth_w = 14;
    let date_w = 12;
    let subj_w = tree_w
        .saturating_sub(prefix_w + hash_w + auth_w + date_w + 6)
        .max(10);

    let fl = app.filter_text.clone();
    let has_desc = !app.descendant_set.is_empty();

    for i in 0..app.commits.len() {
        // Text filter
        if !fl.is_empty() {
            let fl_lower = fl.to_lowercase();
            let h_lower = app.commits[i].hash.to_lowercase();
            let s_lower = app.commits[i].subject.to_lowercase();
            let a_lower = app.commits[i].author.to_lowercase();
            if !h_lower.contains(&fl_lower)
                && !s_lower.contains(&fl_lower)
                && !a_lower.contains(&fl_lower)
            {
                continue;
            }
        }

        // Date filter
        if !app.date_from.is_empty() {
            let c_date = &app.commits[i].date[..10.min(app.commits[i].date.len())];
            if c_date < app.date_from.as_str() {
                continue;
            }
        }
        if !app.date_to.is_empty() {
            let c_date = &app.commits[i].date[..10.min(app.commits[i].date.len())];
            if c_date > app.date_to.as_str() {
                continue;
            }
        }

        // Descendant filter
        if has_desc && !app.descendant_set.contains(&app.commits[i].hash) {
            continue;
        }

        let hash = &app.commits[i].hash;
        let lane = app.lanes[i];
        let parents = &app.commits[i].parents;
        let np = parents.len();

        let short = &hash[..7.min(hash.len())];
        let subj: String = app.commits[i]
            .subject
            .chars()
            .take(subj_w)
            .collect();
        let subj_truncated = if app.commits[i].subject.len() > subj_w {
            format!("{}…", subj)
        } else {
            subj
        };
        let auth: String = app.commits[i]
            .author
            .chars()
            .take(12)
            .collect();
        let dt: String =
            app.commits[i].date[..10.min(app.commits[i].date.len())].to_string();

        // Node character and color
        let (ncol, nchar) = if hash == &app.head_hash {
            (ansi::LRE.to_string(), "▶")
        } else if np > 1 {
            (ansi::YEL.to_string(), "◆")
        } else {
            let mut tip_col = ansi::GRN.to_string();
            let mut tip_char = "●";
            for br in &app.branches {
                if br.head_oid == *hash {
                    tip_col = format!(
                        "\x1b[{}m",
                        color_to_ansi_num(br.color)
                    );
                    tip_char = "◉";
                    break;
                }
            }
            (tip_col, tip_char)
        };

        // Graph - 2-char-per-lane with color-coded branches and merge connectors
        let mut graph = String::new();
        let parent_lanes: Vec<usize> = if np > 1 {
            parents
                .iter()
                .filter_map(|p| app.index.get(p))
                .map(|&pi| app.lanes[pi])
                .collect()
        } else {
            Vec::new()
        };
        let min_ml = parent_lanes.iter().min().copied().unwrap_or(lane);
        let max_ml = parent_lanes.iter().max().copied().unwrap_or(lane);
        let is_merge = np > 1 && !parent_lanes.is_empty();

        for l in 0..app.nlanes {
            let oh = app.occupied.get(l).and_then(|o| o.as_ref());
            let oh_color = oh.map_or_else(|| ansi::DIM.to_string(), |h| hash_to_branch_color(app, h));

            // Check if this lane has a vertical line running through it
            let mut active = false;
            if let Some(oh_hash) = oh {
                for j in (i + 1)..app.commits.len() {
                    for jp in &app.commits[j].parents {
                        if *jp == *oh_hash {
                            active = true;
                            break;
                        }
                    }
                    if active {
                        break;
                    }
                }
            }
            if !active {
                for pp in parents {
                    if let Some(&pi) = app.index.get(pp) {
                        if app.lanes[pi] == l {
                            active = true;
                            break;
                        }
                    }
                }
            }

            let is_parent_lane = is_merge && parent_lanes.contains(&l) && l != lane;
            let is_between = is_merge && l > min_ml && l < max_ml;

            if l == lane {
                // Current lane: space + colored bold node
                graph.push_str(&format!(
                    " {}{}{}{}",
                    ncol, ansi::BLD, nchar, ansi::RST
                ));
            } else if is_parent_lane && l < lane {
                // Parent lane to the left: ╰─ in yellow
                graph.push_str(&format!("{}╰─{}", ansi::YEL, ansi::RST));
            } else if is_parent_lane && l > lane {
                // Parent lane to the right: ─╭ in yellow
                graph.push_str(&format!("{}─╭{}", ansi::YEL, ansi::RST));
            } else if is_between {
                // Between parent and current lane: ── in yellow
                graph.push_str(&format!("{}──{}", ansi::YEL, ansi::RST));
            } else if active {
                // Active lane: space + colored vertical line
                graph.push_str(&format!(" {}│{}", oh_color, ansi::RST));
            } else {
                // Inactive: two spaces
                graph.push_str("  ");
            }
        }

        // Pad graph
        let gv = vis_len(&graph);
        if gv < ga_w {
            graph.push_str(&" ".repeat(ga_w - gv));
        }

        // Build box
        let mut box_str = format!(
            "{}╰─{} {}{}{}{}",
            ncol, ansi::RST, ansi::BLD, ansi::CYN, short, ansi::RST
        );

        // Branch refs
        for br in &app.branches {
            if br.head_oid == *hash {
                let cu = if br.name == app.current_branch {
                    ansi::BLD
                } else {
                    ""
                };
                let br_ansi = format!("\x1b[{}m", color_to_ansi_num(br.color));
                box_str.push_str(&format!(
                    " {}{}{}{}",
                    br_ansi, cu, br.name, ansi::RST,
                ));
            }
        }

        box_str.push_str(&format!(
            "  {}{}{}  {}{}{} {}· {}{}",
            ansi::WHT,
            subj_truncated,
            ansi::RST,
            ansi::DIM,
            auth,
            ansi::RST,
            ansi::GRY,
            dt,
            ansi::RST
        ));

        let prefix = format!("  {:3} ", i);
        app.render_lines.push(crate::app::RenderLine {
            content: format!("{}{} {}", prefix, graph, box_str),
            commit_idx: Some(i),
        });

        // Meta line
        if app.show_meta {
            let mindent = " ".repeat(4 + ga_w + 2);
            let mut mline = format!(
                "{}{}{}  {}{}{}",
                mindent,
                ansi::GRY,
                hash,
                ansi::GRY,
                &app.commits[i].date[11..19.min(app.commits[i].date.len())],
                ansi::RST
            );
            if np > 1 {
                mline.push_str(&format!("  {}⟶ merge{}", ansi::YEL, ansi::RST));
            } else if np == 0 {
                mline.push_str(&format!("  {}⟶ root{}", ansi::GRN, ansi::RST));
            }
            app.render_lines.push(crate::app::RenderLine {
                content: mline,
                commit_idx: Some(i),
            });
        }

        // Spacer line (blank, no vertical connectors)
        // (removed: connector lines between commits)
    }
}

fn compute_tree_w(app: &AppState) -> usize {
    let sidebar_w = if app.term_w >= 80 { 28 } else { 0 };
    let tw = (app.term_w as usize).saturating_sub(sidebar_w + 1);
    tw.max(20)
}

/// Returns the ANSI color code for the branch that has the given commit hash as its head.
/// Falls back to DIM if no branch matches.
fn hash_to_branch_color(app: &AppState, hash: &str) -> String {
    for br in &app.branches {
        if br.head_oid == *hash {
            return format!("\x1b[{}m", color_to_ansi_num(br.color));
        }
    }
    ansi::DIM.to_string()
}

fn color_to_ansi_num(color: ratatui::style::Color) -> u8 {
    use ratatui::style::Color;
    match color {
        Color::Red => 31,
        Color::Green => 32,
        Color::Yellow => 33,
        Color::Blue => 34,
        Color::Magenta => 35,
        Color::Cyan => 36,
        Color::White => 37,
        Color::DarkGray => 90,
        Color::LightRed => 91,
        Color::LightGreen => 92,
        Color::LightYellow => 93,
        Color::LightBlue => 94,
        Color::LightMagenta => 95,
        Color::LightCyan => 96,
        _ => 37,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Sidebar (produces ANSI-styled strings)
// ═══════════════════════════════════════════════════════════════════════════

pub fn build_sidebar(app: &AppState, vis: usize) -> Vec<String> {
    // Issues mode sidebar
    if app.mode == crate::app::AppMode::Issues || app.mode == crate::app::AppMode::IssuesFilter {
        return build_issues_sidebar(app, vis);
    }

    let mut lines: Vec<String> = Vec::new();
    let nb = app.branches.len();
    let na = app.authors.len();
    let half = vis / 2;
    let br_vis = half.max(3);
    let au_vis = vis.saturating_sub(br_vis);
    let inner: usize = 26; // sidebar_w - 2

    // Sidebar header
    lines.push(format!(
        " {}{}BRANCHES{} {}({}){}",
        ansi::BLD, ansi::LBL, ansi::RST, ansi::DIM, nb, ansi::RST
    ));

    let br_end = br_vis.min(nb);
    for bi in 0..br_end {
        let b = &app.branches[bi];
        let marker = if app.mode == crate::app::AppMode::SidebarFocus && bi == app.branch_idx {
            format!("{}{}▸{}", ansi::BLD, ansi::LGR, ansi::RST)
        } else {
            " ".to_string()
        };
        let cur = if b.name == app.current_branch {
            format!("{}*{}", ansi::DIM, ansi::RST)
        } else {
            " ".to_string()
        };
        let filt = if b.name == app.branch_filter {
            format!("{}✓{}", ansi::YEL, ansi::RST)
        } else {
            String::new()
        };
        let br_ansi = format!("\x1b[{}m", color_to_ansi_num(b.color));
        let name_display: String = b.name.chars().take(20).collect();
        lines.push(format!(
            " {}{}◉{} {}{} {}{}",
            marker, br_ansi, ansi::RST, cur, name_display, ansi::RST, filt
        ));
    }
    while lines.len() < br_vis {
        lines.push(String::new());
    }

    // Separator
    lines.push(format!(
        " {}┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄{}",
        ansi::DIM, ansi::RST
    ));

    // Authors header
    lines.push(format!(
        " {}{}AUTHORS{} {}({}){}",
        ansi::BLD, ansi::LMA, ansi::RST, ansi::DIM, na, ansi::RST
    ));

    let mc = app.authors.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let bar_total = (inner.saturating_sub(18)).max(5);

    for ai in 0..na.min(au_vis.saturating_sub(1)) {
        let (name, cnt) = &app.authors[ai];
        let bl = cnt * bar_total / mc.max(1);
        let bar: String = "▓".repeat(bl);
        let emp: String = "░".repeat(bar_total.saturating_sub(bl));
        let bcol = crate::app::BRANCH_COLORS[ai % 8];
        let bcol_ansi = format!("\x1b[{}m", color_to_ansi_num(bcol));
        let aname: String = name.chars().take(12).collect();
        lines.push(format!(
            " {}{:<12}{} {}{}{}{}",
            bcol_ansi, aname, ansi::RST, bcol_ansi, bar, ansi::RST, emp
        ));
    }
    while lines.len() < vis {
        lines.push(String::new());
    }

/// Build sidebar content for the issues view.
fn build_issues_sidebar(app: &AppState, vis: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let data_lines = if app.issues_lines_full.len() > 2 {
        &app.issues_lines_full[2..]
    } else {
        &[]
    };

    let mut open_count = 0usize;
    let mut closed_count = 0usize;
    let mut issues_summary: Vec<(u64, String, bool)> = Vec::new(); // (number, title, is_open)

    for line in data_lines {
        let line = line.as_str();
        // Skip label continuation lines
        if line.starts_with("  ↳") {
            continue;
        }
        // Parse: "  #N  OPEN   title..." or "  #N  CLOSED title..."
        if let Some(rest) = line.strip_prefix("  #") {
            let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = num_str.parse::<u64>() {
                let after_num = &rest[num_str.len()..]; // "  OPEN   ..." or "  CLOSED ..."
                let trimmed = after_num.trim_start();
                let is_open = trimmed.starts_with("OPEN");
                let is_closed = trimmed.starts_with("CLOSED");

                if is_open {
                    open_count += 1;
                } else if is_closed {
                    closed_count += 1;
                }

                // Extract title (skip state label and any ANSI codes)
                let title_start = if is_open {
                    trimmed.trim_start_matches("OPEN")
                } else if is_closed {
                    trimmed.trim_start_matches("CLOSED")
                } else {
                    trimmed
                };
                // Strip ANSI codes and leading whitespace
                let title_clean = strip_ansi(title_start).trim().to_string();
                issues_summary.push((num, title_clean, is_open));
            }
        }
    }

    // State pill
    let state_label = match app.issues_state.as_str() {
        "closed" => "CLOSED",
        "all" => "ALL",
        _ => "OPEN",
    };
    let state_color = match app.issues_state.as_str() {
        "closed" => ansi::LRE,
        "all" => ansi::CYN,
        _ => ansi::LGR,
    };

    let total_issues = open_count + closed_count;
    lines.push(format!(
        " {}{}ISSUES{} {}({}){}  {}{}{}",
        ansi::BLD, ansi::LBL, ansi::RST,
        ansi::DIM, total_issues, ansi::RST,
        state_color, state_label, ansi::RST,
    ));

    // Open/Closed breakdown
    lines.push(format!(
        "  {}{}●{} {}open   {}{}●{} {}closed{}",
        ansi::LGR, ansi::BLD, ansi::RST, open_count,
        ansi::LRE, ansi::BLD, ansi::RST, closed_count,
        ansi::RST
    ));

    // Separator
    lines.push(format!(
        " {}┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄{}",
        ansi::DIM, ansi::RST
    ));

    // Recent issues header
    lines.push(format!(
        " {}{}RECENT{}",
        ansi::BLD, ansi::LMA, ansi::RST,
    ));

    // Calculate how many issues fit in remaining space
    let remaining = vis.saturating_sub(lines.len()).saturating_sub(2); // 2 for padding
    let show_count = remaining.min(issues_summary.len());

    for i in 0..show_count {
        let (num, title, is_open) = &issues_summary[i];
        let state_dot = if *is_open {
            format!("{}{}●{}", ansi::LGR, ansi::BLD, ansi::RST)
        } else {
            format!("{}{}●{}", ansi::LRE, ansi::BLD, ansi::RST)
        };
        let title_trunc: String = title.chars().take(18).collect();
        if title.len() > 18 {
            lines.push(format!(
                " {} #{} {} {}…{}",
                state_dot, num, ansi::RST, title_trunc, ansi::RST
            ));
        } else {
            lines.push(format!(
                " {} #{} {} {}",
                state_dot, num, ansi::RST, title_trunc
            ));
        }
    }

    while lines.len() < vis {
        lines.push(String::new());
    }

    lines
}

/// Strip ANSI escape codes from a string.
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until 'm'
            for ec in &mut chars {
                if ec == 'm' {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}


    lines
}

