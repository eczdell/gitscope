use std::io::Write;
use std::process::{Command, Stdio};

/// Detect available clipboard tool on the system.
pub fn detect_clipboard() -> Option<String> {
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

/// Copy text to clipboard using the given command.
pub fn copy_to_clipboard(cmd: &str, text: &str) -> bool {
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

