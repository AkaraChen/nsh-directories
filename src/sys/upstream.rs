use std::path::PathBuf;

use crate::{BaseDirs, ProjectDirs, UserDirs};

pub(crate) fn base_dirs() -> Option<BaseDirs> {
    ::directories::BaseDirs::new().map(BaseDirs::from)
}

pub(crate) fn user_dirs() -> Option<UserDirs> {
    ::directories::UserDirs::new().map(UserDirs::from)
}

pub(crate) fn project_dirs_from_path(project_path: PathBuf) -> Option<ProjectDirs> {
    ::directories::ProjectDirs::from_path(project_path).map(From::from)
}

pub(crate) fn project_dirs_from(qualifier: &str, organization: &str, application: &str) -> Option<ProjectDirs> {
    ::directories::ProjectDirs::from(qualifier, organization, application).map(From::from)
}
