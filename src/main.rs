mod ansi;
mod app;
mod clipboard;
mod date;
mod git;
mod github;
mod input;
mod render;
mod repos;
mod ui;

use std::collections::HashSet;
use std::io;
use std::process::Command;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::AppState;
use crate::clipboard::detect_clipboard;
use crate::git::fetch_data;
use crate::input::handle_key;
use crate::render::build_render_buffer;
use crate::ui::ui;

// ═══════════════════════════════════════════════════════════════════════════
// CLI
// ═══════════════════════════════════════════════════════════════════════════

fn print_usage() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("{name} v{version}");
    eprintln!("Interactive Git tree visualizer (TUI) + GitHub CLI");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    {name} [OPTIONS]");
    eprintln!("    {name} issues [OPTIONS]");
    eprintln!("    {name} issue create --title <TITLE> [OPTIONS]");
    eprintln!();
    eprintln!("TUI OPTIONS:");
    eprintln!("    -h, --help       Print help information");
    eprintln!("    -V, --version    Print version information");
    eprintln!("    -n, --count N    Number of commits to show (default: 30)");
    eprintln!("    -a, --all        Show all branches at startup");
    eprintln!("    -b, --branch B   Filter by branch at startup");
    eprintln!();
    eprintln!("ISSUES OPTIONS:");
    eprintln!("    --owner O        GitHub owner (default: detected from git remote)");
    eprintln!("    --repo R         GitHub repo (default: detected from git remote)");
    eprintln!("    --state S        Filter by state: open, closed, all (default: open)");
    eprintln!();
    eprintln!("ISSUE CREATE OPTIONS:");
    eprintln!("    --title T        Issue title (required)");
    eprintln!("    --body B         Issue body text");
    eprintln!("    --owner O        GitHub owner (default: detected from git remote)");
    eprintln!("    --repo R         GitHub repo (default: detected from git remote)");
}

// ═══════════════════════════════════════════════════════════════════════════
// Subcommand Handlers
// ═══════════════════════════════════════════════════════════════════════════

fn resolve_owner_repo(
    cli_owner: Option<String>,
    cli_repo: Option<String>,
) -> (String, String) {
    if let (Some(o), Some(r)) = (cli_owner.as_ref(), cli_repo.as_ref()) {
        return (o.clone(), r.clone());
    }
    // Try to detect from git remote
    if let Some(repo) = github::detect_owner_repo() {
        return (cli_owner.unwrap_or(repo.owner), cli_repo.unwrap_or(repo.repo));
    }
    // Fallback
    (
        cli_owner.unwrap_or_else(|| "owner".to_string()),
        cli_repo.unwrap_or_else(|| "repo".to_string()),
    )
}

fn handle_issues_subcommand(args: &[String]) {
    let mut i = 0;
    let mut cli_owner: Option<String> = None;
    let mut cli_repo: Option<String> = None;
    let mut state = "open".to_string();

    while i < args.len() {
        match args[i].as_str() {
            "--owner" => {
                i += 1;
                cli_owner = Some(args[i].clone());
            }
            "--repo" => {
                i += 1;
                cli_repo = Some(args[i].clone());
            }
            "--state" => {
                i += 1;
                state = args[i].clone();
            }
            _ => {
                eprintln!("error: unknown option '{}'", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let (owner, repo) = resolve_owner_repo(cli_owner, cli_repo);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(github::list_issues(&owner, &repo, &state))
        .unwrap_or_else(|e| {
            eprintln!("{}Error:{} {}", ansi::LRE, ansi::RST, e);
            std::process::exit(1);
        });
}

fn handle_issue_create_subcommand(args: &[String]) {
    let mut i = 0;
    let mut cli_owner: Option<String> = None;
    let mut cli_repo: Option<String> = None;
    let mut title: Option<String> = None;
    let mut body = String::new();

    while i < args.len() {
        match args[i].as_str() {
            "--owner" => {
                i += 1;
                cli_owner = Some(args[i].clone());
            }
            "--repo" => {
                i += 1;
                cli_repo = Some(args[i].clone());
            }
            "--title" => {
                i += 1;
                title = Some(args[i].clone());
            }
            "--body" => {
                i += 1;
                body = args[i].clone();
            }
            _ => {
                eprintln!("error: unknown option '{}'", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let title = title.unwrap_or_else(|| {
        eprintln!("error: --title is required for 'issue create'");
        std::process::exit(1);
    });

    let (owner, repo) = resolve_owner_repo(cli_owner, cli_repo);

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(github::create_issue(&owner, &repo, &title, &body))
        .unwrap_or_else(|e| {
            eprintln!("{}Error:{} {}", ansi::LRE, ansi::RST, e);
            std::process::exit(1);
        });
}

// ═══════════════════════════════════════════════════════════════════════════
// Main
// ═══════════════════════════════════════════════════════════════════════════

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // ─── Subcommand routing ──────────────────────────────────────────
    if let Some(subcmd) = args.get(1) {
        let subcmd = subcmd.as_str();
        if subcmd == "issues" {
            handle_issues_subcommand(&args[2..]);
            return Ok(());
        }
        if subcmd == "issue" {
            let subcmd2 = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if subcmd2 == "create" {
                handle_issue_create_subcommand(&args[3..]);
                return Ok(());
            }
            if subcmd2 == "help" || subcmd2.is_empty() {
                print_usage();
                return Ok(());
            }
            eprintln!("error: unknown subcommand 'issue {}'", subcmd2);
            print_usage();
            std::process::exit(1);
        }
        // Not a subcommand — fall through to TUI arg parsing
    }

    // ─── TUI CLI argument parsing ────────────────────────────────────
    let mut cli_count: Option<usize> = None;
    let mut cli_all = false;
    let mut cli_branch: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            "-V" | "--version" => {
                eprintln!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-n" | "--count" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --count requires a value");
                    print_usage();
                    std::process::exit(1);
                }
                cli_count = Some(args[i].parse().unwrap_or_else(|_| {
                    eprintln!("error: invalid count '{}'", args[i]);
                    std::process::exit(1);
                }));
            }
            "-a" | "--all" => {
                cli_all = true;
            }
            "-b" | "--branch" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --branch requires a value");
                    print_usage();
                    std::process::exit(1);
                }
                cli_branch = Some(args[i].clone());
            }
            _ => {
                eprintln!("error: unknown option '{}'", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let mut app = AppState {
        // Apply CLI values
        count: cli_count.unwrap_or(30),
        show_all: cli_all,
        branch_filter: cli_branch.clone().unwrap_or_default(),
        mode: app::AppMode::Tree,
        dirty: true,
        refresh: true,

        commits: Vec::new(),
        index: std::collections::HashMap::new(),
        children: std::collections::HashMap::new(),
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

        show_meta: false,
        compact: false,

        filter_text: String::new(),
        filter_input: String::new(),
        descendant_filter: String::new(),
        descendant_set: std::collections::HashSet::new(),
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

        issues_scroll: 0,
        issues_lines: Vec::new(),
        issues_lines_full: Vec::new(),
        issues_state: "open".to_string(),
        issues_filter_input: String::new(),
        issues_filter_text: String::new(),
        issues_date_filter: String::new(),
        issues_label_filter: String::new(),
        issues_label_filter_input: String::new(),
        issues_project_status: std::collections::HashMap::new(),
        issues_project_status_filter: String::new(),
        issues_project_status_filter_input: String::new(),
        issues_cursor: 0,

        issue_create_title: String::new(),
        issue_create_body: String::new(),
        issue_create_focus: 0,
        issue_create_labels_input: String::new(),

        // Issue edit view
        issue_edit_title: String::new(),
        issue_edit_body: String::new(),
        issue_edit_number: 0,
        issue_edit_focus: 0,
        issue_edit_labels_input: String::new(),

        issue_detail_cursor: 0,
        issue_detail_scroll: 0,
        issue_detail_lines: Vec::new(),

        confirm_delete_issue: false,

        branch_idx: 0,
        authors: Vec::new(),

        term_w: 80,
        term_h: 24,

        clipboard_cmd: detect_clipboard(),

        msg: String::new(),
        msg_time: None,

        repos: Vec::new(),
        repos_cursor: 0,
        repos_scroll: 0,
        repos_add_input: String::new(),

        gists_scroll: 0,
        gists_lines: Vec::new(),
        gists_lines_full: Vec::new(),
        gists_cursor: 0,
        gists_filter_input: String::new(),
        gists_filter_text: String::new(),

        gist_content_lines: Vec::new(),
        gist_content_scroll: 0,

        available_labels: Vec::new(),
        label_ac_list: Vec::new(),
        label_ac_idx: 0,
    };

    // Auto-detect current repo and add it to the list
    if let Some(current) = repos::detect_current_repo() {
        app.repos.push(current);
    }

    // Get terminal size
    app.term_w = terminal::size().map(|(w, _)| w).unwrap_or(80);
    app.term_h = terminal::size().map(|(_, h)| h).unwrap_or(24);

    // Setup ratatui terminal
    terminal::enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Hide cursor
    terminal.hide_cursor()?;

    // Initial fetch
    fetch_data(&mut app);
    if !app.descendant_filter.is_empty() {
        crate::git::build_desc_set(&mut app);
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
                crate::git::build_desc_set(&mut app);
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
        if app.mode == app::AppMode::ReportFilter && !app.report_email_input.is_empty() {
            let input_lower = app.report_email_input.to_lowercase();
            let output = Command::new("git").args(["log", "--format=%ae"]).output();
            if let Ok(o) = output {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let mut matches: Vec<String> = stdout
                    .lines()
                    .map(|s| s.to_string())
                    .collect::<HashSet<_>>()
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

        // Draw using Ratatui
        if app.dirty {
            terminal.draw(|f| ui(f, &app))?;
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
    terminal.show_cursor()?;
    terminal::disable_raw_mode()?;
    Ok(())
}
