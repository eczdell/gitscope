use std::collections::HashMap;
use std::process::Command;

use git2::Repository;

use crate::app::{AppState, BRANCH_COLORS};
use crate::date::format_date_full;

// ═══════════════════════════════════════════════════════════════════════════
// Git Operations
// ═══════════════════════════════════════════════════════════════════════════

pub fn fetch_data(app: &mut AppState) {
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
            .unwrap_or(std::path::Path::new(".")),
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
            app.branches.push(crate::app::BranchInfo {
                name: name.clone(),
                color: BRANCH_COLORS[i % 8],
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
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        let date = format_date_full(commit.time());
        let subject = commit.summary().unwrap_or("").to_string();

        *author_count.entry(author.clone()).or_insert(0) += 1;

        // Build children map
        for p in &parents {
            app.children
                .entry(p.clone())
                .or_default()
                .push(hash.clone());
        }

        app.index.insert(hash.clone(), i);
        app.commits.push(crate::app::CommitInfo {
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

    // Lane assignment
    compute_lanes(app);
}

pub fn compute_lanes(app: &mut AppState) {
    app.lanes = vec![0; app.commits.len()];
    app.occupied = Vec::new();
    app.max_lane = 0;

    for i in 0..app.commits.len() {
        let hash = &app.commits[i].hash;

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

pub fn build_desc_set(app: &mut AppState) {
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
// Diff / Files / Report Openers
// ═══════════════════════════════════════════════════════════════════════════

pub fn open_diff(app: &mut AppState, ci: usize) {
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
    app.mode = crate::app::AppMode::Diff;
}

pub fn open_files(app: &mut AppState, ci: usize) {
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
    app.mode = crate::app::AppMode::Files;
}

pub fn open_report(app: &mut AppState) {
    let output = Command::new("git")
        .args([
            "log",
            "--format=%ae|%as|%s",
            "--name-only",
            "--diff-filter=AM",
            if app.show_all { "--all" } else { "HEAD" },
        ])
        .output();
    let raw = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => {
            app.report_lines = vec!["  Failed to load report".to_string()];
            app.report_scroll = 0;
            app.mode = crate::app::AppMode::Report;
            return;
        }
    };

    // Parse output: collect (email, filename, date)
    let mut entries: Vec<(String, String, String)> = Vec::new();
    let mut cur_email = String::new();
    let mut cur_date = String::new();
    for line in raw.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some(bar_pos) = line.find('|') {
            let email = &line[..bar_pos];
            let rest = &line[bar_pos + 1..];
            if let Some(bar2) = rest.find('|') {
                cur_email = email.to_string();
                cur_date = rest[..bar2].to_string();
                // Skip subject, just use date
            }
        } else if !cur_email.is_empty() {
            // This is a filename
            let fname = line.trim().to_string();
            if !fname.is_empty() {
                entries.push((cur_email.clone(), fname, cur_date.clone()));
            }
        }
    }

    // Apply email filter
    let filter = app.report_email_filter.clone();
    if !filter.is_empty() {
        entries.retain(|(email, _, _)| email.to_lowercase().contains(&filter));
        if entries.is_empty() {
            use crate::ansi;
            app.report_lines = vec![format!(
                "  {}No files found for: {}{}",
                ansi::DIM, app.report_email_filter, ansi::RST
            )];
            app.report_scroll = 0;
            app.mode = crate::app::AppMode::Report;
            return;
        }
    }

    // Sort by date
    if app.report_sort == "date" {
        entries.sort_by(|a, b| b.2.cmp(&a.2));
    }

    // Build lines
    use crate::ansi;
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
            app.report_lines.push(format!(
                "  CREATED BY: {}{}{}",
                ansi::CYN, email, ansi::RST
            ));
            app.report_lines
                .push("------------------------------------------------------------".to_string());
            app.report_lines.push(format!(
                "  {}FILE NAME{}{}{}DATE{}",
                ansi::BLD, ansi::RST,
                " ".repeat(38),
                ansi::BLD, ansi::RST
            ));
            app.report_lines.push(format!(
                "{}------------------------------------------------------------{}",
                ansi::DIM, ansi::RST
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
            ansi::LYL, bname, ansi::RST, " ".repeat(padding), ansi::DIM, date
        ));
    }

    app.report_lines.push(String::new());
    app.report_lines
        .push(format!("={}={:=<60}=", "", ""));

    app.report_scroll = 0;
    app.mode = crate::app::AppMode::Report;
}

