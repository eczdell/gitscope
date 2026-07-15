// ═══════════════════════════════════════════════════════════════════════════
// ANSI Escape Code Constants
// ═══════════════════════════════════════════════════════════════════════════
pub const RST: &str = "\x1b[0m";
pub const BLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
#[allow(dead_code)]
pub const UND: &str = "\x1b[4m";
#[allow(dead_code)]
pub const RED: &str = "\x1b[31m";
pub const GRN: &str = "\x1b[32m";
pub const YEL: &str = "\x1b[33m";
#[allow(dead_code)]
pub const BLU: &str = "\x1b[34m";
#[allow(dead_code)]
pub const MAG: &str = "\x1b[35m";
pub const CYN: &str = "\x1b[36m";
pub const WHT: &str = "\x1b[37m";
pub const GRY: &str = "\x1b[90m";
pub const LRE: &str = "\x1b[91m";
pub const LGR: &str = "\x1b[92m";
pub const LYL: &str = "\x1b[93m";
pub const LBL: &str = "\x1b[94m";
pub const LMA: &str = "\x1b[95m";
#[allow(dead_code)]
pub const LCY: &str = "\x1b[96m";

// ═══════════════════════════════════════════════════════════════════════════
// ANSI Helper Functions
// ═══════════════════════════════════════════════════════════════════════════

/// Returns the visible length of a string that may contain ANSI escape codes.
pub fn vis_len(s: &str) -> usize {
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

/// Remove all ANSI escape codes from a string.
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some('[') = chars.next() {
                for ec in &mut chars {
                    if ec.is_ascii_alphabetic() {
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

/// Truncates a string that may contain ANSI escape codes to a maximum visible length.
pub fn truncate_vis(s: &str, max_len: usize) -> String {
    let mut len = 0;
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                result.push('\x1b');
                result.push(chars.next().unwrap());
                for ec in &mut chars {
                    result.push(ec);
                    if ec.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            if len >= max_len {
                break;
            }
            result.push(c);
            len += 1;
        }
    }
    result
}

