#![allow(missing_docs)]

use std::path::{Path, PathBuf};

use nsh_directories::{
    BaseDirs, ProjectDirs, UPSTREAM_COMMIT, UPSTREAM_REPO, UPSTREAM_VERSION, UserDirs, available, provenance,
};

fn opt_path(path: Option<&Path>) -> Option<&Path> {
    path
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
fn assert_base_dirs_eq(actual: &BaseDirs, expected: &::directories::BaseDirs) {
    assert_eq!(actual.home_dir(), expected.home_dir());
    assert_eq!(actual.cache_dir(), expected.cache_dir());
    assert_eq!(actual.config_dir(), expected.config_dir());
    assert_eq!(actual.config_local_dir(), expected.config_local_dir());
    assert_eq!(actual.data_dir(), expected.data_dir());
    assert_eq!(actual.data_local_dir(), expected.data_local_dir());
    assert_eq!(actual.executable_dir(), expected.executable_dir());
    assert_eq!(actual.preference_dir(), expected.preference_dir());
    assert_eq!(actual.runtime_dir(), expected.runtime_dir());
    assert_eq!(actual.state_dir(), expected.state_dir());
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
fn assert_user_dirs_eq(actual: &UserDirs, expected: &::directories::UserDirs) {
    assert_eq!(actual.home_dir(), expected.home_dir());
    assert_eq!(actual.audio_dir(), expected.audio_dir());
    assert_eq!(actual.desktop_dir(), expected.desktop_dir());
    assert_eq!(actual.document_dir(), expected.document_dir());
    assert_eq!(actual.download_dir(), expected.download_dir());
    assert_eq!(actual.font_dir(), expected.font_dir());
    assert_eq!(actual.picture_dir(), expected.picture_dir());
    assert_eq!(actual.public_dir(), expected.public_dir());
    assert_eq!(actual.template_dir(), expected.template_dir());
    assert_eq!(actual.video_dir(), expected.video_dir());
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
fn assert_project_dirs_eq(actual: &ProjectDirs, expected: &::directories::ProjectDirs) {
    assert_eq!(actual.project_path(), expected.project_path());
    assert_eq!(actual.cache_dir(), expected.cache_dir());
    assert_eq!(actual.config_dir(), expected.config_dir());
    assert_eq!(actual.config_local_dir(), expected.config_local_dir());
    assert_eq!(actual.data_dir(), expected.data_dir());
    assert_eq!(actual.data_local_dir(), expected.data_local_dir());
    assert_eq!(actual.preference_dir(), expected.preference_dir());
    assert_eq!(actual.runtime_dir(), expected.runtime_dir());
    assert_eq!(actual.state_dir(), expected.state_dir());
}

#[test]
fn available_and_provenance_match_expected() {
    assert_eq!(available(), !cfg!(target_arch = "wasm32"));
    assert_eq!(UPSTREAM_REPO, "https://codeberg.org/dirs/directories-rs");
    assert_eq!(UPSTREAM_VERSION, "6.0.0");
    assert_eq!(UPSTREAM_COMMIT, "1f285d9114176e7be8b85b354e3bfde7c3e6356e");
    assert_eq!(provenance::UPSTREAM_CRATE, "directories");
    assert_eq!(provenance::UPSTREAM_DIRS_SYS_VERSION, "0.5.0");
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
#[test]
fn base_dirs_parity_with_upstream() {
    let actual = BaseDirs::new();
    let expected = ::directories::BaseDirs::new();
    assert_eq!(actual.is_some(), expected.is_some());
    if let (Some(actual), Some(expected)) = (actual.as_ref(), expected.as_ref()) {
        assert_base_dirs_eq(actual, expected);
    }
    let _ = opt_path(None);
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
#[test]
fn user_dirs_parity_with_upstream() {
    let actual = UserDirs::new();
    let expected = ::directories::UserDirs::new();
    assert_eq!(actual.is_some(), expected.is_some());
    if let (Some(actual), Some(expected)) = (actual.as_ref(), expected.as_ref()) {
        assert_user_dirs_eq(actual, expected);
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
#[test]
fn project_dirs_from_parity_with_upstream() {
    let cases = [
        ("com", "Foo Corp", "Bar App"),
        ("org", "Baz", "Qux"),
        ("", "Acme Inc", "Widget"),
        ("io.example", "", "My App"),
        ("", "", "Solo"),
        ("uk.co", "Foo Corp.", "Bar-App"),
    ];
    for (qualifier, organization, application) in cases {
        let actual = ProjectDirs::from(qualifier, organization, application);
        let expected = ::directories::ProjectDirs::from(qualifier, organization, application);
        assert_eq!(actual.is_some(), expected.is_some());
        if let (Some(actual), Some(expected)) = (actual.as_ref(), expected.as_ref()) {
            assert_project_dirs_eq(actual, expected);
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
#[test]
fn project_dirs_from_path_parity_with_upstream() {
    for raw in ["simple", "nested/app", "Foo Corp/Bar App"] {
        let path = PathBuf::from(raw);
        let actual = ProjectDirs::from_path(path.clone());
        let expected = ::directories::ProjectDirs::from_path(path);
        assert_eq!(actual.is_some(), expected.is_some());
        if let (Some(actual), Some(expected)) = (actual.as_ref(), expected.as_ref()) {
            assert_project_dirs_eq(actual, expected);
        }
    }
}

#[cfg(feature = "nsh")]
mod nsh_tests {
    use std::sync::Arc;

    use nsh_core::{
        access::{Access, Capabilities},
        element::{
            JsError, System, codes,
            description::{ExportKind, SetDescription},
        },
    };
    use nsh_directories::{
        BaseDirs as RustBaseDirs, ProjectDirs as RustProjectDirs, UserDirs as RustUserDirs,
        nsh::{BaseDirs as JsBaseDirs, Directories, ProjectDirs as JsProjectDirs, UserDirs as JsUserDirs},
        registry,
    };

    use super::*;

    fn sys_with(declared: Capabilities, granted: Option<Capabilities>) -> System {
        System::new(
            Arc::new(Access { declared, granted }),
            None,
            Default::default(),
            Default::default(),
        )
    }

    fn path_str(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    fn opt_path_str(path: Option<&Path>) -> Option<String> {
        path.map(path_str)
    }

    fn assert_js_base_dirs(js: &JsBaseDirs, rust: &RustBaseDirs, expected_home: Option<String>) {
        assert_eq!(js.home_dir, expected_home);
        assert_eq!(js.cache_dir, path_str(rust.cache_dir()));
        assert_eq!(js.config_dir, path_str(rust.config_dir()));
        assert_eq!(js.config_local_dir, path_str(rust.config_local_dir()));
        assert_eq!(js.data_dir, path_str(rust.data_dir()));
        assert_eq!(js.data_local_dir, path_str(rust.data_local_dir()));
        assert_eq!(js.executable_dir, opt_path_str(rust.executable_dir()));
        assert_eq!(js.preference_dir, path_str(rust.preference_dir()));
        assert_eq!(js.runtime_dir, opt_path_str(rust.runtime_dir()));
        assert_eq!(js.state_dir, opt_path_str(rust.state_dir()));
    }

    fn assert_js_user_dirs(js: &JsUserDirs, rust: &RustUserDirs) {
        assert_eq!(js.home_dir, path_str(rust.home_dir()));
        assert_eq!(js.audio_dir, opt_path_str(rust.audio_dir()));
        assert_eq!(js.desktop_dir, opt_path_str(rust.desktop_dir()));
        assert_eq!(js.document_dir, opt_path_str(rust.document_dir()));
        assert_eq!(js.download_dir, opt_path_str(rust.download_dir()));
        assert_eq!(js.font_dir, opt_path_str(rust.font_dir()));
        assert_eq!(js.picture_dir, opt_path_str(rust.picture_dir()));
        assert_eq!(js.public_dir, opt_path_str(rust.public_dir()));
        assert_eq!(js.template_dir, opt_path_str(rust.template_dir()));
        assert_eq!(js.video_dir, opt_path_str(rust.video_dir()));
    }

    fn assert_js_project_dirs(js: &JsProjectDirs, rust: &RustProjectDirs) {
        assert_eq!(js.project_path, path_str(rust.project_path()));
        assert_eq!(js.cache_dir, path_str(rust.cache_dir()));
        assert_eq!(js.config_dir, path_str(rust.config_dir()));
        assert_eq!(js.config_local_dir, path_str(rust.config_local_dir()));
        assert_eq!(js.data_dir, path_str(rust.data_dir()));
        assert_eq!(js.data_local_dir, path_str(rust.data_local_dir()));
        assert_eq!(js.preference_dir, path_str(rust.preference_dir()));
        assert_eq!(js.runtime_dir, opt_path_str(rust.runtime_dir()));
        assert_eq!(js.state_dir, opt_path_str(rust.state_dir()));
    }

    fn assert_access_denied(err: JsError, fn_name: &str) {
        assert_eq!(err.code(), Some(codes::ACCESS_DENIED));
        assert!(
            err.message().contains("\"homeDir\": true"),
            "unexpected error message: {}",
            err.message()
        );
        assert!(
            err.message().contains(fn_name),
            "expected function name `{fn_name}` in error message: {}",
            err.message()
        );
        match err {
            JsError::Fields(fields) => {
                assert_eq!(fields.field("permission").and_then(|v| v.as_str()), Some("homeDir"));
                assert_eq!(fields.field("resource").and_then(|v| v.as_str()), Some(""));
            }
            other => panic!("expected JsError::Fields, got {other:?}"),
        }
    }

    #[test]
    fn registry_description_and_declarations() {
        let desc = SetDescription::of(&registry()).expect("registry has specifier");
        assert_eq!(desc.specifier, "nsh-directories");
        assert!(
            desc.exports
                .iter()
                .any(|e| e.name == "Directories" && e.kind == ExportKind::Module),
            "exports: {:?}",
            desc.exports
        );
        if nsh_core::element::DECLARATIONS {
            assert!(
                desc.undocumented.is_empty(),
                "undocumented exports: {:?}",
                desc.undocumented
            );
            for snippet in [
                "export interface BaseDirs",
                "export interface UserDirs",
                "export interface ProjectDirs",
                "export interface DirectoriesInfo",
                "export interface DirectoriesModule",
                "export declare const Directories: DirectoriesModule;",
                "available(): boolean;",
                "info(): DirectoriesInfo;",
                "homeDir(): string | null;",
                "baseDirs(): BaseDirs | null;",
                "userDirs(): UserDirs | null;",
                "projectDirs(qualifier: string, organization: string, application: string): ProjectDirs | null;",
                "projectDirsFromPath(projectPath: string): ProjectDirs | null;",
            ] {
                assert!(
                    desc.declarations.contains(snippet),
                    "missing `{snippet}` in declarations:\n{}",
                    desc.declarations
                );
            }
        }
    }

    #[test]
    fn homedir_capability_denied_gates_homedir_and_userdirs_only() {
        let denied_systems = [
            sys_with(Capabilities::default(), None),
            sys_with(Capabilities::all(), Some(Capabilities::default())),
        ];

        for sys in denied_systems {
            let home_err = Directories::home_dir(sys.clone()).expect_err("homeDir must require homeDir capability");
            assert_access_denied(home_err, "homeDir");

            let user_err = Directories::user_dirs(sys.clone()).expect_err("userDirs must require homeDir capability");
            assert_access_denied(user_err, "userDirs");

            // Ungated APIs still return standard paths, with `BaseDirs.homeDir` redacted to `None`.
            assert_eq!(Directories::available(), available());
            let info = Directories::info();
            assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
            assert_eq!(info.upstream_version, UPSTREAM_VERSION);
            assert_eq!(info.upstream_commit, UPSTREAM_COMMIT);
            assert_eq!(info.available, available());

            let js_base = Directories::base_dirs(sys);
            let rust_base = RustBaseDirs::new();
            assert_eq!(js_base.is_some(), rust_base.is_some());
            if let (Some(js_base), Some(rust_base)) = (js_base.as_ref(), rust_base.as_ref()) {
                assert_js_base_dirs(js_base, rust_base, None);
            }

            let js_proj = Directories::project_dirs("com".into(), "Foo Corp".into(), "Bar App".into());
            let rust_proj = RustProjectDirs::from("com", "Foo Corp", "Bar App");
            assert_eq!(js_proj.is_some(), rust_proj.is_some());
            if let (Some(js_proj), Some(rust_proj)) = (js_proj.as_ref(), rust_proj.as_ref()) {
                assert_js_project_dirs(js_proj, rust_proj);
            }

            let js_proj_path = Directories::project_dirs_from_path("custom/app".into());
            let rust_proj_path = RustProjectDirs::from_path(PathBuf::from("custom/app"));
            assert_eq!(js_proj_path.is_some(), rust_proj_path.is_some());
            if let (Some(js_proj_path), Some(rust_proj_path)) = (js_proj_path.as_ref(), rust_proj_path.as_ref()) {
                assert_js_project_dirs(js_proj_path, rust_proj_path);
            }
        }
    }

    #[test]
    fn homedir_capability_granted_returns_full_snapshots() {
        let mut home_only = Capabilities::default();
        home_only.home_dir = true;
        let granted_systems = [sys_with(Capabilities::all(), None), sys_with(home_only, None)];

        for sys in granted_systems {
            let rust_base = RustBaseDirs::new();
            let expected_home = rust_base.as_ref().map(|b| path_str(b.home_dir()));

            let js_home = Directories::home_dir(sys.clone()).expect("homeDir granted");
            assert_eq!(js_home, expected_home);

            let js_base = Directories::base_dirs(sys.clone());
            assert_eq!(js_base.is_some(), rust_base.is_some());
            if let (Some(js_base), Some(rust_base)) = (js_base.as_ref(), rust_base.as_ref()) {
                assert_js_base_dirs(js_base, rust_base, expected_home);
            }

            let js_user = Directories::user_dirs(sys).expect("userDirs granted");
            let rust_user = RustUserDirs::new();
            assert_eq!(js_user.is_some(), rust_user.is_some());
            if let (Some(js_user), Some(rust_user)) = (js_user.as_ref(), rust_user.as_ref()) {
                assert_js_user_dirs(js_user, rust_user);
            }
        }
    }
}
