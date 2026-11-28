//! Native Shell bindings (feature `nsh`).
//!
//! Exposes `import { Directories } from "nsh-directories"` to TSX as a static
//! module returning plain JS path-snapshot objects (`#[derive(JsObject)]`).
//!
//! # Capability model
//!
//! Only `Directories.homeDir()`, the `homeDir` property inside
//! `Directories.baseDirs()` (redacted to `null` when ungranted), and
//! `Directories.userDirs()` require `capabilities.homeDir` (`ERR_ACCESS_DENIED`
//! otherwise). Application-scoped directories (`cacheDir`, `configDir`,
//! `configLocalDir`, `dataDir`, `dataLocalDir`, `preferenceDir`, `runtimeDir`,
//! `stateDir`, `executableDir`, and `projectDirs` / `projectDirsFromPath`) do
//! not require `capabilities.homeDir`, matching `process.dataDir`. Returned
//! paths are metadata only and do not grant `capabilities.fs` access.

use std::path::{Path, PathBuf};

use nsh_core::prelude::*;

/// Snapshot of user-invisible standard base directory paths for the current platform.
#[derive(Debug, Clone, PartialEq, Eq, JsObject)]
pub struct BaseDirs {
    /// Path to the user's home directory, or `null` if `capabilities.homeDir` is not granted.
    pub home_dir: Option<String>,
    /// Path to the user's cache directory.
    pub cache_dir: String,
    /// Path to the user's config directory.
    pub config_dir: String,
    /// Path to the user's local config directory.
    pub config_local_dir: String,
    /// Path to the user's data directory.
    pub data_dir: String,
    /// Path to the user's local data directory.
    pub data_local_dir: String,
    /// Path to the user's executable directory, or `null` if undefined on this platform.
    pub executable_dir: Option<String>,
    /// Path to the user's preference directory.
    pub preference_dir: String,
    /// Path to the user's runtime directory, or `null` if undefined on this platform.
    pub runtime_dir: Option<String>,
    /// Path to the user's state directory, or `null` if undefined on this platform.
    pub state_dir: Option<String>,
}

/// Snapshot of user-facing standard directory paths for the current platform.
#[derive(Debug, Clone, PartialEq, Eq, JsObject)]
pub struct UserDirs {
    /// Path to the user's home directory.
    pub home_dir: String,
    /// Path to the user's audio directory, or `null` if unavailable.
    pub audio_dir: Option<String>,
    /// Path to the user's desktop directory, or `null` if unavailable.
    pub desktop_dir: Option<String>,
    /// Path to the user's document directory, or `null` if unavailable.
    pub document_dir: Option<String>,
    /// Path to the user's download directory, or `null` if unavailable.
    pub download_dir: Option<String>,
    /// Path to the user's font directory, or `null` if unavailable.
    pub font_dir: Option<String>,
    /// Path to the user's picture directory, or `null` if unavailable.
    pub picture_dir: Option<String>,
    /// Path to the user's public directory, or `null` if unavailable.
    pub public_dir: Option<String>,
    /// Path to the user's template directory, or `null` if unavailable.
    pub template_dir: Option<String>,
    /// Path to the user's video directory, or `null` if unavailable.
    pub video_dir: Option<String>,
}

/// Snapshot of project-specific standard directory paths for an application.
#[derive(Debug, Clone, PartialEq, Eq, JsObject)]
pub struct ProjectDirs {
    /// Project path fragment derived from the qualifier, organization, and application.
    pub project_path: String,
    /// Path to the project's cache directory.
    pub cache_dir: String,
    /// Path to the project's config directory.
    pub config_dir: String,
    /// Path to the project's local config directory.
    pub config_local_dir: String,
    /// Path to the project's data directory.
    pub data_dir: String,
    /// Path to the project's local data directory.
    pub data_local_dir: String,
    /// Path to the project's preference directory.
    pub preference_dir: String,
    /// Path to the project's runtime directory, or `null` if undefined on this platform.
    pub runtime_dir: Option<String>,
    /// Path to the project's state directory, or `null` if undefined on this platform.
    pub state_dir: Option<String>,
}

/// Crate and pinned upstream provenance information.
#[derive(Debug, Clone, PartialEq, Eq, JsObject)]
pub struct DirectoriesInfo {
    /// This `nsh-directories` crate version.
    pub version: String,
    /// Pinned upstream `directories` crate version.
    pub upstream_version: String,
    /// Pinned upstream `directories-rs` repository commit SHA.
    pub upstream_commit: String,
    /// Whether native standard directory lookup is supported on this platform.
    pub available: bool,
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn opt_path_to_string(path: Option<&Path>) -> Option<String> {
    path.map(path_to_string)
}

fn home_dir_denied(fn_name: &str) -> JsError {
    JsError::access_denied(
        format!("Directories.{fn_name}() is not granted: add `\"homeDir\": true` to capabilities in nsh.json"),
        "homeDir",
        "",
    )
}

impl BaseDirs {
    fn from_rust(dirs: crate::BaseDirs, include_home_dir: bool) -> Self {
        Self {
            home_dir: include_home_dir.then(|| path_to_string(dirs.home_dir())),
            cache_dir: path_to_string(dirs.cache_dir()),
            config_dir: path_to_string(dirs.config_dir()),
            config_local_dir: path_to_string(dirs.config_local_dir()),
            data_dir: path_to_string(dirs.data_dir()),
            data_local_dir: path_to_string(dirs.data_local_dir()),
            executable_dir: opt_path_to_string(dirs.executable_dir()),
            preference_dir: path_to_string(dirs.preference_dir()),
            runtime_dir: opt_path_to_string(dirs.runtime_dir()),
            state_dir: opt_path_to_string(dirs.state_dir()),
        }
    }
}

impl From<crate::UserDirs> for UserDirs {
    fn from(dirs: crate::UserDirs) -> Self {
        Self {
            home_dir: path_to_string(dirs.home_dir()),
            audio_dir: opt_path_to_string(dirs.audio_dir()),
            desktop_dir: opt_path_to_string(dirs.desktop_dir()),
            document_dir: opt_path_to_string(dirs.document_dir()),
            download_dir: opt_path_to_string(dirs.download_dir()),
            font_dir: opt_path_to_string(dirs.font_dir()),
            picture_dir: opt_path_to_string(dirs.picture_dir()),
            public_dir: opt_path_to_string(dirs.public_dir()),
            template_dir: opt_path_to_string(dirs.template_dir()),
            video_dir: opt_path_to_string(dirs.video_dir()),
        }
    }
}

impl From<crate::ProjectDirs> for ProjectDirs {
    fn from(dirs: crate::ProjectDirs) -> Self {
        Self {
            project_path: path_to_string(dirs.project_path()),
            cache_dir: path_to_string(dirs.cache_dir()),
            config_dir: path_to_string(dirs.config_dir()),
            config_local_dir: path_to_string(dirs.config_local_dir()),
            data_dir: path_to_string(dirs.data_dir()),
            data_local_dir: path_to_string(dirs.data_local_dir()),
            preference_dir: path_to_string(dirs.preference_dir()),
            runtime_dir: opt_path_to_string(dirs.runtime_dir()),
            state_dir: opt_path_to_string(dirs.state_dir()),
        }
    }
}

/// Platform-standard config, cache, data, and user directory paths.
#[module]
pub struct Directories;

#[module]
impl Directories {
    /// Returns `true` when native standard directory lookup is supported on this platform (`false` on web/wasm and HarmonyOS).
    pub fn available() -> bool {
        crate::available()
    }

    /// Returns version and pinned upstream provenance metadata for `nsh-directories`.
    pub fn info() -> DirectoriesInfo {
        DirectoriesInfo {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            upstream_version: crate::UPSTREAM_VERSION.to_owned(),
            upstream_commit: crate::UPSTREAM_COMMIT.to_owned(),
            available: crate::available(),
        }
    }

    /// Returns the user's home directory path (requires `capabilities.homeDir`), or `null` when unavailable.
    pub fn home_dir(sys: System) -> Result<Option<String>, JsError> {
        if !sys.access.allows_home_dir() {
            return Err(home_dir_denied("homeDir"));
        }
        Ok(crate::BaseDirs::new().map(|dirs| path_to_string(dirs.home_dir())))
    }

    /// Returns standard base directories, or `null` when unavailable (`homeDir` is `null` unless `capabilities.homeDir` is granted).
    pub fn base_dirs(sys: System) -> Option<BaseDirs> {
        let include_home_dir = sys.access.allows_home_dir();
        crate::BaseDirs::new().map(|dirs| BaseDirs::from_rust(dirs, include_home_dir))
    }

    /// Returns user-facing directories (requires `capabilities.homeDir`), or `null` when unavailable (e.g., Android, web).
    pub fn user_dirs(sys: System) -> Result<Option<UserDirs>, JsError> {
        if !sys.access.allows_home_dir() {
            return Err(home_dir_denied("userDirs"));
        }
        Ok(crate::UserDirs::new().map(UserDirs::from))
    }

    /// Computes platform-standard directories for a project identified by qualifier, organization, and application.
    pub fn project_dirs(qualifier: String, organization: String, application: String) -> Option<ProjectDirs> {
        crate::ProjectDirs::from(&qualifier, &organization, &application).map(From::from)
    }

    /// Computes project directories directly from a verbatim relative `projectPath` fragment.
    pub fn project_dirs_from_path(project_path: String) -> Option<ProjectDirs> {
        crate::ProjectDirs::from_path(PathBuf::from(project_path)).map(From::from)
    }
}
