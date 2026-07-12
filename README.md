# gitscope

A fast, interactive terminal UI for exploring git history as a visual tree. Built in pure bash with zero dependencies.

## Install

```bash
git clone https://github.com/eczdell/gitscope.git
chmod +x gitscope/gitscope
# Optional: add to PATH
ln -s ~/gitscope/gitscope ~/.local/bin/gitscope
```

## Usage

```bash
cd /path/to/your/repo
gitscope
```

## Key Bindings

### Navigation
| Key | Action |
|-----|--------|
| `j` / `k` | Move cursor down / up |
| `J` / `K` | Move 5 lines down / up |
| `g` / `G` | Jump to top / bottom |
| `PgUp` / `PgDn` | Page up / down |

### Actions
| Key | Action |
|-----|--------|
| `Enter` / `o` | Zoom into commit descendants |
| `d` | View commit diff |
| `f` | View changed files |
| `y` | Copy commit hash to clipboard |
| `/` | Filter commits (supports regex) |
| `D` | Toggle last-7-day date filter |
| `R` | Refresh data |

### Display
| Key | Action |
|-----|--------|
| `a` | Toggle all branches |
| `m` | Toggle commit metadata |
| `c` | Toggle compact mode |
| `+` / `-` | ±10 commits |

### Sidebar
| Key | Action |
|-----|--------|
| `b` | Focus branch picker |
| `0` | Reset all filters |
| `Esc` | Clear zoom/date filter |

### General
| Key | Action |
|-----|--------|
| `?` | Toggle help screen |
| `q` | Quit |

## Features

- **Visual tree graph** with merge/branch indicators
- **Branch sidebar** with commit counts
- **Author sidebar** with contribution bars
- **Commit search** with regex support
- **Date filtering** (last 7 days)
- **Diff view** with syntax highlighting
- **Changed files** list with stats
- **Clipboard integration** (xclip/xsel/wl-copy/pbcopy)
- **Descendant zoom** to explore branch history

## Requirements

- bash 4+
- git
- Optional: xclip, xsel, wl-copy, or pbcopy for clipboard support
