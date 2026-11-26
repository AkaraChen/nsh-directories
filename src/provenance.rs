//! Provenance metadata for the pinned upstream `directories` crate.
//!
//! Maintained by `scripts/sync-upstream.mjs`.

/// Upstream source repository URL.
pub const UPSTREAM_REPO: &str = "https://codeberg.org/dirs/directories-rs";

/// Upstream crates.io package name.
pub const UPSTREAM_CRATE: &str = "directories";

/// Pinned upstream `directories` crate version.
pub const UPSTREAM_VERSION: &str = "6.0.0";

/// Pinned upstream repository commit SHA.
pub const UPSTREAM_COMMIT: &str = "1f285d9114176e7be8b85b354e3bfde7c3e6356e";

/// Pinned upstream `dirs-sys` version used by `directories`.
pub const UPSTREAM_DIRS_SYS_VERSION: &str = "0.5.0";
