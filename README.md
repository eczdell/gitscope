# gitscope

Interactive Git tree visualizer (TUI) with GitHub CLI integration.

## Features

- **Interactive commit tree** — Visualize your Git commit history with branch topology, lane assignments, and merge connectors
- **Diff viewer** — View commit diffs directly in the terminal
- **File viewer** — See files changed in each commit
- **Author report** — Files grouped by author with contribution bars
- **Filtering** — Filter commits by text, date range, branch, or commit descendants
- **Clipboard support** — Copy commit hashes to clipboard (xclip, xsel, wl-copy, pbcopy)
- **GitHub integration** — List, view, create, edit, and close issues from the TUI

## Installation

```bash
# Build from source
cargo build --release

# The binary will be at target/release/gitscope
```

### Prerequisites

- Rust 2021 edition
- libgit2 (for the `git2` crate)
- `gh` CLI (GitHub CLI) for issue operations (create, edit, close)

## Usage

### TUI mode

```bash
# Show commit tree (default 30 commits)
gitscope

# Show more commits
gitscope -n 100

# Show all branches
gitscope -a

# Filter by branch at startup
gitscope -b main

# Show help
gitscope --help
```

### TUI key bindings

| Key | Action |
|-----|--------|
| `j`/`k` | Move cursor up/down |
| `J`/`K` | Move 5 lines |
| `g`/`G` | Top / Bottom |
| `PgUp`/`PgDn` | Page up/down |
| `Enter`/`o` | Zoom into commit descendants |
| `Esc` | Clear zoom filter |
| `d` | View commit diff |
| `f` | View changed files |
| `y` | Copy commit hash |
| `a` | Toggle all branches |
| `m` | Toggle commit metadata |
| `c` | Toggle compact mode |
| `D` | Toggle last-7-day date filter |
| `R` | Report: files by author |
| `/` | Filter commits (regex) |
| `b` | Focus branch picker (sidebar) |
| `0` | Reset all filters |
| `?` | Toggle help |
| `r` | Refresh data |
| `q` / `Ctrl-C` | Quit |

### GitHub Issues (CLI)

```bash
# List issues
gitscope issues
gitscope issues --state closed
gitscope issues --owner myorg --repo myrepo

# Create an issue
gitscope issue create --title "Fix the thing" --body "Details here"
```

### GitHub Issues (TUI)

From the commit tree view:
- Press `i` to open the issues list
- `j`/`k` to navigate
- `Enter` to view issue details
- `x` to close an issue
- `e` to edit an issue (title & body)
- `c` to create a new issue
- `s` to toggle state filter (open/closed/all)
- `q`/`Esc` to go back

## Authentication

For GitHub operations, you need one of:
- `GITHUB_TOKEN` environment variable
- `GH_TOKEN` environment variable
- Run `gh auth login` (GitHub CLI)

## License

MIT

