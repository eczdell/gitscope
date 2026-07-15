use std::path::Path;
use std::process::Command;

/// A registered git repository entry.
#[derive(Debug, Clone)]
pub struct RepoEntry {
    /// Absolute path to the repo
    pub path: String,
    /// Display name (directory name)
    pub name: String,
    /// Number of commits last loaded (or 0 if not yet loaded)
    pub commit_count: usize,
    /// Error message if the repo is invalid / unreachable
    pub error: Option<String>,
}

impl RepoEntry {
    /// Create a RepoEntry from a path. This validates that the path has a `.git` directory.
    pub fn from_path(path_str: &str) -> Self {
        let p = Path::new(path_str);
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.to_string());

        // Check if it's a valid git repo
        let git_dir = p.join(".git");
        let mut error = None;
        let mut commit_count = 0;

        if git_dir.exists() {
            // Try to count commits
            if let Ok(output) = Command::new("git")
                .args(["rev-list", "--count", "HEAD"])
                .current_dir(p)
                .output()
            {
                if output.status.success() {
                    let count_str = String::from_utf8_lossy(&output.stdout);
                    commit_count = count_str.trim().parse().unwrap_or(0);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error = Some(format!("git error: {}", stderr.trim()));
                }
            } else {
                error = Some("Failed to run git".to_string());
            }
        } else {
            error = Some("Not a git repository".to_string());
        }

        RepoEntry {
            path: path_str.to_string(),
            name,
            commit_count,
            error,
        }
    }

    /// Returns true if this entry is a valid, loadable repo.
    pub fn is_valid(&self) -> bool {
        self.error.is_none()
    }
}

/// Scan a directory for git repositories (subdirectories containing .git).
pub fn scan_directory(dir: &str) -> Vec<RepoEntry> {
    let mut repos = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return repos,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let git_dir = path.join(".git");
        if git_dir.exists() {
            let path_str = path.to_string_lossy().to_string();
            repos.push(RepoEntry::from_path(&path_str));
        }
    }

    repos.sort_by(|a, b| a.name.cmp(&b.name));
    repos
}

/// Switch the application's working directory to the given repo and refresh.
/// Returns an error message if the path is invalid.
pub fn switch_repo(repo_path: &str) -> Result<(), String> {
    let p = Path::new(repo_path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", repo_path));
    }
    if !p.join(".git").exists() {
        return Err(format!("Not a git repository: {}", repo_path));
    }

    // Change the process's current directory
    std::env::set_current_dir(p).map_err(|e| format!("Failed to change directory: {}", e))?;

    Ok(())
}

/// Get the current working directory.
pub fn get_cwd() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

/// Validate that a given path string is a readable directory with a .git subfolder.
pub fn validate_repo_path(path_str: &str) -> Result<(), String> {
    let p = Path::new(path_str);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path_str));
    }
    if !p.is_dir() {
        return Err(format!("Not a directory: {}", path_str));
    }
    if !p.join(".git").exists() {
        return Err(format!("No .git directory found in: {}", path_str));
    }
    Ok(())
}

/// Try to detect the current directory's repo and add it if not already in the list.
pub fn detect_current_repo() -> Option<RepoEntry> {
    let cwd = get_cwd();
    let p = Path::new(&cwd);
    if p.join(".git").exists() {
        Some(RepoEntry::from_path(&cwd))
    } else {
        None
    }
}

/// Read a directory, list all subdirectories (for autocomplete / scanning).
pub fn list_subdirs(dir: &str) -> Vec<String> {
    let mut dirs = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return dirs,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                dirs.push(name.to_string_lossy().to_string());
            }
        }
    }

    dirs.sort();
    dirs
}

/// Try to expand a relative path or home-directory path to an absolute path.
pub fn expand_path(path_str: &str) -> String {
    if path_str.starts_with("~/") {
        if let Some(home) = std::env::var("HOME").ok() {
            return format!("{}/{}", home.trim_end_matches('/'), &path_str[2..]);
        }
    }
    if path_str.starts_with('~') && path_str.len() == 1 {
        return std::env::var("HOME").unwrap_or_else(|_| path_str.to_string());
    }
    // If it's a relative path, resolve it
    let p = Path::new(path_str);
    if p.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join(p).to_string_lossy().to_string();
        }
    }
    path_str.to_string()
}

