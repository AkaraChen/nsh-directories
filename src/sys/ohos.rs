//! HarmonyOS / OpenHarmony (`target_env = "ohos"`) — **spike placeholder**.
//!
//! OHOS targets report `target_os = "linux"`, so without this module they would fall
//! through to upstream `lin.rs` and read `$HOME` / XDG variables that do not describe
//! an app sandbox. Until a real implementation queries the ability context
//! (`filesDir`, `cacheDir`, `tempDir`, …), every lookup is `todo!()`: the API exists
//! and compiles, but calling it panics. `crate::available()` is `false` on OHOS so
//! callers can feature-detect before calling.

use std::path::PathBuf;

use crate::{BaseDirs, ProjectDirs, UserDirs};

pub(crate) fn base_dirs() -> Option<BaseDirs> {
    todo!("nsh-directories: BaseDirs is not implemented on HarmonyOS/OpenHarmony yet")
}

pub(crate) fn user_dirs() -> Option<UserDirs> {
    todo!("nsh-directories: UserDirs is not implemented on HarmonyOS/OpenHarmony yet")
}

pub(crate) fn project_dirs_from_path(_project_path: PathBuf) -> Option<ProjectDirs> {
    todo!("nsh-directories: ProjectDirs is not implemented on HarmonyOS/OpenHarmony yet")
}

pub(crate) fn project_dirs_from(_qualifier: &str, _organization: &str, _application: &str) -> Option<ProjectDirs> {
    todo!("nsh-directories: ProjectDirs is not implemented on HarmonyOS/OpenHarmony yet")
}
