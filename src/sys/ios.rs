use std::path::PathBuf;

use crate::UserDirs;
#[cfg(target_os = "ios")]
use crate::{BaseDirs, ProjectDirs};

#[cfg(target_os = "ios")]
pub(crate) fn base_dirs() -> Option<BaseDirs> {
    ::directories::BaseDirs::new().map(BaseDirs::from)
}

/// On iOS, only `$HOME` and `$HOME/Documents` are standard container-visible user
/// directories; desktop-oriented folders (`Desktop`, `Music`, `Movies`, etc.) do
/// not exist in the iOS sandbox and are returned as `None`.
pub(crate) fn user_dirs_from_home(home_dir: PathBuf) -> UserDirs {
    let document_dir = Some(home_dir.join("Documents"));
    UserDirs {
        home_dir,
        audio_dir: None,
        desktop_dir: None,
        document_dir,
        download_dir: None,
        font_dir: None,
        picture_dir: None,
        public_dir: None,
        template_dir: None,
        video_dir: None,
    }
}

#[cfg(target_os = "ios")]
pub(crate) fn user_dirs() -> Option<UserDirs> {
    ::directories::BaseDirs::new().map(|base| user_dirs_from_home(base.home_dir().to_path_buf()))
}

#[cfg(target_os = "ios")]
pub(crate) fn project_dirs_from_path(project_path: PathBuf) -> Option<ProjectDirs> {
    ::directories::ProjectDirs::from_path(project_path).map(From::from)
}

#[cfg(target_os = "ios")]
pub(crate) fn project_dirs_from(qualifier: &str, organization: &str, application: &str) -> Option<ProjectDirs> {
    ::directories::ProjectDirs::from(qualifier, organization, application).map(From::from)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn ios_user_dirs_keeps_only_home_and_documents() {
        let home = PathBuf::from("/var/mobile/Containers/Data/Application/TEST-UUID");
        let dirs = user_dirs_from_home(home.clone());
        assert_eq!(dirs.home_dir(), home.as_path());
        assert_eq!(dirs.document_dir(), Some(home.join("Documents").as_path()));
        assert_eq!(dirs.audio_dir(), None);
        assert_eq!(dirs.desktop_dir(), None);
        assert_eq!(dirs.download_dir(), None);
        assert_eq!(dirs.font_dir(), None);
        assert_eq!(dirs.picture_dir(), None);
        assert_eq!(dirs.public_dir(), None);
        assert_eq!(dirs.template_dir(), None);
        assert_eq!(dirs.video_dir(), None);
    }
}
