#[cfg(any(target_os = "android", test))]
pub(crate) mod android;
#[cfg(any(target_os = "ios", test))]
pub(crate) mod ios;
#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
mod upstream;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_os = "android")]
pub(crate) use android::{base_dirs, project_dirs_from, project_dirs_from_path, user_dirs};
#[cfg(target_os = "ios")]
pub(crate) use ios::{base_dirs, project_dirs_from, project_dirs_from_path, user_dirs};
#[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
pub(crate) use upstream::{base_dirs, project_dirs_from, project_dirs_from_path, user_dirs};
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm::{base_dirs, project_dirs_from, project_dirs_from_path, user_dirs};
