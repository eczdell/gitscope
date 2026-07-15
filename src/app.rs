use std::collections::{HashMap, HashSet};
use std::time::Instant;

use ratatui::style::Color;

// ═══════════════════════════════════════════════════════════════════════════
// Style Constants (Ratatui colors)
// ═══════════════════════════════════════════════════════════════════════════
pub const S_GRN: Color = Color::Green;
pub const S_CYN: Color = Color::Cyan;
pub const S_LRE: Color = Color::LightRed;
pub const S_LGR: Color = Color::LightGreen;
pub const S_LYL: Color = Color::LightYellow;
pub const S_LBL: Color = Color::LightBlue;
pub const S_LMA: Color = Color::LightMagenta;
pub const S_LCY: Color = Color::LightCyan;

pub const BRANCH_COLORS: [Color; 8] = [S_GRN, S_LCY, S_LYL, S_LMA, S_LRE, S_LBL, S_LGR, S_CYN];

// ═══════════════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Tree,
    TreeFilter,
    Diff,
    Files,
    Issues,
    IssuesFilter,
    IssueCreate,
    IssueDetail,
    IssueEdit,
    Report,
    ReportFilter,
    SidebarFocus,
    Help,
}

pub struct CommitInfo {
    pub hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub date: String,
    pub subject: String,
}

pub struct BranchInfo {
    pub name: String,
    pub color: Color,
    pub head_oid: String,
}

pub struct RenderLine {
    pub content: String,
    pub commit_idx: Option<usize>,
}

pub struct AppState {
    pub mode: AppMode,
    pub dirty: bool,
    pub refresh: bool,

    // Git data
    pub commits: Vec<CommitInfo>,
    pub index: HashMap<String, usize>,
    pub children: HashMap<String, Vec<String>>,
    pub branches: Vec<BranchInfo>,
    pub repo_name: String,
    pub current_branch: String,
    pub head_hash: String,
    pub total: usize,

    // Layout
    pub lanes: Vec<usize>,
    pub occupied: Vec<Option<String>>,
    pub max_lane: usize,
    pub nlanes: usize,

    // Render buffer
    pub render_lines: Vec<RenderLine>,

    // Cursor & scroll
    pub cursor: isize,
    pub scroll: usize,

    // Tree display
    pub count: usize,
    pub show_all: bool,
    pub show_meta: bool,
    pub compact: bool,

    // Filters
    pub filter_text: String,
    pub filter_input: String,
    pub branch_filter: String,
    pub descendant_filter: String,
    pub descendant_set: HashSet<String>,
    pub date_from: String,
    pub date_to: String,

    // Diff view
    pub diff_scroll: usize,
    pub diff_lines: Vec<String>,

    // Files view
    pub files_scroll: usize,
    pub files_lines: Vec<String>,

    // Issues view
    pub issues_scroll: usize,
    pub issues_lines: Vec<String>,
    pub issues_lines_full: Vec<String>,
    pub issues_state: String,
    pub issues_filter_input: String,
    pub issues_filter_text: String,

    pub issue_create_title: String,
    pub issue_create_body: String,
    pub issue_create_focus_title: bool,

    // Issue edit view
    pub issue_edit_title: String,
    pub issue_edit_body: String,
    pub issue_edit_number: u64,
    pub issue_edit_focus_title: bool,

    // Issue detail view
    pub issue_detail_cursor: usize,
    pub issue_detail_scroll: usize,
    pub issue_detail_lines: Vec<String>,

    // Issues list cursor
    pub issues_cursor: usize,

    // Issue deletion confirmation
    pub confirm_delete_issue: bool,

    // Report view
    pub report_scroll: usize,
    pub report_lines: Vec<String>,
    pub report_email_filter: String,
    pub report_email_input: String,
    pub report_sort: String,
    pub report_ac_idx: usize,
    pub report_ac_list: Vec<String>,

    // Sidebar
    pub branch_idx: usize,
    pub authors: Vec<(String, usize)>,

    // Terminal
    pub term_w: u16,
    pub term_h: u16,

    // Clipboard
    pub clipboard_cmd: Option<String>,

    // Message
    pub msg: String,
    pub msg_time: Option<Instant>,
}

