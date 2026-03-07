use std::env;
use std::path::{Path, PathBuf};

fn executable_dir() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(exe_dir) = executable_dir() {
        roots.push(exe_dir);
    }

    if let Ok(cwd) = env::current_dir() {
        roots.push(cwd.clone());
        roots.push(cwd.join("backend"));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    roots.push(manifest_dir.clone());
    roots.push(manifest_dir.join(".."));

    roots
}

pub fn resolve_runtime_path(relative: impl AsRef<Path>) -> PathBuf {
    let rel = relative.as_ref();

    for root in candidate_roots() {
        let candidate = root.join(rel);
        if candidate.exists() {
            return candidate;
        }
    }

    if let Some(exe_dir) = executable_dir() {
        return exe_dir.join(rel);
    }

    PathBuf::from(rel)
}
