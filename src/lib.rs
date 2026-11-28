//! # nsh-directories
//!
//! Platform-standard configuration, cache, data, runtime, and user directories
//! for [Native Shell](https://nsh.dev) apps in Rust and TSX, wrapping the
//! pinned [`directories`](https://crates.io/crates/directories) crate (`6.0.0`).
//!
//! - **Linux / macOS / Windows / BSD / Redox**: delegates directly to upstream
//!   `directories` (`BaseDirs`, `UserDirs`, `ProjectDirs`).
//! - **iOS**: delegates `BaseDirs` and `ProjectDirs` to upstream `mac.rs`
//!   (sandbox container `$HOME/Library/...`), and restricts `UserDirs` to
//!   `home_dir` and `document_dir` (`$HOME/Documents`).
//! - **Android**: queries `android.content.Context` (`getDataDir`, `getFilesDir`,
//!   `getCacheDir`, `getNoBackupFilesDir`) via `ndk-context` + `jni`;
//!   `UserDirs::new()` returns `None` due to Scoped Storage.
//! - **Web (`wasm32`)**: all factories return `None` and [`available()`] is
//!   `false`.
//!
//! With the default `nsh` feature enabled, this crate registers the
//! `nsh-directories` native module (`import { Directories } from "nsh-directories"`)
//! without modifying NSH's built-in `os.homedir()` or `process.dataDir`.
//! Disable default features (`default-features = false`) for pure Rust usage.

#![deny(missing_docs)]

use std::path::{Path, PathBuf};

pub mod provenance;
mod sys;

#[cfg(feature = "nsh")]
pub mod nsh;

/// Registers the `nsh-directories` module for Native Shell apps.
#[cfg(feature = "nsh")]
#[nsh_core::prelude::native_module("nsh-directories")]
fn native(registry: &mut nsh_core::prelude::Registry) {
    registry.module::<nsh::Directories>();
}

/// Upstream repository URL (`dirs/directories-rs`).
pub const UPSTREAM_REPO: &str = provenance::UPSTREAM_REPO;
/// Pinned upstream `directories` crate version.
pub const UPSTREAM_VERSION: &str = provenance::UPSTREAM_VERSION;
/// Pinned upstream `directories` commit SHA.
pub const UPSTREAM_COMMIT: &str = provenance::UPSTREAM_COMMIT;

/// Returns `true` when the target platform supports native standard
/// directories (`false` on `wasm32` and on HarmonyOS/OpenHarmony, whose
/// lookups are still `todo!()`; `true` otherwise).
pub const fn available() -> bool {
    !cfg!(any(target_arch = "wasm32", target_env = "ohos"))
}

/// `BaseDirs` provides paths of user-invisible standard directories, following
/// the conventions of the operating system the library is running on.
///
/// To compute the location of cache, config or data directories for individual
/// projects or applications, use [`ProjectDirs`] instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseDirs {
    pub(crate) home_dir: PathBuf,
    pub(crate) cache_dir: PathBuf,
    pub(crate) config_dir: PathBuf,
    pub(crate) config_local_dir: PathBuf,
    pub(crate) data_dir: PathBuf,
    pub(crate) data_local_dir: PathBuf,
    pub(crate) executable_dir: Option<PathBuf>,
    pub(crate) preference_dir: PathBuf,
    pub(crate) runtime_dir: Option<PathBuf>,
    pub(crate) state_dir: Option<PathBuf>,
}

/// `UserDirs` provides paths of user-facing standard directories, following the
/// conventions of the operating system the library is running on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDirs {
    pub(crate) home_dir: PathBuf,
    pub(crate) audio_dir: Option<PathBuf>,
    pub(crate) desktop_dir: Option<PathBuf>,
    pub(crate) document_dir: Option<PathBuf>,
    pub(crate) download_dir: Option<PathBuf>,
    pub(crate) font_dir: Option<PathBuf>,
    pub(crate) picture_dir: Option<PathBuf>,
    pub(crate) public_dir: Option<PathBuf>,
    pub(crate) template_dir: Option<PathBuf>,
    pub(crate) video_dir: Option<PathBuf>,
}

/// `ProjectDirs` computes the location of cache, config or data directories for
/// a specific application, derived from standard directories and the project's
/// qualifier, organization, and application name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDirs {
    pub(crate) project_path: PathBuf,
    pub(crate) cache_dir: PathBuf,
    pub(crate) config_dir: PathBuf,
    pub(crate) config_local_dir: PathBuf,
    pub(crate) data_dir: PathBuf,
    pub(crate) data_local_dir: PathBuf,
    pub(crate) preference_dir: PathBuf,
    pub(crate) runtime_dir: Option<PathBuf>,
    pub(crate) state_dir: Option<PathBuf>,
}

impl BaseDirs {
    /// Creates a `BaseDirs` struct holding a snapshot of standard base
    /// directory paths, or returns `None` if no valid home/container directory
    /// is available on the current platform.
    pub fn new() -> Option<BaseDirs> {
        sys::base_dirs()
    }

    /// Returns the path to the user's home directory (or app sandbox container root on iOS/Android).
    pub fn home_dir(&self) -> &Path {
        self.home_dir.as_path()
    }

    /// Returns the path to the user's cache directory.
    pub fn cache_dir(&self) -> &Path {
        self.cache_dir.as_path()
    }

    /// Returns the path to the user's config directory.
    pub fn config_dir(&self) -> &Path {
        self.config_dir.as_path()
    }

    /// Returns the path to the user's local config directory.
    pub fn config_local_dir(&self) -> &Path {
        self.config_local_dir.as_path()
    }

    /// Returns the path to the user's data directory.
    pub fn data_dir(&self) -> &Path {
        self.data_dir.as_path()
    }

    /// Returns the path to the user's local data directory.
    pub fn data_local_dir(&self) -> &Path {
        self.data_local_dir.as_path()
    }

    /// Returns the path to the user's executable directory, if defined on this platform.
    pub fn executable_dir(&self) -> Option<&Path> {
        self.executable_dir.as_deref()
    }

    /// Returns the path to the user's preference directory.
    pub fn preference_dir(&self) -> &Path {
        self.preference_dir.as_path()
    }

    /// Returns the path to the user's runtime directory, if defined on this platform.
    pub fn runtime_dir(&self) -> Option<&Path> {
        self.runtime_dir.as_deref()
    }

    /// Returns the path to the user's state directory, if defined on this platform.
    pub fn state_dir(&self) -> Option<&Path> {
        self.state_dir.as_deref()
    }
}

impl UserDirs {
    /// Creates a `UserDirs` struct holding paths to user-facing directories, or
    /// returns `None` when unavailable (e.g., on Android, `wasm32`, or when no
    /// valid home directory can be retrieved).
    pub fn new() -> Option<UserDirs> {
        sys::user_dirs()
    }

    /// Returns the path to the user's home directory.
    pub fn home_dir(&self) -> &Path {
        self.home_dir.as_path()
    }

    /// Returns the path to the user's audio directory, if available.
    pub fn audio_dir(&self) -> Option<&Path> {
        self.audio_dir.as_deref()
    }

    /// Returns the path to the user's desktop directory, if available.
    pub fn desktop_dir(&self) -> Option<&Path> {
        self.desktop_dir.as_deref()
    }

    /// Returns the path to the user's document directory, if available.
    pub fn document_dir(&self) -> Option<&Path> {
        self.document_dir.as_deref()
    }

    /// Returns the path to the user's download directory, if available.
    pub fn download_dir(&self) -> Option<&Path> {
        self.download_dir.as_deref()
    }

    /// Returns the path to the user's font directory, if available.
    pub fn font_dir(&self) -> Option<&Path> {
        self.font_dir.as_deref()
    }

    /// Returns the path to the user's picture directory, if available.
    pub fn picture_dir(&self) -> Option<&Path> {
        self.picture_dir.as_deref()
    }

    /// Returns the path to the user's public directory, if available.
    pub fn public_dir(&self) -> Option<&Path> {
        self.public_dir.as_deref()
    }

    /// Returns the path to the user's template directory, if available.
    pub fn template_dir(&self) -> Option<&Path> {
        self.template_dir.as_deref()
    }

    /// Returns the path to the user's video directory, if available.
    pub fn video_dir(&self) -> Option<&Path> {
        self.video_dir.as_deref()
    }
}

impl ProjectDirs {
    /// Creates a `ProjectDirs` struct directly from a `PathBuf` value used
    /// verbatim as the project subpath. Prefer [`ProjectDirs::from`].
    pub fn from_path(project_path: PathBuf) -> Option<ProjectDirs> {
        sys::project_dirs_from_path(project_path)
    }

    /// Creates a `ProjectDirs` struct from qualifier, organization, and
    /// application name following platform conventions.
    pub fn from(qualifier: &str, organization: &str, application: &str) -> Option<ProjectDirs> {
        sys::project_dirs_from(qualifier, organization, application)
    }

    /// Returns the project path fragment used to compute project directories.
    pub fn project_path(&self) -> &Path {
        self.project_path.as_path()
    }

    /// Returns the path to the project's cache directory.
    pub fn cache_dir(&self) -> &Path {
        self.cache_dir.as_path()
    }

    /// Returns the path to the project's config directory.
    pub fn config_dir(&self) -> &Path {
        self.config_dir.as_path()
    }

    /// Returns the path to the project's local config directory.
    pub fn config_local_dir(&self) -> &Path {
        self.config_local_dir.as_path()
    }

    /// Returns the path to the project's data directory.
    pub fn data_dir(&self) -> &Path {
        self.data_dir.as_path()
    }

    /// Returns the path to the project's local data directory.
    pub fn data_local_dir(&self) -> &Path {
        self.data_local_dir.as_path()
    }

    /// Returns the path to the project's preference directory.
    pub fn preference_dir(&self) -> &Path {
        self.preference_dir.as_path()
    }

    /// Returns the path to the project's runtime directory, if defined on this platform.
    pub fn runtime_dir(&self) -> Option<&Path> {
        self.runtime_dir.as_deref()
    }

    /// Returns the path to the project's state directory, if defined on this platform.
    pub fn state_dir(&self) -> Option<&Path> {
        self.state_dir.as_deref()
    }
}

#[cfg(not(target_os = "android"))]
impl From<::directories::BaseDirs> for BaseDirs {
    fn from(dirs: ::directories::BaseDirs) -> Self {
        Self {
            home_dir: dirs.home_dir().to_path_buf(),
            cache_dir: dirs.cache_dir().to_path_buf(),
            config_dir: dirs.config_dir().to_path_buf(),
            config_local_dir: dirs.config_local_dir().to_path_buf(),
            data_dir: dirs.data_dir().to_path_buf(),
            data_local_dir: dirs.data_local_dir().to_path_buf(),
            executable_dir: dirs.executable_dir().map(Path::to_path_buf),
            preference_dir: dirs.preference_dir().to_path_buf(),
            runtime_dir: dirs.runtime_dir().map(Path::to_path_buf),
            state_dir: dirs.state_dir().map(Path::to_path_buf),
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl From<::directories::UserDirs> for UserDirs {
    fn from(dirs: ::directories::UserDirs) -> Self {
        Self {
            home_dir: dirs.home_dir().to_path_buf(),
            audio_dir: dirs.audio_dir().map(Path::to_path_buf),
            desktop_dir: dirs.desktop_dir().map(Path::to_path_buf),
            document_dir: dirs.document_dir().map(Path::to_path_buf),
            download_dir: dirs.download_dir().map(Path::to_path_buf),
            font_dir: dirs.font_dir().map(Path::to_path_buf),
            picture_dir: dirs.picture_dir().map(Path::to_path_buf),
            public_dir: dirs.public_dir().map(Path::to_path_buf),
            template_dir: dirs.template_dir().map(Path::to_path_buf),
            video_dir: dirs.video_dir().map(Path::to_path_buf),
        }
    }
}

#[cfg(not(target_os = "android"))]
impl From<::directories::ProjectDirs> for ProjectDirs {
    fn from(dirs: ::directories::ProjectDirs) -> Self {
        Self {
            project_path: dirs.project_path().to_path_buf(),
            cache_dir: dirs.cache_dir().to_path_buf(),
            config_dir: dirs.config_dir().to_path_buf(),
            config_local_dir: dirs.config_local_dir().to_path_buf(),
            data_dir: dirs.data_dir().to_path_buf(),
            data_local_dir: dirs.data_local_dir().to_path_buf(),
            preference_dir: dirs.preference_dir().to_path_buf(),
            runtime_dir: dirs.runtime_dir().map(Path::to_path_buf),
            state_dir: dirs.state_dir().map(Path::to_path_buf),
        }
    }
}
