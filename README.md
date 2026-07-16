# gitscope

Interactive Git tree visualizer (TUI) with GitHub CLI integration.

## Features

- **Interactive commit tree** — Visualize your Git commit history with branch topology, lane assignments, and merge connectors
- **Diff viewer** — View commit diffs directly in the terminal
- **File viewer** — See files changed in each commit
- **Author report** — Files grouped by author with contribution bars
- **Filtering** — Filter commits by text, date range, branch, or commit descendants
- **Clipboard support** — Copy commit hashes and gist content (`y`) to clipboard
  - Supported tools: `xclip`, `xsel` (X11), `wl-copy` (Wayland), `pbcopy` (macOS), `termux-clipboard-set` (Android Termux)
- **GitHub integration** — List, view, create, edit, and close issues from the TUI
- **Gists view** — Browse and copy GitHub gists from the TUI

## Installation

### Linux / macOS — One-liner install

```bash
curl -fsSL https://raw.githubusercontent.com/eczdell/gitscope/main/install.sh | bash
```

This downloads the latest binary for your platform and installs it to `/usr/local/bin/gitscope`.

### Windows

#### Option 1: Install via Cargo

```powershell
# Install Rust first: https://rustup.rs
cargo install gitscope
```

#### Option 2: Build from source

```powershell
git clone https://github.com/eczdell/gitscope.git
cd gitscope
cargo build --release
# The binary will be at target/release/gitscope.exe
# Add it to your PATH manually
```

#### Option 3: Scoop (if available)

```powershell
scoop bucket add eczdell https://github.com/eczdell/gitscope.git
scoop install gitscope
```

### Build from source (any platform)

```bash
# Requires Rust: https://rustup.rs
cargo install gitscope

# Or clone and build locally
git clone https://github.com/eczdell/gitscope.git
cd gitscope
cargo build --release

# The binary will be at target/release/gitscope
# Move it to a directory in your PATH, e.g.:
sudo cp target/release/gitscope /usr/local/bin/
```

### Prerequisites

- Rust 2021 edition
- libgit2 (for the `git2` crate)
- `gh` CLI (GitHub CLI) for issue operations (create, edit, close)
- One of the following for clipboard support (optional):
  - `xclip` or `xsel` (X11/Linux)
  - `wl-copy` (Wayland/Linux)
  - `pbcopy` (macOS)
  - `termux-clipboard-set` (Termux/Android)

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
| `'` (single quote) | Open gists view |
| `P` | Open repos management view |
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
- `/` to search/filter issues
- `q`/`Esc` to go back

### GitHub Gists (TUI)

From the commit tree view:
- Press `'` (single quote) to open the gists list
- `j`/`k` to navigate
- `y` to copy gist file content to clipboard (requires clipboard tool, see prerequisites)
- `/` to search/filter gists
- `g`/`G` to go to top/bottom
- `q`/`Esc` to go back

## Authentication

For GitHub operations, you need one of:
- `GITHUB_TOKEN` environment variable
- `GH_TOKEN` environment variable
- Run `gh auth login` (GitHub CLI)

## License

MIT

