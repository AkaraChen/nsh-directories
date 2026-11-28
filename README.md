# nsh-directories

Platform-standard configuration, cache, data, runtime, and user directories for [Native Shell](https://nsh.dev) apps in Rust and TSX.

`nsh-directories` is a thin, non-forked wrapper around [`directories`](https://crates.io/crates/directories) `6.0.0` (`dirs-sys` `0.5.0`, upstream commit [`1f285d91`](https://codeberg.org/dirs/directories-rs/commit/1f285d9114176e7be8b85b354e3bfde7c3e6356e)) that preserves 100% parity on Linux, macOS, Windows, and `wasm32`, while adding Native Shell TSX bindings and fixing mobile sandbox behavior on Android and iOS.

## Installation

### Native Shell TSX app

```sh
nsh add github:AkaraChen/nsh-directories
```

### Pure Rust crate (without NSH bindings)

```toml
[dependencies]
nsh-directories = { git = "https://github.com/AkaraChen/nsh-directories", default-features = false }
```

Keep default features enabled (`features = ["nsh"]`) to register the `#[native_module("nsh-directories")]` module automatically for Native Shell apps. Linking `nsh-directories` never monkey-patches `os.homedir()`, `process.dataDir`, or `nsh/storage`.

## TSX Usage

```tsx
import { Directories } from "nsh-directories";

if (Directories.available()) {
  const project = Directories.projectDirs("com", "Example Corp", "My App");
  if (project) {
    console.log(project.configDir, project.cacheDir, project.runtimeDir ?? "none");
  }

  const base = Directories.baseDirs();
  if (base) {
    // Plain JS object snapshot: supports spread and JSON.stringify.
    console.log(base.cacheDir, base.configDir, base.homeDir);
  }

  // Requires `"homeDir": true` when `capabilities` is declared in nsh.json:
  const home = Directories.homeDir();
  const user = Directories.userDirs();
  console.log(home, user?.documentDir);
}
```

### TSX API Summary

| Member | `capabilities.homeDir` required? | Description |
|---|---|---|
| `Directories.available()` | No | `true` on native targets, `false` on Web (`wasm32`). |
| `Directories.info()` | No | Crate version, pinned upstream version/commit, and `available`. |
| `Directories.baseDirs()` | Partial (`homeDir` field only) | Returns plain `BaseDirs \| null`. When `capabilities.homeDir` is not granted, `homeDir` is `null` while `cacheDir`, `configDir`, `configLocalDir`, `dataDir`, `dataLocalDir`, `executableDir`, `preferenceDir`, `runtimeDir`, and `stateDir` are still returned. |
| `Directories.homeDir()` | **Yes** | Returns `string \| null`. Throws `ERR_ACCESS_DENIED` (`permission: "homeDir"`) if ungranted. |
| `Directories.userDirs()` | **Yes** | Returns `UserDirs \| null`. Throws `ERR_ACCESS_DENIED` (`permission: "homeDir"`) if ungranted. |
| `Directories.projectDirs(qualifier, organization, application)` | No | Returns plain `ProjectDirs \| null` using platform-standard project naming. |
| `Directories.projectDirsFromPath(projectPath)` | No | Returns plain `ProjectDirs \| null` using `projectPath` verbatim. |

> **Note on filesystem permissions**: Directory strings returned by `nsh-directories` are metadata only and do **not** grant filesystem read/write access. Calling `fs/promises` on returned paths still requires `capabilities.fs.read` / `capabilities.fs.write` in `nsh.json`. Use `nsh/storage` for capability-free app-private key-value storage.

## Rust Usage

```rust
use nsh_directories::{BaseDirs, ProjectDirs, UserDirs, available};

if available() {
    if let Some(base) = BaseDirs::new() {
        println!("config: {}", base.config_dir().display());
    }
    if let Some(user) = UserDirs::new() {
        println!("documents: {:?}", user.document_dir());
    }
    if let Some(proj) = ProjectDirs::from("com", "Example Corp", "My App") {
        println!("cache: {}", proj.cache_dir().display());
    }
}
```

## Platform Behavior & Intentional Upstream Divergences

| Platform | `BaseDirs` | `UserDirs` | `ProjectDirs` | Notes |
|---|---|---|---|---|
| **Linux / BSD / Redox** | Upstream `lin.rs` (XDG Base Directory) | Upstream `lin.rs` (`user-dirs.dirs` + `$XDG_DATA_HOME/fonts`) | Upstream `lin.rs` (`barapp`) | 100% parity with `directories = "6.0.0"`. |
| **macOS** | Upstream `mac.rs` (`~/Library/{Application Support,Caches,Preferences}`) | Upstream `mac.rs` | Upstream `mac.rs` (`com.Foo-Corp.Bar-App`) | 100% parity with `directories = "6.0.0"`. |
| **Windows** | Upstream `win.rs` (Known Folders) | Upstream `win.rs` (Known Folders) | Upstream `win.rs` (`Foo Corp\Bar App\{cache,config,data}`) | 100% parity with `directories = "6.0.0"`. |
| **iOS** | Upstream `mac.rs` inside sandbox `$HOME` | **Overridden**: `home_dir` + `document_dir` (`$HOME/Documents`) only; all other 8 fields `None` | Upstream `mac.rs` inside sandbox `$HOME` | Intentional divergence: upstream `mac.rs` emits non-existent `$HOME/Desktop`, `$HOME/Music`, etc. on iOS. Note that `Library/Application Support` may need `mkdir({ recursive: true })` before first write, `home_dir` is the container root (do not write arbitrary business files there), and `Library/Preferences` is managed by `NSUserDefaults`. |
| **Android** | **Overridden**: `ndk-context` + `jni` queries `Context.{getDataDir, getFilesDir, getCacheDir, getNoBackupFilesDir}` | **Overridden**: `None` | **Overridden**: joins reverse-DNS `project_path` onto `filesDir` / `cacheDir` / `noBackupFilesDir` | Intentional divergence: upstream routes Android to `lin.rs` (`$HOME`), which returns `None` or invalid paths inside APKs. Queries are cached in `OnceLock` once initialized and return `None` safely without panicking if `ndk-context` is not yet initialized (under `panic = "unwind"`). Scoped Storage prohibits direct POSIX writes to shared user directories, so `UserDirs::new()` returns `None`. |
| **HarmonyOS / OpenHarmony (`target_env = "ohos"`, spike)** | `todo!()` | `todo!()` | `todo!()` | **Placeholder only.** OHOS reports `target_os = "linux"`, so it is routed away from upstream `lin.rs` (whose `$HOME`/XDG paths are meaningless in an app sandbox) to [`src/sys/ohos.rs`](src/sys/ohos.rs). The API compiles for `aarch64-unknown-linux-ohos` (e.g. under [`nsh-platform-harmonyos`](https://github.com/huacnlee/nsh-platform-harmonyos)), but every directory lookup **panics**; `available()` is `false` so apps can feature-detect. Capability checks in TSX still run first. A real implementation should read the ability context (`filesDir` / `cacheDir` / `tempDir`). |
| **Web (`wasm32`)** | `None` / `null` | `None` / `null` (still checks `capabilities.homeDir` in TSX) | `None` / `null` | `available()` is `false`. Unlike `os.homedir()` (which throws `ERR_FEATURE_UNAVAILABLE_ON_PLATFORM` because its type is non-null `string`), `Directories` factories return `null` after capability checks pass. |

## Upstream Pinning & Sync

Upstream provenance is pinned in [`upstream.json`](upstream.json), [`Cargo.toml`](Cargo.toml), [`Cargo.lock`](Cargo.lock), [`src/provenance.rs`](src/provenance.rs), and [`NOTICE`](NOTICE):

```sh
# Verify pinned version, commit, Cargo.lock, provenance.rs, and API surface
node scripts/sync-upstream.mjs --check

# Check Codeberg main & crates.io for upstream drift
node scripts/sync-upstream.mjs --check-latest
```

## Verification Scope

Automated CI checks cover formatting, Clippy (Linux + `wasm32`, Android, iOS, with and without `nsh`), full field-by-field parity tests against `::directories` on Linux/macOS/Windows, Linux XDG env-override parity, host unit tests for Android and iOS mapping functions, cross-compilation checks (`wasm32`, Android, iOS), headless `nsh test` + real-platform E2E tests (`nsh test --target web` in headless Chrome, `nsh test --target ios` in the Xcode iOS Simulator on `macos-15`, and `nsh test --target android` in an API 35 `x86_64` Android Emulator with KVM exercising `ndk-context` + JNI `Context.{getDataDir, getFilesDir, getCacheDir, getNoBackupFilesDir}`) across both `examples/playground` and `examples/denied-cap`, and upstream sync verification.

The following items are **not** executed in automated CI and remain unverified at runtime:
- Physical Android and iOS hardware devices
- Mac Catalyst runtime (`--target catalyst`)
- HarmonyOS / OpenHarmony (only `cargo check --target aarch64-unknown-linux-ohos`; lookups are `todo!()`)
- Published static site deployment via `nsh package --target web`
- Real-device mobile/browser runtime performance

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE). See [NOTICE](NOTICE) for upstream attribution.
