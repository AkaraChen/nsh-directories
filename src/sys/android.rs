use std::path::PathBuf;
#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
use jni::{
    JNIEnv, JavaVM,
    objects::{JObject, JString},
};

use crate::{BaseDirs, ProjectDirs, UserDirs};

/// App-private directories resolved from `android.content.Context`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AndroidPaths {
    /// `Context.getDataDir()` (API 24+), e.g. `/data/user/0/<pkg>`.
    pub(crate) data_dir: Option<PathBuf>,
    /// `Context.getFilesDir()`, e.g. `/data/user/0/<pkg>/files`.
    pub(crate) files_dir: PathBuf,
    /// `Context.getCacheDir()`, e.g. `/data/user/0/<pkg>/cache`.
    pub(crate) cache_dir: PathBuf,
    /// `Context.getNoBackupFilesDir()`, e.g. `/data/user/0/<pkg>/no_backup`.
    pub(crate) no_backup_files_dir: Option<PathBuf>,
}

/// Maps `AndroidPaths` into [`BaseDirs`].
pub(crate) fn base_dirs_from_paths(paths: &AndroidPaths) -> BaseDirs {
    let home_dir = paths
        .data_dir
        .clone()
        .or_else(|| paths.files_dir.parent().map(PathBuf::from))
        .unwrap_or_else(|| paths.files_dir.clone());
    let local_dir = paths
        .no_backup_files_dir
        .clone()
        .unwrap_or_else(|| paths.files_dir.clone());

    BaseDirs {
        home_dir,
        cache_dir: paths.cache_dir.clone(),
        config_dir: paths.files_dir.clone(),
        config_local_dir: local_dir.clone(),
        data_dir: paths.files_dir.clone(),
        data_local_dir: local_dir,
        executable_dir: None,
        preference_dir: paths.files_dir.clone(),
        runtime_dir: None,
        state_dir: None,
    }
}

/// Formats `(qualifier, organization, application)` into a reverse-DNS bundle
/// segment matching upstream Apple/macOS conventions (`com.Foo-Corp.Bar-App`).
pub(crate) fn format_project_path(qualifier: &str, organization: &str, application: &str) -> PathBuf {
    let organization = organization.replace(' ', "-");
    let application = application.replace(' ', "-");
    let parts: Vec<&str> = [qualifier, organization.as_str(), application.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect();
    PathBuf::from(parts.join("."))
}

/// Maps `AndroidPaths` and a relative `project_path` into [`ProjectDirs`].
pub(crate) fn project_dirs_from_paths(paths: &AndroidPaths, project_path: PathBuf) -> ProjectDirs {
    let local_root = paths
        .no_backup_files_dir
        .as_deref()
        .unwrap_or(paths.files_dir.as_path());
    let shared_dir = paths.files_dir.join(&project_path);
    let local_dir = local_root.join(&project_path);

    ProjectDirs {
        cache_dir: paths.cache_dir.join(&project_path),
        config_dir: shared_dir.clone(),
        config_local_dir: local_dir.clone(),
        data_dir: shared_dir.clone(),
        data_local_dir: local_dir,
        preference_dir: shared_dir,
        project_path,
        runtime_dir: None,
        state_dir: None,
    }
}

/// Maps `AndroidPaths` and project metadata into [`ProjectDirs`].
pub(crate) fn project_dirs_from_parts(
    paths: &AndroidPaths,
    qualifier: &str,
    organization: &str,
    application: &str,
) -> ProjectDirs {
    let project_path = format_project_path(qualifier, organization, application);
    project_dirs_from_paths(paths, project_path)
}

/// On Android, `UserDirs::new()` always returns `None`: Scoped Storage
/// prohibits direct POSIX writes to shared user directories, and
/// `getExternalFilesDir` is app-private and deleted on uninstall.
pub(crate) fn user_dirs_android() -> Option<UserDirs> {
    None
}

#[cfg(target_os = "android")]
static ANDROID_PATHS: OnceLock<AndroidPaths> = OnceLock::new();

#[cfg(target_os = "android")]
fn call_context_dir(env: &mut JNIEnv<'_>, context: &JObject<'_>, method: &str) -> Option<PathBuf> {
    let result = (|| -> jni::errors::Result<Option<PathBuf>> {
        let file = env.call_method(context, method, "()Ljava/io/File;", &[])?.l()?;
        if file.is_null() {
            return Ok(None);
        }
        let path_obj = env
            .call_method(&file, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?;
        if path_obj.is_null() {
            return Ok(None);
        }
        let path_str = JString::from(path_obj);
        let rust_str = String::from(env.get_string(&path_str)?);
        if rust_str.is_empty() {
            return Ok(None);
        }
        Ok(Some(PathBuf::from(rust_str)))
    })();
    match result {
        Ok(path) => path,
        Err(_) => {
            let _ = env.exception_clear();
            None
        }
    }
}

#[cfg(target_os = "android")]
fn query_android_paths() -> Option<AndroidPaths> {
    // We use BOTH `catch_unwind` and explicit null-pointer checks here:
    // `ndk_context::android_context()` panics via `.expect(...)` when NSH/the
    // host has not yet called `initialize_android_context`, so pointer null
    // checks alone cannot prevent a panic in uninitialized hosts (note: under
    // `panic = "abort"`, calling before initialization would still abort inside
    // `ndk-context`, whereas NSH initializes it when creating the first `NshView`).
    let ctx = std::panic::catch_unwind(ndk_context::android_context).ok()?;
    if ctx.vm().is_null() || ctx.context().is_null() {
        return None;
    }

    // SAFETY: `ctx.vm()` is non-null and initialized by `ndk-context` for the
    // lifetime of the Android process.
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    // SAFETY: `ctx.context()` is a global reference managed by `ndk-context`
    // and is only borrowed as `&JObject` within this call.
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    env.with_local_frame::<_, Option<AndroidPaths>, jni::errors::Error>(16, |env| {
        let Some(files_dir) = call_context_dir(env, &context, "getFilesDir") else {
            return Ok(None);
        };
        let Some(cache_dir) = call_context_dir(env, &context, "getCacheDir") else {
            return Ok(None);
        };
        let data_dir = call_context_dir(env, &context, "getDataDir");
        let no_backup_files_dir = call_context_dir(env, &context, "getNoBackupFilesDir");
        Ok(Some(AndroidPaths {
            data_dir,
            files_dir,
            cache_dir,
            no_backup_files_dir,
        }))
    })
    .ok()
    .flatten()
}

#[cfg(target_os = "android")]
fn android_paths() -> Option<&'static AndroidPaths> {
    if let Some(cached) = ANDROID_PATHS.get() {
        return Some(cached);
    }
    // Only cache successful lookups so a pre-initialization call returning
    // `None` does not permanently poison subsequent calls once initialized.
    let queried = query_android_paths()?;
    let _ = ANDROID_PATHS.set(queried);
    ANDROID_PATHS.get()
}

#[cfg(target_os = "android")]
pub(crate) fn base_dirs() -> Option<BaseDirs> {
    android_paths().map(base_dirs_from_paths)
}

#[cfg(target_os = "android")]
pub(crate) fn user_dirs() -> Option<UserDirs> {
    user_dirs_android()
}

#[cfg(target_os = "android")]
pub(crate) fn project_dirs_from_path(project_path: PathBuf) -> Option<ProjectDirs> {
    android_paths().map(|paths| project_dirs_from_paths(paths, project_path))
}

#[cfg(target_os = "android")]
pub(crate) fn project_dirs_from(qualifier: &str, organization: &str, application: &str) -> Option<ProjectDirs> {
    android_paths().map(|paths| project_dirs_from_parts(paths, qualifier, organization, application))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn sample_paths() -> AndroidPaths {
        AndroidPaths {
            data_dir: Some(PathBuf::from("/data/user/0/com.example.app")),
            files_dir: PathBuf::from("/data/user/0/com.example.app/files"),
            cache_dir: PathBuf::from("/data/user/0/com.example.app/cache"),
            no_backup_files_dir: Some(PathBuf::from("/data/user/0/com.example.app/no_backup")),
        }
    }

    #[test]
    fn android_base_dirs_maps_context_directories() {
        let paths = sample_paths();
        let dirs = base_dirs_from_paths(&paths);
        assert_eq!(dirs.home_dir(), Path::new("/data/user/0/com.example.app"));
        assert_eq!(dirs.cache_dir(), Path::new("/data/user/0/com.example.app/cache"));
        assert_eq!(dirs.config_dir(), Path::new("/data/user/0/com.example.app/files"));
        assert_eq!(dirs.data_dir(), Path::new("/data/user/0/com.example.app/files"));
        assert_eq!(dirs.preference_dir(), Path::new("/data/user/0/com.example.app/files"));
        assert_eq!(
            dirs.config_local_dir(),
            Path::new("/data/user/0/com.example.app/no_backup")
        );
        assert_eq!(
            dirs.data_local_dir(),
            Path::new("/data/user/0/com.example.app/no_backup")
        );
        assert_eq!(dirs.executable_dir(), None);
        assert_eq!(dirs.runtime_dir(), None);
        assert_eq!(dirs.state_dir(), None);
    }

    #[test]
    fn android_fallbacks_when_data_dir_or_no_backup_is_missing() {
        let paths = AndroidPaths {
            data_dir: None,
            files_dir: PathBuf::from("/data/user/0/com.example.app/files"),
            cache_dir: PathBuf::from("/data/user/0/com.example.app/cache"),
            no_backup_files_dir: None,
        };
        let base = base_dirs_from_paths(&paths);
        assert_eq!(base.home_dir(), Path::new("/data/user/0/com.example.app"));
        assert_eq!(base.config_local_dir(), Path::new("/data/user/0/com.example.app/files"));
        assert_eq!(base.data_local_dir(), Path::new("/data/user/0/com.example.app/files"));

        let proj = project_dirs_from_parts(&paths, "com", "Foo Corp", "Bar App");
        assert_eq!(proj.project_path(), Path::new("com.Foo-Corp.Bar-App"));
        assert_eq!(
            proj.config_local_dir(),
            Path::new("/data/user/0/com.example.app/files/com.Foo-Corp.Bar-App")
        );
        assert_eq!(
            proj.data_local_dir(),
            Path::new("/data/user/0/com.example.app/files/com.Foo-Corp.Bar-App")
        );
    }

    #[test]
    fn android_project_dirs_formats_reverse_dns_and_verbatim_paths() {
        let paths = sample_paths();
        let proj = project_dirs_from_parts(&paths, "com", "Foo Corp", "Bar App");
        assert_eq!(proj.project_path(), Path::new("com.Foo-Corp.Bar-App"));
        assert_eq!(
            proj.cache_dir(),
            Path::new("/data/user/0/com.example.app/cache/com.Foo-Corp.Bar-App")
        );
        assert_eq!(
            proj.config_dir(),
            Path::new("/data/user/0/com.example.app/files/com.Foo-Corp.Bar-App")
        );
        assert_eq!(
            proj.config_local_dir(),
            Path::new("/data/user/0/com.example.app/no_backup/com.Foo-Corp.Bar-App")
        );
        assert_eq!(
            proj.data_dir(),
            Path::new("/data/user/0/com.example.app/files/com.Foo-Corp.Bar-App")
        );
        assert_eq!(
            proj.data_local_dir(),
            Path::new("/data/user/0/com.example.app/no_backup/com.Foo-Corp.Bar-App")
        );
        assert_eq!(
            proj.preference_dir(),
            Path::new("/data/user/0/com.example.app/files/com.Foo-Corp.Bar-App")
        );
        assert_eq!(proj.runtime_dir(), None);
        assert_eq!(proj.state_dir(), None);

        let empty_org = project_dirs_from_parts(&paths, "io.example", "", "My App");
        assert_eq!(empty_org.project_path(), Path::new("io.example.My-App"));

        let verbatim = project_dirs_from_paths(&paths, PathBuf::from("custom/nested"));
        assert_eq!(verbatim.project_path(), Path::new("custom/nested"));
        assert_eq!(
            verbatim.cache_dir(),
            Path::new("/data/user/0/com.example.app/cache/custom/nested")
        );
    }

    #[test]
    fn android_user_dirs_is_always_none() {
        assert_eq!(user_dirs_android(), None);
    }
}
