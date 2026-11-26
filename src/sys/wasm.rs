use std::path::PathBuf;

use crate::{BaseDirs, ProjectDirs, UserDirs};

pub(crate) fn base_dirs() -> Option<BaseDirs> {
    None
}

pub(crate) fn user_dirs() -> Option<UserDirs> {
    None
}

pub(crate) fn project_dirs_from_path(_project_path: PathBuf) -> Option<ProjectDirs> {
    None
}

pub(crate) fn project_dirs_from(_qualifier: &str, _organization: &str, _application: &str) -> Option<ProjectDirs> {
    None
}
