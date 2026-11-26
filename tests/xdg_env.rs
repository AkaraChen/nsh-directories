#![allow(missing_docs)]
#![cfg(all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))))]

use std::ffi::OsString;
use std::path::PathBuf;

use nsh_directories::{BaseDirs, ProjectDirs, UserDirs};

const XDG_VARS: [&str; 6] = [
    "XDG_CACHE_HOME",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "XDG_STATE_HOME",
    "XDG_BIN_HOME",
    "XDG_RUNTIME_DIR",
];

struct EnvGuard {
    saved: Vec<(&'static str, Option<OsString>)>,
}

impl EnvGuard {
    fn capture(keys: &[&'static str]) -> Self {
        let saved = keys.iter().map(|&k| (k, std::env::var_os(k))).collect();
        Self { saved }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.saved {
            match value {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }
}

fn assert_full_parity() {
    let actual_base = BaseDirs::new().expect("BaseDirs::new");
    let expected_base = ::directories::BaseDirs::new().expect("upstream BaseDirs::new");
    assert_eq!(actual_base.home_dir(), expected_base.home_dir());
    assert_eq!(actual_base.cache_dir(), expected_base.cache_dir());
    assert_eq!(actual_base.config_dir(), expected_base.config_dir());
    assert_eq!(actual_base.config_local_dir(), expected_base.config_local_dir());
    assert_eq!(actual_base.data_dir(), expected_base.data_dir());
    assert_eq!(actual_base.data_local_dir(), expected_base.data_local_dir());
    assert_eq!(actual_base.executable_dir(), expected_base.executable_dir());
    assert_eq!(actual_base.preference_dir(), expected_base.preference_dir());
    assert_eq!(actual_base.runtime_dir(), expected_base.runtime_dir());
    assert_eq!(actual_base.state_dir(), expected_base.state_dir());

    let actual_user = UserDirs::new().expect("UserDirs::new");
    let expected_user = ::directories::UserDirs::new().expect("upstream UserDirs::new");
    assert_eq!(actual_user.home_dir(), expected_user.home_dir());
    assert_eq!(actual_user.font_dir(), expected_user.font_dir());

    let actual_proj = ProjectDirs::from("com", "Foo Corp", "Bar App").expect("ProjectDirs::from");
    let expected_proj =
        ::directories::ProjectDirs::from("com", "Foo Corp", "Bar App").expect("upstream ProjectDirs::from");
    assert_eq!(actual_proj.project_path(), expected_proj.project_path());
    assert_eq!(actual_proj.cache_dir(), expected_proj.cache_dir());
    assert_eq!(actual_proj.config_dir(), expected_proj.config_dir());
    assert_eq!(actual_proj.config_local_dir(), expected_proj.config_local_dir());
    assert_eq!(actual_proj.data_dir(), expected_proj.data_dir());
    assert_eq!(actual_proj.data_local_dir(), expected_proj.data_local_dir());
    assert_eq!(actual_proj.preference_dir(), expected_proj.preference_dir());
    assert_eq!(actual_proj.runtime_dir(), expected_proj.runtime_dir());
    assert_eq!(actual_proj.state_dir(), expected_proj.state_dir());

    let actual_raw = ProjectDirs::from_path(PathBuf::from("custom-app")).expect("ProjectDirs::from_path");
    let expected_raw = ::directories::ProjectDirs::from_path(PathBuf::from("custom-app")).expect("upstream from_path");
    assert_eq!(actual_raw.cache_dir(), expected_raw.cache_dir());
    assert_eq!(actual_raw.runtime_dir(), expected_raw.runtime_dir());
    assert_eq!(actual_raw.state_dir(), expected_raw.state_dir());
}

#[test]
fn xdg_env_overrides_and_relative_fallbacks_match_upstream() {
    let _guard = EnvGuard::capture(&XDG_VARS);

    // 1. Valid absolute XDG overrides.
    unsafe {
        std::env::set_var("XDG_CACHE_HOME", "/tmp/nsh-xdg-test/cache");
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/nsh-xdg-test/config");
        std::env::set_var("XDG_DATA_HOME", "/tmp/nsh-xdg-test/data");
        std::env::set_var("XDG_STATE_HOME", "/tmp/nsh-xdg-test/state");
        std::env::set_var("XDG_BIN_HOME", "/tmp/nsh-xdg-test/bin");
        std::env::set_var("XDG_RUNTIME_DIR", "/tmp/nsh-xdg-test/runtime");
    }
    assert_full_parity();
    let base = BaseDirs::new().unwrap();
    assert_eq!(base.cache_dir(), PathBuf::from("/tmp/nsh-xdg-test/cache").as_path());
    assert_eq!(base.config_dir(), PathBuf::from("/tmp/nsh-xdg-test/config").as_path());
    assert_eq!(base.data_dir(), PathBuf::from("/tmp/nsh-xdg-test/data").as_path());
    assert_eq!(
        base.state_dir(),
        Some(PathBuf::from("/tmp/nsh-xdg-test/state").as_path())
    );
    assert_eq!(
        base.executable_dir(),
        Some(PathBuf::from("/tmp/nsh-xdg-test/bin").as_path())
    );
    assert_eq!(
        base.runtime_dir(),
        Some(PathBuf::from("/tmp/nsh-xdg-test/runtime").as_path())
    );

    // 2. Relative XDG paths must be ignored per XDG spec, matching upstream fallbacks.
    unsafe {
        std::env::set_var("XDG_CACHE_HOME", "relative/cache");
        std::env::set_var("XDG_CONFIG_HOME", "relative/config");
        std::env::set_var("XDG_DATA_HOME", "relative/data");
        std::env::set_var("XDG_STATE_HOME", "relative/state");
        std::env::set_var("XDG_BIN_HOME", "relative/bin");
        std::env::set_var("XDG_RUNTIME_DIR", "relative/runtime");
    }
    assert_full_parity();
    let fallback = BaseDirs::new().unwrap();
    assert_eq!(fallback.cache_dir(), fallback.home_dir().join(".cache").as_path());
    assert_eq!(fallback.runtime_dir(), None);
}
