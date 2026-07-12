use std::collections::{HashMap, HashSet};
use std::io::{self, Write, stdout};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::Repository;

// ═══════════════════════════════════════════════════════════════════════════
// Colors
// ═══════════════════════════════════════════════════════════════════════════
const RST: &str = "\x1b[0m";
const BLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
#[allow(dead_code)]
const UND: &str = "\x1b[4m";
#[allow(dead_code)]
const RED: &str = "\x1b[31m";
const GRN: &str = "\x1b[32m";
const YEL: &str = "\x1b[33m";
#[allow(dead_code)]
const BLU: &str = "\x1b[34m";
#[allow(dead_code)]
const MAG: &str = "\x1b[35m";
const CYN: &str = "\x1b[36m";
const WHT: &str = "\x1b[37m";
const GRY: &str = "\x1b[90m";
const LRE: &str = "\x1b[91m";
const LGR: &str = "\x1b[92m";
const LYL: &str = "\x1b[93m";
const LBL: &str = "\x1b[94m";
const LMA: &str = "\x1b[95m";
const LCY: &str = "\x1b[96m";

const BRANCH_COLORS: [&str; 8] = [GRN, LCY, LYL, LMA, LRE, LBL, LGR, CYN];

// ═══════════════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Tree,
    TreeFilter,
    Diff,
    Files,
    Report,
    ReportFilter,
    SidebarFocus,
    Help,
}

struct CommitInfo {
    hash: String,
    parents: Vec<String>,
    author: String,
    date: String,
    subject: String,
}

struct BranchInfo {
    name: String,
    color: String,
    head_oid: String,
}

struct RenderLine {
    content: String,
    commit_idx: Option<usize>,
}

struct AppState {
    mode: AppMode,
    dirty: bool,
    refresh: bool,

    // Git data
    commits: Vec<CommitInfo>,
    index: HashMap<String, usize>,
    children: HashMap<String, Vec<String>>,
    branches: Vec<BranchInfo>,
    repo_name: String,
    current_branch: String,
    head_hash: String,
    total: usize,

    // Layout
    lanes: Vec<usize>,
    occupied: Vec<Option<String>>,
    max_lane: usize,
    nlanes: usize,

    // Render buffer
    render_lines: Vec<RenderLine>,

    // Cursor & scroll
    cursor: isize,
    scroll: usize,

    // Tree display
    count: usize,
    show_all: bool,
    show_meta: bool,
    compact: bool,

    // Filters
    filter_text: String,
    filter_input: String,
    branch_filter: String,
    descendant_filter: String,
    descendant_set: HashSet<String>,
    date_from: String,
    date_to: String,

    // Diff view
    diff_scroll: usize,
    diff_lines: Vec<String>,

    // Files view
    files_scroll: usize,
    files_lines: Vec<String>,

    // Report view
    report_scroll: usize,
    report_lines: Vec<String>,
    report_email_filter: String,
    report_email_input: String,
    report_sort: String,
    report_ac_idx: usize,
    report_ac_list: Vec<String>,

    // Sidebar
    branch_idx: usize,
    authors: Vec<(String, usize)>,

    // Terminal
    term_w: u16,
    term_h: u16,

    // Clipboard
    clipboard_cmd: Option<String>,

    // Message
    msg: String,
    msg_time: Option<Instant>,
}

// ═══════════════════════════════════════════════════════════════════════════
// ANSI Helpers
// ═══════════════════════════════════════════════════════════════════════════
fn vis_len(s: &str) -> usize {
    let mut len = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some('[') = chars.next() {
                for c in &mut chars {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else if c == '\n' || c == '\r' {
            continue;
        } else {
            len += 1;
        }
    }
    len
}

fn pad_to(s: &str, target: usize) -> String {
    let cur = vis_len(s);
    if cur < target {
        format!("{}{}", s, " ".repeat(target - cur))
    } else {
        s.to_string()
    }
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some('[') = chars.next() {
                for c in &mut chars {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

// ═══════════════════════════════════════════════════════════════════════════
// Date Formatting
// ═══════════════════════════════════════════════════════════════════════════
fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn secs_to_ymdhm(secs: i64) -> (i64, u32, u32, u32, u32) {
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hour = (remaining / 3600) as u32;
    let min = ((remaining % 3600) / 60) as u32;

    let mut year = 1970i64;
    let mut day_of_year = days;
    loop {
        let diy = if is_leap(year) { 366 } else { 365 };
        if day_of_year < diy {
            break;
        }
        day_of_year -= diy;
        year += 1;
    }
    let md = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u32;
    for &d in &md {
        if day_of_year < d as i64 {
            break;
        }
        day_of_year -= d as i64;
        month += 1;
    }
    (year, month, (day_of_year + 1) as u32, hour, min)
}

fn format_date_full(time: git2::Time) -> String {
    let secs = time.seconds();
    let offset = time.offset_minutes();
    let (y, mo, d, h, mi) = secs_to_ymdhm(secs);
    let tz_h = offset / 60;
    let tz_m = (offset.abs() % 60) as u32;
    let sign = if offset >= 0 { '+' } else { '-' };
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:00{}{:02}:{:02}",
        y, mo, d, h, mi, sign, tz_h.abs(), tz_m
    )
}

#[allow(dead_code)]
fn format_date_short(time: git2::Time) -> String {
    let secs = time.seconds();
    let (y, mo, d, _, _) = secs_to_ymdhm(secs);
    format!("{:04}-{:02}-{:02}", y, mo, d)
}

#[allow(dead_code)]
fn format_date_time_only(time: git2::Time) -> String {
    let secs = time.seconds();
    let (_, _, _, h, m) = secs_to_ymdhm(secs);
    format!("{:02}:{:02}", h, m)
}

// ═══════════════════════════════════════════════════════════════════════════
// Clipboard
// ═══════════════════════════════════════════════════════════════════════════
fn detect_clipboard() -> Option<String> {
    let cmds = ["xclip", "xsel", "wl-copy", "pbcopy", "termux-clipboard-set"];
    for cmd in &cmds {
        if Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(match *cmd {
                "xclip" => "xclip -selection clipboard".to_string(),
                "xsel" => "xsel --clipboard --input".to_string(),
                _ => cmd.to_string(),
            });
        }
    }
    None
}

fn copy_to_clipboard(cmd: &str, text: &str) -> bool {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }
    Command::new(parts[0])
        .args(&parts[1..])
        .stdin(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(text.as_bytes())?;
            child.wait()
        })
        .map(|s| s.success())
        .unwrap_or(false)
}

// ═══════════════════════════════════════════════════════════════════════════
// Git Operations
// ═══════════════════════════════════════════════════════════════════════════
fn fetch_data(app: &mut AppState) {
    app.commits.clear();
    app.index.clear();
    app.children.clear();
    app.branches.clear();
    app.lanes.clear();
    app.occupied.clear();
    app.max_lane = 0;
    app.nlanes = 1;
    app.authors.clear();

    let cwd = std::env::current_dir().unwrap_or_default();
    let repo = match Repository::discover(&cwd) {
        Ok(r) => r,
        Err(_) => {
            app.repo_name = "not a git repo".to_string();
            return;
        }
    };

    app.repo_name = std::path::Path::new(
        repo.workdir()
            .unwrap_or(std::path::Path::new("."))
    )
    .file_name()
    .map(|f| f.to_string_lossy().to_string())
    .unwrap_or_else(|| "repo".to_string());

    app.current_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "detached".to_string());

    app.head_hash = repo
        .head()
        .ok()
        .and_then(|h| h.target().map(|o| o.to_string()))
        .unwrap_or_default();

    // Branches
    if let Ok(brs) = repo.branches(Some(git2::BranchType::Local)) {
        let mut names: Vec<String> = Vec::new();
        for br in brs.flatten() {
            if let Ok(Some(name)) = br.0.name() {
                names.push(name.to_string());
            }
        }
        names.sort();
        for (i, name) in names.iter().enumerate() {
            let oid = repo
                .find_branch(name, git2::BranchType::Local)
                .ok()
                .and_then(|b| b.get().target());
            app.branches.push(BranchInfo {
                name: name.clone(),
                color: BRANCH_COLORS[i % 8].to_string(),
                head_oid: oid.map(|o| o.to_string()).unwrap_or_default(),
            });
        }
    }

    // Total commit count
    app.total = {
        match repo.revwalk() {
            Ok(mut rw) => {
                rw.set_sorting(git2::Sort::TIME).ok();
                if !app.branch_filter.is_empty() {
                    let _ = rw.push_ref(&format!("refs/heads/{}", app.branch_filter));
                } else if app.show_all {
                    let _ = rw.push_glob("refs/heads/*");
                } else {
                    let _ = rw.push_head();
                }
                rw.count()
            }
            Err(_) => 0,
        }
    };

    // Fetch commits
    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return,
    };
    revwalk.set_sorting(git2::Sort::TIME).ok();
    if !app.branch_filter.is_empty() {
        let _ = revwalk.push_ref(&format!("refs/heads/{}", app.branch_filter));
    } else if app.show_all {
        let _ = revwalk.push_glob("refs/heads/*");
    } else {
        let _ = revwalk.push_head();
    }
    let mut author_count: HashMap<String, usize> = HashMap::new();

    for (i, oid) in revwalk.take(app.count).enumerate() {
        let oid = match oid {
            Ok(o) => o,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let hash = commit.id().to_string();
        let parents: Vec<String> = commit.parents().map(|p| p.id().to_string()).collect();
        let author = commit
            .author()
            .name()
            .unwrap_or("Unknown")
            .to_string();
        let date = format_date_full(commit.time());
        let subject = commit
            .summary()
            .unwrap_or("")
            .to_string();

        *author_count.entry(author.clone()).or_insert(0) += 1;

        // Build children map
        for p in &parents {
            app.children
                .entry(p.clone())
                .or_default()
                .push(hash.clone());
        }

        app.index.insert(hash.clone(), i);
        app.commits.push(CommitInfo {
            hash,
            parents,
            author,
            date,
            subject,
        });
    }

    // Sort authors by count descending
    let mut authors: Vec<(String, usize)> = author_count.into_iter().collect();
    authors.sort_by(|a, b| b.1.cmp(&a.1));
    app.authors = authors;

    // Lane assignment (O(n^2) but same as bash version)
    compute_lanes(app);
}

fn compute_lanes(app: &mut AppState) {
    app.lanes = vec![0; app.commits.len()];
    app.occupied = Vec::new();
    app.max_lane = 0;

    for i in 0..app.commits.len() {
        let hash = &app.commits[i].hash;

        // Find first unoccupied lane
        let mut ln = None;
        for l in 0..app.occupied.len() {
            if app.occupied[l].is_none() {
                ln = Some(l);
                break;
            }
        }
        let ln = match ln {
            Some(l) => l,
            None => {
                let l = app.occupied.len();
                app.occupied.push(None);
                l
            }
        };
        if ln >= app.occupied.len() {
            app.occupied.push(None);
        }
        app.occupied[ln] = Some(hash.clone());
        app.lanes[i] = ln;
        if ln > app.max_lane {
            app.max_lane = ln;
        }

        // Free lanes no longer needed
        for l in 0..app.occupied.len() {
            let oh = match &app.occupied[l] {
                Some(h) => h.clone(),
                None => continue,
            };
            if oh == *hash {
                continue;
            }
            let mut needed = false;
            'outer: for j in (i + 1)..app.commits.len() {
                for jp in &app.commits[j].parents {
                    if *jp == oh {
                        needed = true;
                        break 'outer;
                    }
                }
            }
            if !needed {
                app.occupied[l] = None;
            }
        }
    }
    app.nlanes = app.max_lane + 1;
}

// ═══════════════════════════════════════════════════════════════════════════
// Descendant Filter
// ═══════════════════════════════════════════════════════════════════════════
fn build_desc_set(app: &mut AppState) {
    app.descendant_set.clear();
    if app.descendant_filter.is_empty() {
        return;
    }
    let mut queue = vec![app.descendant_filter.clone()];
    app.descendant_set.insert(app.descendant_filter.clone());
    while let Some(cur) = queue.first().cloned() {
        queue.remove(0);
        if let Some(children) = app.children.get(&cur) {
            for c in children {
                if !app.descendant_set.contains(c) {
                    app.descendant_set.insert(c.clone());
                    queue.push(c.clone());
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Render Buffer
// ═══════════════════════════════════════════════════════════════════════════
fn build_render_buffer(app: &mut AppState) {
    app.render_lines.clear();

    if app.commits.is_empty() {
        app.render_lines.push(RenderLine {
            content: format!("  {}No commits to display{}", DIM, RST),
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
    let subj_w = tree_w.saturating_sub(prefix_w + hash_w + auth_w + date_w + 6).max(10);

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
        let dt: String = app.commits[i].date[..10.min(app.commits[i].date.len())].to_string();

        // Node character and color
        let (ncol, nchar) = if hash == &app.head_hash {
            (LRE, "▶")
        } else if np > 1 {
            (YEL, "◆")
        } else {
            // Check if this is a branch tip
            let mut tip_col = GRN;
            let mut tip_char = "●";
            for br in &app.branches {
                if br.head_oid == *hash {
                    tip_col = &br.color;
                    tip_char = "◉";
                    break;
                }
            }
            (tip_col, tip_char)
        };

        // Graph
        let mut graph = String::new();
        for l in 0..app.nlanes {
            if l == lane {
                graph.push_str(&format!("{}{}{}{}", ncol, BLD, nchar, RST));
            } else {
                let oh = app.occupied.get(l).and_then(|o| o.as_ref());
                let mut show = false;
                if let Some(oh_hash) = oh {
                    'outer: for j in (i + 1)..app.commits.len() {
                        for jp in &app.commits[j].parents {
                            if *jp == *oh_hash {
                                show = true;
                                break 'outer;
                            }
                        }
                    }
                }
                if !show {
                    for pp in parents {
                        if let Some(&pi) = app.index.get(pp) {
                            if app.lanes[pi] == l {
                                show = true;
                                break;
                            }
                        }
                    }
                }
                if show {
                    graph.push_str(&format!("{}│{}", DIM, RST));
                } else {
                    graph.push(' ');
                }
            }
        }

        // Merge connectors
        if np > 1 {
            for pi in 1..np {
                let pp = &parents[pi];
                if let Some(&pp_idx) = app.index.get(pp) {
                    let pl = app.lanes[pp_idx];
                    if pl < lane {
                        graph.push_str(&format!("{}╮{}", YEL, RST));
                    } else if pl > lane {
                        graph.push_str(&format!("{}╭{}", YEL, RST));
                    }
                }
            }
        }

        // Pad graph
        let gv = vis_len(&graph);
        if gv < ga_w {
            graph.push_str(&" ".repeat(ga_w - gv));
        }

        // Build box
        let mut box_str = format!("{}╰─{} {}{}{}{}", ncol, RST, BLD, CYN, short, RST);

        // Branch refs
        for br in &app.branches {
            if br.head_oid == *hash {
                let cu = if br.name == app.current_branch {
                    BLD
                } else {
                    ""
                };
                box_str.push_str(&format!(
                    " {}{}{}{}{}",
                    br.color, cu, br.name, RST, ""
                ));
            }
        }

        box_str.push_str(&format!(
            "  {}{}{}  {}{}{} {}· {}{}",
            WHT, subj_truncated, RST, DIM, auth, RST, GRY, dt, RST
        ));

        let prefix = format!("  {:3} ", i);
        app.render_lines.push(RenderLine {
            content: format!("{}{} {}", prefix, graph, box_str),
            commit_idx: Some(i),
        });

        // Meta line
        if app.show_meta {
            let mindent = " ".repeat(4 + ga_w + 2);
            let mut mline = format!(
                "{}{}{}  {}{}{}",
                mindent, GRY, hash, GRY,
                &app.commits[i].date[11..19.min(app.commits[i].date.len())],
                RST
            );
            if np > 1 {
                mline.push_str(&format!("  {}⟶ merge{}", YEL, RST));
            } else if np == 0 {
                mline.push_str(&format!("  {}⟶ root{}", GRN, RST));
            }
            app.render_lines.push(RenderLine {
                content: mline,
                commit_idx: Some(i),
            });
        }

        // Connector line
        if !app.compact && i < app.commits.len() - 1 {
            let mut conn = String::new();
            for l in 0..app.nlanes {
                if l == lane {
                    conn.push_str(&format!("{}│{}", DIM, RST));
                } else {
                    let oh = app.occupied.get(l).and_then(|o| o.as_ref());
                    let mut show = false;
                    if let Some(oh_hash) = oh {
                        'outer2: for j in (i + 1)..app.commits.len() {
                            for jp in &app.commits[j].parents {
                                if *jp == *oh_hash {
                                    show = true;
                                    break 'outer2;
                                }
                            }
                        }
                    }
                    if show {
                        conn.push_str(&format!("{}│{}", DIM, RST));
                    } else {
                        conn.push(' ');
                    }
                }
            }
            let cv = vis_len(&conn);
            if cv < ga_w {
                conn.push_str(&" ".repeat(ga_w - cv));
            }
            app.render_lines.push(RenderLine {
                content: format!("     {}", conn),
                commit_idx: Some(i),
            });
        }
    }
}

fn compute_tree_w(app: &AppState) -> usize {
    let sidebar_w = if app.term_w >= 60 { 32 } else { 0 };
    let tw = (app.term_w as usize).saturating_sub(sidebar_w + 1);
    tw.max(20)
}

// ═══════════════════════════════════════════════════════════════════════════
// Sidebar
// ═══════════════════════════════════════════════════════════════════════════
fn build_sidebar(app: &AppState, vis: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let nb = app.branches.len();
    let na = app.authors.len();
    let half = vis / 2;
    let br_vis = half.max(3);
    let au_vis = vis.saturating_sub(br_vis);
    let inner: usize = 30; // sidebar_w - 2

    let sep: String = "─".repeat(32);

    lines.push(format!(
        " {}{}Branches{}{}{}({}){}",
        BLD, LBL, RST, DIM, "", nb, RST
    ));

    let br_end = br_vis.min(nb);
    for bi in 0..br_end {
        let b = &app.branches[bi];
        let marker = if app.mode == AppMode::SidebarFocus && bi == app.branch_idx {
            format!("{}{}▸{} ", BLD, LGR, RST)
        } else {
            "  ".to_string()
        };
        let cur = if b.name == app.current_branch {
            format!(" {}⊕{}", DIM, RST)
        } else {
            String::new()
        };
        let filt = if b.name == app.branch_filter {
            format!(" {}✓{}", YEL, RST)
        } else {
            String::new()
        };
        lines.push(format!(
            " {}{}◉{} {}{}{}{}{} {}",
            marker, b.color, RST, BLD, b.name, RST, cur, filt, ""
        ));
    }
    while lines.len() < br_vis {
        lines.push(String::new());
    }

    lines.push(format!("{}{}{}", DIM, sep, RST));

    lines.push(format!(
        " {}{}Authors{}{}{}({}){}",
        BLD, LMA, RST, DIM, "", na, RST
    ));

    let mc = app.authors.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let bar_total = (inner.saturating_sub(18)).max(5);

    for ai in 0..na.min(au_vis.saturating_sub(1)) {
        let (name, cnt) = &app.authors[ai];
        let bl = cnt * bar_total / mc.max(1);
        let bar: String = "█".repeat(bl);
        let emp: String = "░".repeat(bar_total.saturating_sub(bl));
        let bcol = BRANCH_COLORS[ai % 8];
        let aname: String = name.chars().take(12).collect();
        lines.push(format!(
            " {}{:<12}{} {}{}{}{} {}{}",
            bcol, aname, RST, bcol, bar, RST, DIM, emp, RST
        ));
    }
    while lines.len() < vis {
        lines.push(String::new());
    }

    lines
}

// ═══════════════════════════════════════════════════════════════════════════
// Drawing
// ═══════════════════════════════════════════════════════════════════════════
fn build_status_bar(app: &AppState) -> String {
    let mode_label = match app.mode {
        AppMode::Diff => "diff",
        AppMode::Files => "files",
        AppMode::Report | AppMode::ReportFilter => "report",
        _ => "tree",
    };
    let mut s = format!(
        " {}{repo}{sep}{}{branch}{sep}{}{total} commits{sep}{}{mode}{}",
        BLD, BLD, BLD, BLD, RST,
        repo = format!("{}{}{}", CYN, app.repo_name, RST),
        sep = format!("{} │ {}", DIM, RST),
        branch = format!("{}{}{}", GRN, app.current_branch, RST),
        total = format!("{}{}", WHT, app.total),
        mode = format!("{}{}", LCY, mode_label),
    );
    if !app.filter_text.is_empty() {
        s.push_str(&format!("{} │ {}{}{}{}", DIM, RST, BLD, LGR, RST));
        s.push_str(&format!("/{}", app.filter_text));
    }
    if !app.date_from.is_empty() || !app.date_to.is_empty() {
        s.push_str(&format!(
            "{} │ {}📅{} {}→{}",
            DIM, RST, BLD, app.date_from, app.date_to
        ));
    }
    if !app.descendant_filter.is_empty() {
        s.push_str(&format!(
            "{} │ {}▼{} zoom:{}",
            DIM, RST, BLD, &app.descendant_filter[..7.min(app.descendant_filter.len())]
        ));
    }
    if app.show_all {
        s.push_str(&format!("{} │ {}ALL{}", DIM, RST, BLD));
    }
    if app.show_meta {
        s.push_str(&format!("{} │ {}META{}", DIM, RST, BLD));
    }
    if app.compact {
        s.push_str(&format!("{} │ {}CMP{}", DIM, RST, BLD));
    }
    s
}

fn build_hr(term_w: u16) -> String {
    format!("{}{}{}", DIM, "─".repeat(term_w as usize), RST)
}

fn draw_tree(app: &AppState) -> String {
    let mut out = String::new();
    out.push_str("\x1b[H\x1b[2J"); // clear screen

    // Status bar
    out.push_str(&build_status_bar(app));
    out.push('\n');

    // Filter input
    if app.mode == AppMode::TreeFilter {
        out.push_str(&format!(
            "  {}{}{}{}{}█ Esc to cancel{}\n",
            BLD, LGR, app.filter_input, RST, DIM, RST
        ));
    }

    // HR
    out.push_str(&build_hr(app.term_w));
    out.push('\n');

    let mut vis = (app.term_h as usize).saturating_sub(5);
    if app.mode == AppMode::TreeFilter {
        vis = vis.saturating_sub(1);
    }
    vis = vis.max(1);

    let render_n = app.render_lines.len();
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

    if app.mode == AppMode::Help {
        out.push_str(&draw_help(app, vis));
        return out;
    }

    let sidebar_w = if app.term_w >= 60 { 32 } else { 0 };
    let tree_w = compute_tree_w(app);

    if sidebar_w > 0 {
        let sidebar = build_sidebar(app, vis);
        let vl = format!("{}│{}", DIM, RST);
        let mut drawn = 0;
        for idx in scroll..render_n {
            if drawn >= vis {
                break;
            }
            let mut tl = pad_to(&app.render_lines[idx].content, tree_w);
            if idx == app.cursor as usize {
                // Overlay cursor highlight
                let stripped = strip_ansi(&tl);
                let rest: String = stripped.chars().skip(1).collect();
                tl = format!("{}▸{}{}", YEL, rest, RST);
            }
            let sl = sidebar.get(drawn).map(|s| s.as_str()).unwrap_or("");
            let sl_padded = pad_to(sl, sidebar_w);
            out.push_str(&format!("{}{}{}\n", tl, vl, sl_padded));
            drawn += 1;
        }
        for _ in drawn..vis {
            let padding = " ".repeat(tree_w);
            let sl = sidebar.get(drawn).map(|s| s.as_str()).unwrap_or("");
            let sl_padded = pad_to(sl, sidebar_w);
            out.push_str(&format!("{}{}{}\n", padding, vl, sl_padded));
            drawn += 1;
        }
    } else {
        let mut drawn = 0;
        for idx in scroll..render_n {
            if drawn >= vis {
                break;
            }
            if idx == app.cursor as usize {
                let stripped = strip_ansi(&app.render_lines[idx].content);
                let rest: String = stripped.chars().skip(1).collect();
                out.push_str(&format!("{}▸{}{}\n", YEL, rest, RST));
            } else {
                out.push_str(&format!("{}\n", app.render_lines[idx].content));
            }
            drawn += 1;
        }
        for _ in drawn..vis {
            out.push('\n');
        }
    }

    // HR
    out.push_str(&build_hr(app.term_w));
    out.push('\n');

    // Message or footer
    if let Some(ref msg_time) = app.msg_time {
        if !app.msg.is_empty() && msg_time.elapsed() < Duration::from_secs(2) {
            out.push_str(&format!("  {}{}{}{}\n", BLD, LGR, app.msg, RST));
        }
    }

    // Footer
    if app.mode == AppMode::SidebarFocus {
        out.push_str(&format!(
            "  {}j/k{} ↕ select  {}Enter{} confirm  {}0{} reset  {}Esc/b{} cancel{}\n",
            DIM, RST, DIM, RST, DIM, RST, DIM, RST, RST
        ));
    } else {
        out.push_str(&format!(
            "  {}j/k{} ↕  {}d{} diff  {}f{} files  {}y{} copy  {}D{} date  {}R{} report  {}/{} filter  {}?{} help  {}q{} quit{}\n",
            DIM, RST, DIM, RST, DIM, RST, DIM, RST, DIM, RST, DIM, RST, DIM, RST, DIM, RST, DIM, RST, RST
        ));
    }

    out
}

fn draw_help(app: &AppState, vis: usize) -> String {
    let mut out = String::new();
    let hx = ((app.term_w as usize).saturating_sub(54)) / 2;
    let hx = hx.max(2);
    let pfx = " ".repeat(hx);

    out.push('\n');
    out.push_str(&format!("{}{}╔════════════════════════════════════════════════════╗{}\n", pfx, CYN, RST));
    out.push_str(&format!("{}{}║{}  {}gitscope - Interactive Git Tree Viewer{}          {}{}║{}\n", pfx, CYN, RST, BLD, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}╠════════════════════════════════════════════════════╣{}\n", pfx, CYN, RST));
    out.push_str(&format!("{}{}║{}  {}Navigation{}                                      {}{}║{}\n", pfx, CYN, RST, BLD, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}j{} / {}↓{}       Move cursor down                  {}{}║{}\n", pfx, CYN, RST, LGR, RST, LGR, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}k{} / {}↑{}       Move cursor up                    {}{}║{}\n", pfx, CYN, RST, LGR, RST, LGR, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}J{} / {}K{}       Move 5 lines                     {}{}║{}\n", pfx, CYN, RST, LGR, RST, LGR, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}g{} / {}G{}       Top / Bottom                      {}{}║{}\n", pfx, CYN, RST, LGR, RST, LGR, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}PgUp/PgDn{}   Page up/down                      {}{}║{}\n", pfx, CYN, RST, LGR, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}╠════════════════════════════════════════════════════╣{}\n", pfx, CYN, RST));
    out.push_str(&format!("{}{}║{}  {}Actions{}                                          {}{}║{}\n", pfx, CYN, RST, BLD, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}Enter/o{}     Zoom into commit descendants      {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}d{}           View commit diff                  {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}f{}           View changed files                 {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}y{}           Copy commit hash                   {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}/{}           Filter commits (regex)            {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}D{}           Toggle last-7-day date filter      {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}R{}           Report: files by author            {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}r{}           Refresh data                      {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}╠════════════════════════════════════════════════════╣{}\n", pfx, CYN, RST));
    out.push_str(&format!("{}{}║{}  {}Display{}                                          {}{}║{}\n", pfx, CYN, RST, BLD, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}a{}           Toggle all branches                {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}m{}           Toggle commit metadata             {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}c{}           Toggle compact mode                {}{}║{}\n", pfx, CYN, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}+{} / {}-{}         ±10 commits                       {}{}║{}\n", pfx, CYN, RST, LYL, RST, LYL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}╠════════════════════════════════════════════════════╣{}\n", pfx, CYN, RST));
    out.push_str(&format!("{}{}║{}  {}Sidebar{}                                           {}{}║{}\n", pfx, CYN, RST, BLD, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}b{}           Focus branch picker (sidebar)     {}{}║{}\n", pfx, CYN, RST, LBL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}0{}           Reset all filters                 {}{}║{}\n", pfx, CYN, RST, LBL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}ESC{}          Clear zoom filter                  {}{}║{}\n", pfx, CYN, RST, LBL, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}╠════════════════════════════════════════════════════╣{}\n", pfx, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}?{}           Toggle this help                  {}{}║{}\n", pfx, CYN, RST, LRE, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}║{}    {}q{} / {}Ctrl-C{}    Quit                              {}{}║{}\n", pfx, CYN, RST, LRE, RST, LRE, RST, BLD, CYN, RST));
    out.push_str(&format!("{}{}╚════════════════════════════════════════════════════╝{}\n", pfx, CYN, RST));

    let remain = vis.saturating_sub(22);
    for _ in 0..remain {
        out.push('\n');
    }
    out.push_str(&build_hr(app.term_w));
    out.push('\n');
    out.push_str(&format!(
        "  {}?{} close  {}q{} quit{}\n",
        DIM, RST, DIM, RST, RST
    ));
    out
}

fn draw_diff(app: &AppState) -> String {
    let mut out = String::new();
    out.push_str("\x1b[H\x1b[2J");
    out.push_str(&build_status_bar(app));
    out.push('\n');
    out.push_str(&build_hr(app.term_w));
    out.push('\n');

    let vis = (app.term_h as usize).saturating_sub(5);
    let total = app.diff_lines.len();
    let max_s = total.saturating_sub(vis);
    let scroll = app.diff_scroll.min(max_s);

    let mut drawn = 0;
    for idx in scroll..total {
        if drawn >= vis {
            break;
        }
        let line = &app.diff_lines[idx];
        if line.contains("diff --git") {
            out.push_str(&format!("{}{}{}{}\n", BLD, LGR, line, RST));
        } else if line.contains("@@") {
            out.push_str(&format!("{}{}{}{}\n", BLD, CYN, line, RST));
        } else if (line.contains('+') && !line.contains("+++")) || (line.starts_with('+') && !line.starts_with("+++")) {
            out.push_str(&format!("{}{}{}\n", LGR, line, RST));
        } else if (line.contains('-') && !line.contains("---")) || (line.starts_with('-') && !line.starts_with("---")) {
            out.push_str(&format!("{}{}{}\n", LRE, line, RST));
        } else {
            out.push_str(&format!("{}\n", line));
        }
        drawn += 1;
    }
    for _ in drawn..vis {
        out.push('\n');
    }

    out.push_str(&build_hr(app.term_w));
    out.push('\n');
    out.push_str(&format!(
        "  {}j/k{} ↕ scroll  {}q/Esc{} close{}\n",
        DIM, RST, DIM, RST, RST
    ));
    out
}

fn draw_files(app: &AppState) -> String {
    let mut out = String::new();
    out.push_str("\x1b[H\x1b[2J");
    out.push_str(&build_status_bar(app));
    out.push('\n');
    out.push_str(&build_hr(app.term_w));
    out.push('\n');

    let vis = (app.term_h as usize).saturating_sub(5);
    let total = app.files_lines.len();
    let max_s = total.saturating_sub(vis);
    let scroll = app.files_scroll.min(max_s);

    let mut drawn = 0;
    for idx in scroll..total {
        if drawn >= vis {
            break;
        }
        let line = &app.files_lines[idx];
        if line.contains('│') {
            out.push_str(&format!("  {}{}{}\n", DIM, line, RST));
        } else if line.contains("file changed") || line.contains("files changed") {
            out.push_str(&format!("{}{}{}\n", BLD, line, RST));
        } else {
            out.push_str(&format!("  {}\n", line));
        }
        drawn += 1;
    }
    for _ in drawn..vis {
        out.push('\n');
    }

    out.push_str(&build_hr(app.term_w));
    out.push('\n');
    out.push_str(&format!(
        "  {}j/k{} ↕ scroll  {}q/Esc{} close{}\n",
        DIM, RST, DIM, RST, RST
    ));
    out
}

fn draw_report(app: &AppState) -> String {
    let mut out = String::new();
    out.push_str("\x1b[H\x1b[2J");
    out.push_str(&build_status_bar(app));

    if app.mode == AppMode::ReportFilter {
        out.push('\n');
        out.push_str(&format!(
            "  {}{}{}{}{}█ Esc to cancel  Tab to select{}\n",
            BLD, LGR, app.report_email_input, RST, DIM, RST
        ));
        let ac_total = app.report_ac_list.len();
        if ac_total > 0 {
            out.push_str(&format!("  {}{} match(es){}\n", DIM, ac_total, RST));
            for (idx, sug) in app.report_ac_list.iter().enumerate() {
                let prefix_len = sug
                    .to_lowercase()
                    .find(&app.report_email_input.to_lowercase())
                    .unwrap_or(0);
                let prefix = &sug[..prefix_len];
                let match_str = &sug[prefix_len..prefix_len + app.report_email_input.len()];
                let rest = &sug[prefix_len + app.report_email_input.len()..];
                let marker = if idx == app.report_ac_idx {
                    format!("{}{}▸{} ", BLD, LGR, RST)
                } else {
                    "  ".to_string()
                };
                out.push_str(&format!(
                    "  {}{}{}{}{}{}{}{}{}\n",
                    marker, DIM, prefix, RST, BLD, LGR, match_str, RST, ""
                ));
                if !rest.is_empty() {
                    out.push_str(&format!("{}{}{}{}", DIM, rest, RST, ""));
                }
            }
        } else if !app.report_email_input.is_empty() {
            out.push_str(&format!("  {}  no matches{}\n", DIM, RST));
        }
    }

    out.push_str(&build_hr(app.term_w));
    out.push('\n');

    let mut vis = (app.term_h as usize).saturating_sub(5);
    if app.mode == AppMode::ReportFilter {
        vis = vis.saturating_sub(1);
        let ac_count = app.report_ac_list.len();
        if ac_count > 0 {
            vis = vis.saturating_sub(ac_count + 1);
        } else if !app.report_email_input.is_empty() {
            vis = vis.saturating_sub(1);
        }
    }
    vis = vis.max(1);

    let total = app.report_lines.len();
    let max_s = total.saturating_sub(vis);
    let scroll = app.report_scroll.min(max_s);

    let mut drawn = 0;
    for idx in scroll..total {
        if drawn >= vis {
            break;
        }
        out.push_str(&format!("{}\n", app.report_lines[idx]));
        drawn += 1;
    }
    for _ in drawn..vis {
        out.push('\n');
    }

    out.push_str(&build_hr(app.term_w));
    out.push('\n');
    out.push_str(&format!(
        "  {}j/k{} ↕  {}g/G{} top/bot  {}/{} filter  {}t{} sort  {}b{} branch  {}q/Esc{} close{}\n",
        DIM, RST, DIM, RST, DIM, RST, DIM, RST, DIM, RST, DIM, RST, RST
    ));
    out
}

fn draw(app: &AppState) -> String {
    match app.mode {
        AppMode::Diff => draw_diff(app),
        AppMode::Files => draw_files(app),
        AppMode::Report | AppMode::ReportFilter => draw_report(app),
        _ => draw_tree(app),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Diff / Files / Report Openers
// ═══════════════════════════════════════════════════════════════════════════
fn open_diff(app: &mut AppState, ci: usize) {
    if ci >= app.commits.len() {
        return;
    }
    let hash = &app.commits[ci].hash;
    let output = Command::new("git")
        .args([
            "show",
            "--format=%C(yellow)commit %h%Creset%n%an <%ae> %C(dim)%ai%Creset%n%n%s%n",
            "--stat",
            hash,
        ])
        .output();
    if let Ok(o) = output {
        app.diff_lines = String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
    } else {
        app.diff_lines = vec!["  Failed to load diff".to_string()];
    }
    app.diff_scroll = 0;
    app.mode = AppMode::Diff;
}

fn open_files(app: &mut AppState, ci: usize) {
    if ci >= app.commits.len() {
        return;
    }
    let hash = &app.commits[ci].hash;
    let output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "-r", "--stat", hash])
        .output();
    if let Ok(o) = output {
        app.files_lines = String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
    } else {
        app.files_lines = vec!["  Failed to load files".to_string()];
    }
    app.files_scroll = 0;
    app.mode = AppMode::Files;
}

fn open_report(app: &mut AppState) {
    let output = Command::new("git")
        .args(["log", "--format=COMMIT:%ae|%ad", "--date=short", "--name-only"])
        .output();

    let stdout = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => {
            app.report_lines = vec!["  No file data found".to_string()];
            app.report_scroll = 0;
            app.mode = AppMode::Report;
            return;
        }
    };

    // Parse: commit line -> email|date, then non-empty lines are filenames
    let mut entries: Vec<(String, String, String)> = Vec::new();
    let mut cur_email = String::new();
    let mut cur_date = String::new();

    for line in stdout.lines() {
        if line.starts_with("COMMIT:") {
            let rest = &line[7..];
            if let Some((email, date)) = rest.split_once('|') {
                cur_email = email.to_string();
                cur_date = date.to_string();
            }
        } else if line.is_empty() {
            continue;
        } else {
            entries.push((cur_email.clone(), line.to_string(), cur_date.clone()));
        }
    }

    // Dedup
    entries.sort();
    entries.dedup();

    // Filter
    if !app.report_email_filter.is_empty() {
        let filter = app.report_email_filter.to_lowercase();
        entries.retain(|(email, _, _)| email.to_lowercase().contains(&filter));
        if entries.is_empty() {
            app.report_lines = vec![format!(
                "  {}No files found for: {}{}",
                DIM, app.report_email_filter, RST
            )];
            app.report_scroll = 0;
            app.mode = AppMode::Report;
            return;
        }
    }

    // Sort by date
    if app.report_sort == "date" {
        entries.sort_by(|a, b| b.2.cmp(&a.2));
    }

    // Build lines
    app.report_lines.clear();
    app.report_lines
        .push(format!("={}={:=<60}=", "", ""));
    app.report_lines
        .push("                    FILES BY AUTHOR".to_string());
    app.report_lines
        .push(format!("={}={:=<60}=", "", ""));

    let mut cur_email = String::new();
    for (email, filename, date) in &entries {
        if email != &cur_email {
            app.report_lines.push(String::new());
            app.report_lines
                .push("------------------------------------------------------------".to_string());
            app.report_lines
                .push(format!("  CREATED BY: {}{}{}", CYN, email, RST));
            app.report_lines
                .push("------------------------------------------------------------".to_string());
            app.report_lines.push(format!(
                "  {}FILE NAME{}{}{}DATE{}",
                BLD, RST,
                " ".repeat(38),
                BLD, RST
            ));
            app.report_lines.push(format!(
                "{}------------------------------------------------------------{}",
                DIM, RST
            ));
            cur_email = email.clone();
        }

        let bname = std::path::Path::new(filename)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| filename.to_string());
        let padding = 48usize.saturating_sub(bname.len()).max(2);
        app.report_lines.push(format!(
            "  {}{}{}{}{}{}",
            LYL, bname, RST, " ".repeat(padding), DIM, date
        ));
    }

    app.report_lines.push(String::new());
    app.report_lines
        .push(format!("={}={:=<60}=", "", ""));

    app.report_scroll = 0;
    app.mode = AppMode::Report;
}

// ═══════════════════════════════════════════════════════════════════════════
// Input Handling
// ═══════════════════════════════════════════════════════════════════════════
fn show_msg(app: &mut AppState, msg: &str) {
    app.msg = msg.to_string();
    app.msg_time = Some(Instant::now());
}

fn handle_key(app: &mut AppState, key: event::KeyEvent) -> bool {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return false;
    }

    match &app.mode {
        // ─── Tree Filter Mode ───
        AppMode::TreeFilter => match key.code {
            KeyCode::Enter => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            KeyCode::Esc => {
                app.filter_input.clear();
                app.filter_text.clear();
                app.mode = AppMode::Tree;
                build_render_buffer(app);
                app.dirty = true;
            }
            KeyCode::Backspace => {
                app.filter_input.pop();
                app.filter_text = app.filter_input.clone();
                app.scroll = 0;
                build_render_buffer(app);
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                app.filter_input.push(c);
                app.filter_text = app.filter_input.clone();
                app.scroll = 0;
                build_render_buffer(app);
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Diff Mode ───
        AppMode::Diff => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.diff_scroll += 1;
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.diff_scroll = app.diff_scroll.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.diff_scroll += 5;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.diff_scroll = app.diff_scroll.saturating_sub(5);
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Files Mode ───
        AppMode::Files => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.files_scroll += 1;
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.files_scroll = app.files_scroll.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.files_scroll += 5;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.files_scroll = app.files_scroll.saturating_sub(5);
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Report Filter Mode ───
        AppMode::ReportFilter => match key.code {
            KeyCode::Enter => {
                if let Some(first) = app.report_ac_list.first().cloned() {
                    app.report_email_input = first;
                }
                app.report_email_filter = app.report_email_input.clone();
                app.mode = AppMode::Report;
                open_report(app);
                app.dirty = true;
            }
            KeyCode::Tab => {
                if app.report_ac_list.len() == 1 {
                    app.report_email_input = app.report_ac_list[0].clone();
                    app.report_email_filter = app.report_email_input.clone();
                    app.mode = AppMode::Report;
                    open_report(app);
                    app.dirty = true;
                } else if app.report_ac_list.len() > 1 {
                    app.report_ac_idx += 1;
                    if app.report_ac_idx >= app.report_ac_list.len() {
                        app.report_ac_idx = 0;
                    }
                    app.report_email_input = app.report_ac_list[app.report_ac_idx].clone();
                    app.dirty = true;
                }
            }
            KeyCode::Esc => {
                app.report_email_input.clear();
                app.mode = AppMode::Report;
                app.dirty = true;
            }
            KeyCode::Backspace => {
                app.report_email_input.pop();
                app.report_ac_idx = 0;
                app.dirty = true;
            }
            KeyCode::Char(c) => {
                app.report_email_input.push(c);
                app.report_ac_idx = 0;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Report Mode ───
        AppMode::Report => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.report_scroll += 1;
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.report_scroll = app.report_scroll.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                app.report_scroll += 5;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                app.report_scroll = app.report_scroll.saturating_sub(5);
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.report_scroll = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.report_scroll = usize::MAX;
                app.dirty = true;
            }
            KeyCode::Char('/') => {
                app.report_email_input.clear();
                app.report_ac_idx = 0;
                app.report_ac_list.clear();
                app.mode = AppMode::ReportFilter;
                app.dirty = true;
            }
            KeyCode::Char('t') => {
                if app.report_sort == "name" {
                    app.report_sort = "date".to_string();
                } else {
                    app.report_sort = "name".to_string();
                }
                open_report(app);
                app.dirty = true;
            }
            KeyCode::Char('b') => {
                app.mode = AppMode::SidebarFocus;
                app.report_email_filter.clear();
                app.branch_idx = 0;
                app.dirty = true;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.report_email_filter.clear();
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Sidebar Focus ───
        AppMode::SidebarFocus => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.branch_idx += 1;
                if app.branch_idx >= app.branches.len() {
                    app.branch_idx = app.branches.len().saturating_sub(1);
                }
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.branch_idx = app.branch_idx.saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.branch_idx = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.branch_idx = app.branches.len().saturating_sub(1);
                app.dirty = true;
            }
            KeyCode::Enter => {
                if let Some(br) = app.branches.get(app.branch_idx) {
                    app.branch_filter = br.name.clone();
                }
                app.mode = AppMode::Tree;
                app.refresh = true;
            }
            KeyCode::Char('0') => {
                app.branch_filter.clear();
                app.mode = AppMode::Tree;
                app.refresh = true;
            }
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('q') => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Help ───
        AppMode::Help => match key.code {
            KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
                app.mode = AppMode::Tree;
                app.dirty = true;
            }
            _ => {}
        },

        // ─── Tree Mode (Normal) ───
        AppMode::Tree => match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return false,

            KeyCode::Char('j') | KeyCode::Down => {
                let ci = app
                    .render_lines
                    .get(app.cursor as usize)
                    .and_then(|rl| rl.commit_idx);
                let mut n = (app.cursor + 1) as usize;
                while n < app.render_lines.len() {
                    if app.render_lines[n].commit_idx != ci {
                        break;
                    }
                    n += 1;
                }
                if n < app.render_lines.len() {
                    app.cursor = n as isize;
                }
                app.dirty = true;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let ci = app
                    .render_lines
                    .get(app.cursor as usize)
                    .and_then(|rl| rl.commit_idx);
                let mut n = (app.cursor as usize).saturating_sub(1);
                loop {
                    if app.render_lines[n].commit_idx != ci {
                        break;
                    }
                    if n == 0 {
                        break;
                    }
                    n -= 1;
                }
                if app.render_lines[n].commit_idx != ci {
                    app.cursor = n as isize;
                }
                app.dirty = true;
            }
            KeyCode::Char('J') => {
                let mut cnt = 0;
                let mut ci = app
                    .render_lines
                    .get(app.cursor as usize)
                    .and_then(|rl| rl.commit_idx);
                let mut n = app.cursor as usize;
                while cnt < 5 && n < app.render_lines.len() - 1 {
                    n += 1;
                    if app.render_lines[n].commit_idx != ci {
                        ci = app.render_lines[n].commit_idx;
                        cnt += 1;
                    }
                }
                app.cursor = n as isize;
                app.dirty = true;
            }
            KeyCode::Char('K') => {
                let mut cnt = 0;
                let mut ci = app
                    .render_lines
                    .get(app.cursor as usize)
                    .and_then(|rl| rl.commit_idx);
                let mut n = app.cursor as usize;
                while cnt > 0 || n == app.cursor as usize {
                    if n == 0 {
                        break;
                    }
                    n -= 1;
                    if app.render_lines[n].commit_idx != ci {
                        ci = app.render_lines[n].commit_idx;
                        cnt += 1;
                    }
                    if cnt >= 5 {
                        break;
                    }
                }
                if n != app.cursor as usize {
                    app.cursor = n as isize;
                }
                app.dirty = true;
            }
            KeyCode::Char('g') => {
                app.cursor = 0;
                app.dirty = true;
            }
            KeyCode::Char('G') => {
                app.cursor = app.render_lines.len().saturating_sub(1) as isize;
                app.dirty = true;
            }
            KeyCode::PageUp => {
                app.cursor -= (app.term_h as isize) - 4;
                if app.cursor < 0 {
                    app.cursor = 0;
                }
                app.dirty = true;
            }
            KeyCode::PageDown => {
                app.cursor += (app.term_h as isize) - 4;
                let max = app.render_lines.len().saturating_sub(1) as isize;
                if app.cursor > max {
                    app.cursor = max;
                }
                app.dirty = true;
            }
            KeyCode::Home => {
                app.cursor = 0;
                app.dirty = true;
            }
            KeyCode::End => {
                app.cursor = app.render_lines.len().saturating_sub(1) as isize;
                app.dirty = true;
            }

            KeyCode::Char('a') => {
                app.show_all = !app.show_all;
                app.branch_filter.clear();
                app.filter_text.clear();
                app.descendant_filter.clear();
                app.refresh = true;
            }
            KeyCode::Char('m') => {
                app.show_meta = !app.show_meta;
                build_render_buffer(app);
                app.dirty = true;
            }
            KeyCode::Char('c') => {
                app.compact = !app.compact;
                build_render_buffer(app);
                app.dirty = true;
            }
            KeyCode::Char('+') => {
                app.count = (app.count + 10).min(200);
                app.refresh = true;
            }
            KeyCode::Char('-') => {
                if app.count > 10 {
                    app.count -= 10;
                    app.refresh = true;
                }
            }
            KeyCode::Char('b') => {
                app.mode = AppMode::SidebarFocus;
                app.branch_idx = 0;
                app.dirty = true;
            }
            KeyCode::Char('0') => {
                app.branch_filter.clear();
                app.filter_text.clear();
                app.descendant_filter.clear();
                app.date_from.clear();
                app.date_to.clear();
                app.refresh = true;
            }
            KeyCode::Char('/') => {
                app.filter_input.clear();
                app.mode = AppMode::TreeFilter;
                app.dirty = true;
            }

            KeyCode::Char('o') | KeyCode::Enter => {
                if app.cursor >= 0 && (app.cursor as usize) < app.render_lines.len() {
                    if let Some(ci) = app.render_lines[app.cursor as usize].commit_idx {
                        app.descendant_filter = app.commits[ci].hash.clone();
                        build_desc_set(app);
                        app.cursor = 0;
                        build_render_buffer(app);
                        app.dirty = true;
                    }
                }
            }

            KeyCode::Esc => {
                if !app.descendant_filter.is_empty() {
                    app.descendant_filter.clear();
                    app.cursor = 0;
                    build_render_buffer(app);
                    app.dirty = true;
                } else if !app.date_from.is_empty() || !app.date_to.is_empty() {
                    app.date_from.clear();
                    app.date_to.clear();
                    app.cursor = 0;
                    build_render_buffer(app);
                    app.dirty = true;
                }
            }

            KeyCode::Char('d') => {
                if app.cursor >= 0 {
                    if let Some(ci) = app
                        .render_lines
                        .get(app.cursor as usize)
                        .and_then(|rl| rl.commit_idx)
                    {
                        open_diff(app, ci);
                        app.dirty = true;
                    }
                }
            }
            KeyCode::Char('f') => {
                if app.cursor >= 0 {
                    if let Some(ci) = app
                        .render_lines
                        .get(app.cursor as usize)
                        .and_then(|rl| rl.commit_idx)
                    {
                        open_files(app, ci);
                        app.dirty = true;
                    }
                }
            }
            KeyCode::Char('y') => {
                if app.cursor >= 0 {
                    if let Some(ci) = app
                        .render_lines
                        .get(app.cursor as usize)
                        .and_then(|rl| rl.commit_idx)
                    {
                        if ci < app.commits.len() {
                            if let Some(ref cmd) = app.clipboard_cmd.clone() {
                                if copy_to_clipboard(cmd, &app.commits[ci].hash) {
                                    show_msg(
                                        app,
                                        &format!("Copied {} to clipboard", &app.commits[ci].hash[..7]),
                                    );
                                }
                            } else {
                                show_msg(app, "No clipboard tool found");
                            }
                        }
                    }
                }
                app.dirty = true;
            }

            KeyCode::Char('D') => {
                if !app.date_from.is_empty() || !app.date_to.is_empty() {
                    app.date_from.clear();
                    app.date_to.clear();
                    show_msg(app, "Date filter cleared");
                } else {
                    let today = std::process::Command::new("date")
                        .args(["+%Y-%m-%d"])
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    let week_ago = std::process::Command::new("date")
                        .args(["-d", &format!("{} - 7 days", today), "+%Y-%m-%d"])
                        .output()
                        .ok()
                        .and_then(|o| String::from_utf8(o.stdout).ok())
                        .map(|s| s.trim().to_string())
                        .or_else(|| {
                            std::process::Command::new("date")
                                .args(["-v-7d", "+%Y-%m-%d"])
                                .output()
                                .ok()
                                .and_then(|o| String::from_utf8(o.stdout).ok())
                                .map(|s| s.trim().to_string())
                        })
                        .unwrap_or_default();
                    if !week_ago.is_empty() {
                        app.date_from = week_ago.clone();
                        app.date_to = today;
                        show_msg(app, &format!("Last 7 days: {} → {}", week_ago, app.date_to));
                    }
                }
                app.cursor = 0;
                build_render_buffer(app);
                app.dirty = true;
            }

            KeyCode::Char('?') => {
                app.mode = AppMode::Help;
                app.dirty = true;
            }
            KeyCode::Char('R') => {
                open_report(app);
                app.dirty = true;
            }
            KeyCode::Char('r') => {
                app.refresh = true;
            }

            _ => {}
        },
    }

    // Update report autocomplete when in ReportFilter mode
    if app.mode == AppMode::ReportFilter && !app.report_email_input.is_empty() {
        let input_lower = app.report_email_input.to_lowercase();
        let output = Command::new("git")
            .args(["log", "--format=%ae"])
            .output();
        if let Ok(o) = output {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let mut matches: Vec<String> = stdout
                .lines()
                .map(|s| s.to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .filter(|e| e.to_lowercase().contains(&input_lower))
                .collect();
            matches.sort();
            matches.truncate(5);
            app.report_ac_list = matches;
        }
    }

    true
}

// ═══════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════
fn main() -> io::Result<()> {
    let mut app = AppState {
        mode: AppMode::Tree,
        dirty: true,
        refresh: true,

        commits: Vec::new(),
        index: HashMap::new(),
        children: HashMap::new(),
        branches: Vec::new(),
        repo_name: String::new(),
        current_branch: String::new(),
        head_hash: String::new(),
        total: 0,

        lanes: Vec::new(),
        occupied: Vec::new(),
        max_lane: 0,
        nlanes: 1,

        render_lines: Vec::new(),

        cursor: 0,
        scroll: 0,

        count: 30,
        show_all: false,
        show_meta: false,
        compact: false,

        filter_text: String::new(),
        filter_input: String::new(),
        branch_filter: String::new(),
        descendant_filter: String::new(),
        descendant_set: HashSet::new(),
        date_from: String::new(),
        date_to: String::new(),

        diff_scroll: 0,
        diff_lines: Vec::new(),

        files_scroll: 0,
        files_lines: Vec::new(),

        report_scroll: 0,
        report_lines: Vec::new(),
        report_email_filter: String::new(),
        report_email_input: String::new(),
        report_sort: "name".to_string(),
        report_ac_idx: 0,
        report_ac_list: Vec::new(),

        branch_idx: 0,
        authors: Vec::new(),

        term_w: 80,
        term_h: 24,

        clipboard_cmd: detect_clipboard(),

        msg: String::new(),
        msg_time: None,
    };

    // Get terminal size
    app.term_w = terminal::size().map(|(w, _)| w).unwrap_or(80);
    app.term_h = terminal::size().map(|(_, h)| h).unwrap_or(24);

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // Initial fetch
    fetch_data(&mut app);
    if !app.descendant_filter.is_empty() {
        build_desc_set(&mut app);
    }
    build_render_buffer(&mut app);
    if app.cursor < 0 {
        app.cursor = 0;
    }
    if app.cursor >= app.render_lines.len() as isize {
        app.cursor = app.render_lines.len().saturating_sub(1) as isize;
    }
    app.refresh = false;

    loop {
        // Handle refresh
        if app.refresh {
            fetch_data(&mut app);
            if !app.descendant_filter.is_empty() {
                build_desc_set(&mut app);
            }
            build_render_buffer(&mut app);
            app.dirty = true;
            app.refresh = false;
            if app.cursor < 0 {
                app.cursor = 0;
            }
            if app.cursor >= app.render_lines.len() as isize {
                app.cursor = app.render_lines.len().saturating_sub(1) as isize;
            }
        }

        // Update report autocomplete
        if app.mode == AppMode::ReportFilter && !app.report_email_input.is_empty() {
            let input_lower = app.report_email_input.to_lowercase();
            let output = Command::new("git")
                .args(["log", "--format=%ae"])
                .output();
            if let Ok(o) = output {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let mut matches: Vec<String> = stdout
                    .lines()
                    .map(|s| s.to_string())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .filter(|e| e.to_lowercase().contains(&input_lower))
                    .collect();
                matches.sort();
                matches.truncate(5);
                if app.report_ac_list != matches {
                    app.report_ac_list = matches;
                    app.dirty = true;
                }
            }
        }

        // Draw
        if app.dirty {
            let output = draw(&app);
            print!("{}", output);
            stdout.flush()?;
            app.dirty = false;
        }

        // Poll for events
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if !handle_key(&mut app, key) {
                        break;
                    }
                    app.dirty = true;
                }
                Event::Resize(w, h) => {
                    app.term_w = w;
                    app.term_h = h;
                    app.dirty = true;
                }
                _ => {}
            }
        }

        // Message timeout
        if let Some(ref msg_time) = app.msg_time {
            if !app.msg.is_empty() && msg_time.elapsed() > Duration::from_secs(2) {
                app.msg.clear();
                app.dirty = true;
            }
        }
    }

    // Cleanup
    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
