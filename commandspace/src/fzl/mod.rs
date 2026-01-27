use std::path::PathBuf;

pub fn fzl_path() -> Option<PathBuf> {
    // return a cached (static) result
    // in app, the path should be fixed.
    // non_app usage is not a priority so we just use which.
    which::which("fzl").ok()
}
