# Platform-Specific Dependency Impact Analysis

> Generated: 2026-06-01
> Scope: Platform-gated deps, `#[cfg(...)]` code, Android concerns, Tauri version interop, vendored crate compatibility

---

## 1. Tauri Version

| Property | Value |
|----------|-------|
| Specified version (Cargo.toml) | `"2"` |
| Resolved version (Cargo.lock) | **2.10.2** |
| Internal reqwest dep | 0.13.2 |
| Internal hyper dep | 1.8.1 |
| Internal rustls dep | 0.23.36 |

**Source files**: `src-tauri/Cargo.toml` (line 72), `src-tauri/Cargo.lock` (line 10473)

---

## 2. Platform-Specific Dependency Sections in Cargo.toml

Four `[target.'cfg(...)'.dependencies]` blocks exist:

### 2a. Desktop-only (macOS/Windows/Linux)
```toml
[target.'cfg(any(target_os = "macos", windows, target_os = "linux"))'.dependencies]
tauri-plugin-updater = "2"    # resolved: 2.10.0
tauri-plugin-process = "2"   # resolved: 2.3.1
```
- These are excluded on Android/iOS.
- `tauri-plugin-updater` internally requires **reqwest 0.13.2** and **rustls 0.23.36**.
- **Impact**: If consolidating reqwest to a single version across the project, this plugin forces 0.13.x into the tree. Cannot be removed without losing auto-update on desktop.

### 2b. Non-Android (OAuth2)
```toml
[target.'cfg(not(target_os = "android"))'.dependencies]
oauth2 = "5.0"
pkce = "0.2"
```
- OAuth2 requires native-tls, which is problematic on Android NDK cross-compilation.
- Entire `mcp::auth` module is gated behind `#[cfg(not(target_os = "android"))]`.
- **Impact**: These are NOT target crates for consolidation. No impact.

### 2c. macOS-only
```toml
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"
```
- Used for native menu bar and window styling (lib.rs lines 800-840).
- **Impact**: NOT target crates for consolidation. No impact.

### 2d. Windows-only
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Media_Ocr", ...] }
```
- Used for Windows system OCR adapter.
- **Impact**: NOT a target crate for consolidation. No impact.

---

## 3. Platform-Gated Code Analysis

### 3.1 Count of `#[cfg(...)]` Occurrences

| Pattern | Count | Files Affected |
|---------|-------|---------------|
| `#[cfg(target_os = "...")]` | 34 | 9 files |
| `#[cfg(not(target_os = "..."))]` | 4 | 3 files |
| `#[cfg(any(...))]` | 8 | 4 files |
| `#[cfg(windows)]` | 9 | 3 files |
| `#[cfg(unix)]` | 14 | 6 files |
| **Total (approximate)** | **~69** | **15 files** |

### 3.2 All Platform-Gated Files and Their Crate Usage

| File | Platform Gates | Uses reqwest/hyper/rustls? | Notes |
|------|---------------|---------------------------|-------|
| `crash_logger.rs` | windows, not(windows) | No. Uses `regex` only. | PII scrubbing with different path patterns per OS. |
| `commands.rs` | windows, macos, linux | No. Uses `std::process::Command`. | Platform-appropriate file opener (notepad vs open vs xdg-open). |
| `cmd/ocr.rs` | macos, windows, ios | No. Returns string literals. | Display name for system OCR provider. |
| `lib.rs` | macos, linux, windows, android, ios | No. Uses `std::env`, `cocoa`, `objc`. | Window setup, env vars, AppImage runtime. |
| `mcp/mod.rs` | not(android) | No (mod-level gate). | Excludes `auth` module on Android. |
| `mcp/sse_transport.rs` | android, not(android) | **Yes -- uses reqwest** | The reqwest usage is in the *non-gated* body. Only OAuth function is gated. |
| `mcp/stdio_proxy.rs` | android, ios | No. Returns errors on mobile. | Stdio transport stub on mobile platforms. |
| `ocr_adapters/system_ocr/mod.rs` | macos, windows | No. Calls native platform OCR. | |
| `page_rasterizer.rs` | windows, macos, linux | No. Uses `std::process::Command`. | DOCX-to-PDF conversion strategy per platform. |
| `pdfium_utils.rs` | android, macos, windows, linux | No. Uses `Pdfium::bind_to_library`. | Library loading path per platform. |
| `tts.rs` | windows, macos, linux | No. Uses `std::process::Command`. | Platform TTS (say/espeak). |
| `backup_common.rs` | windows, unix | No. Uses Win32 FFI, `std::process::Command`. | Disk space query per platform. |
| `backup_config.rs` | android, ios | No. Returns error on mobile. | `blocking_pick_folder` unavailable on mobile. |
| `data_space.rs` | unix | No. | |
| `crypto/mod.rs` | unix | No. | |
| `data_governance/backup/mod.rs` | unix | No. | |
| `data_governance/critical_audit_tests.rs` | unix | No. | |
| `mcp/global.rs` | unix | No. | |

### 3.3 Critical Finding

**NONE of the platform-gated code directly uses reqwest, hyper, or rustls.** These crates are imported and used uniformly across all platforms in non-gated code:

- `reqwest` is used in: `mcp/sse_transport.rs`, `mcp/http_transport.rs`, `mcp/rmcp.rs`, `mcp/global.rs`, `chat_v2/tools/*.rs`, `cmd/mcp.rs`, `commands.rs`, `cloud_storage/webdav.rs`, `voice_input.rs`, `streaming_anki_service.rs`, `llm_manager/mod.rs`, `utils/fetch.rs`, `tools/web_search.rs`
- `hyper` is used in: `metrics_server.rs`
- `sentry` (which bundles `rustls 0.21`) is initialized in `lib.rs` (non-gated)

**Conclusion**: Version consolidation of reqwest, hyper, or rustls will NOT break platform-gated code in any special way. The impact is uniform across all platforms.

---

## 4. Android-Specific Concerns

### 4.1 NDK Version & API Level

| Property | Value |
|----------|-------|
| NDK version | **27.0.12077973** |
| C linker target | `aarch64-linux-android21-clang` (API 21) |
| SDK build tools | 35.0.0 |
| SDK platform | android-35 |
| Target architecture | aarch64 (ARM64) |

Source: `.github/workflows/rebuild-android.yml` (line 146-147), `.cargo/config.toml` (line 34)

### 4.2 rustls/reqwest Android Compatibility

The project has already been carefully configured for Android cross-compilation:

| Crate | TLS Backend | Android Compatible? | Notes |
|-------|-------------|---------------------|-------|
| `reqwest` | `rustls-tls` (no native-tls) | **Yes** | No OpenSSL/root CA dependency. WebPKI roots bundled. |
| `tokio-tungstenite` | `rustls-tls-webpki-roots` | **Yes** | WebSocket over rustls. |
| `sentry` | `rustls` feature | **Yes** | Uses rustls 0.21. Avoids OpenSSL. |
| OAuth2 (`oauth2` crate) | native-tls only | **No** | Explicitly excluded on Android via `cfg(not(target_os = "android"))`. |

**Known Android build quirks for rustls/reqwest:**
- rustls 0.23.x has no Android-specific compilation issues — it's pure Rust with no native dependencies.
- reqwest 0.12.x with `rustls-tls` feature works cleanly on Android arm64.
- The NDK linker is configured (`aarch64-linux-android21-clang`) in `.cargo/config.toml`.
- **No OpenSSL issues** because the project deliberately avoids `native-tls` on all platforms.

### 4.3 Platform-Gated Dependency Exclusions on Android

| Dependency | Excluded on Android? | If removed, what breaks? |
|-----------|---------------------|--------------------------|
| `tauri-plugin-updater` | Yes (desktop-only) | Android auto-update (not needed -- uses APK download via JS) |
| `tauri-plugin-process` | Yes (desktop-only) | Exit/restart process (not relevant on Android) |
| `oauth2` + `pkce` | Yes | OAuth2 auth for MCP (falls back to API Key) |
| `cocoa` + `objc` | Yes (macOS-only) | No effect on Android |

---

## 5. Version Conflict Analysis

### 5.1 Multiple Versions in Cargo.lock

| Crate | Version 1 | Used By | Version 2 | Used By | Conflict? |
|-------|-----------|---------|-----------|---------|-----------|
| `reqwest` | **0.12.28** | Our project, sentry, vendored lancedb/object_store | **0.13.2** | Tauri 2.10.2, tauri-plugin-updater 2.10.0, tauri-plugin-http | **YES** -- semver-incompatible |
| `hyper` | **0.14.32** | Our project (`metrics_server.rs`) | **1.8.1** | Tauri, reqwest 0.13.2, vendored object_store | **YES** -- major version diff |
| `rustls` | **0.21.12** | Sentry 0.32.3 | **0.23.36** | reqwest 0.12/0.13, Tauri, tokio-tungstenite, tokio-rustls | **YES** -- semver-incompatible |
| `chrono` | **0.4.38** | Our project | **=0.4.41** | Vendored lancedb (exact pin) | **Minor** -- lancedb has own lockfile |
| `rand` | **0.8.5** | Our project | **0.9.2** | Vendored lancedb/object_store | **YES** -- semver-incompatible |

### 5.2 Tauri Internal Dep Conflict Risk

**Tauri 2.10.2 requires reqwest 0.13.2.** If our project upgrades from reqwest 0.12 to 0.13, we would align with Tauri and eliminate the duplicate. However:

- **Breaking change risk**: reqwest 0.13 removed `blocking` feature from default and changed some API surface. Our project uses `reqwest::blocking::Client` in `utils/fetch.rs`.
- **Vendored crates**: lancedb and object_store both pin reqwest 0.12.x. If we upgrade to 0.13, they would still carry 0.12 in their own dep trees (but they're vendored with separate lock files, so there's no actual resolution conflict).

**Our direct dep on hyper 0.14** is used only in `metrics_server.rs` for a lightweight HTTP metrics endpoint. Tauri and reqwest both use hyper 1.x. Upgrading to hyper 1.x would:
- Align with the rest of the dependency tree
- Remove one duplicate major version
- Require API migration in `metrics_server.rs` (hyper 1.x API changes)

**rustls duality**: Sentry 0.32.3 pins rustls 0.21.12 internally. Sentry 0.36+ uses rustls 0.23. Upgrading sentry would eliminate the rustls duplicate.

### 5.3 Consolidation Targets Summary

| Action | Impact | Difficulty |
|--------|--------|-----------|
| reqwest 0.12 -> 0.13 | Removes 1 duplicate. Breaks blocking API. Vendored crates unaffected (own lockfiles). | Medium |
| hyper 0.14 -> 1.x | Removes 1 duplicate. Requires `metrics_server.rs` migration. | Medium |
| sentry 0.32 -> 0.36 | Removes rustls 0.21 duplicate. | Low |
| All three combined | Removes ~40MB+ of duplicate compiled code, faster CI builds. | Medium-High |

---

## 6. Vendored Crate Analysis

### 6.1 Vendored lancedb (v0.22.1)

Source: `src-tauri/vendor/lancedb/Cargo.toml`

| Our Crate | Our Version | lancedb Requirement | Compatible? |
|-----------|-------------|-------------------|-------------|
| reqwest | 0.12.28 | 0.12.0 | **Yes** (same major) |
| hyper | 0.14.32 (ours) / 1.8.1 | 1.7.0 (via object_store dep) | **Yes** (in its own tree) |
| chrono | 0.4.38 | =0.4.41 (exact pin) | **Compatible** (4.x range, separate lockfile) |
| moka | 0.12.x | 0.12 | **Yes** |
| rand | 0.8.5 | 0.9 (optional) | **Incompatible** but optional and separate lockfile |
| object_store | (vendored separately) | 0.12.0 | N/A (vendored) |

### 6.2 Vendored object_store (v0.12.4)

Source: `src-tauri/vendor/object_store/Cargo.toml`

| Our Crate | Our Version | object_store Requirement | Compatible? |
|-----------|-------------|------------------------|-------------|
| reqwest | 0.12.28 | 0.12 | **Yes** |
| hyper | 0.14.32 (ours) / 1.8.1 | 1.2 | **Yes** (in its own tree) |
| chrono | 0.4.38 | 0.4.34 | **Yes** |
| rand | 0.8.5 | 0.9 | **Incompatible** but separate lockfile |
| base64 | 0.22.x | 0.22 | **Yes** |
| ring | 0.17.x | 0.17 | **Yes** |

### 6.3 Key Compat Observations

1. **reqwest**: Vendored crates use 0.12.x. If we consolidate upward to 0.13.x, the vendored deps still get 0.12.x from their own lockfiles. No conflict.
2. **hyper**: Both vendored crates use hyper 1.x, which matches Tauri's hyper 1.8.1. Our project's hyper 0.14 is the outlier.
3. **chrono**: Vendored lancedb pins =0.4.41 (exact). Our project uses 0.4.38. Since lancedb has its own Cargo.lock, this is fine. Cargo will build chrono twice if both 0.4.38 and 0.4.41 are needed, but typically 0.4.38 can be loosened to `0.4` to match.
4. **rand**: vendored crates use rand 0.9 (optional for lancedb, required for object_store cloud features). Our project uses rand 0.8. This is a semver difference managed by separate lockfiles.

---

## 7. Risk Matrix

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| reqwest 0.12->0.13 breaks blocking API | Medium | High | Change `utils/fetch.rs` to use async or enable blocking feature compat |
| hyper 0.14->1.x breaks metrics_server | High | Medium | Migrate `metrics_server.rs` to hyper 1.x API (service_fn changes) |
| sentry upgrade breaks rustls compat | Low | Low | Sentry 0.36 uses rustls 0.23 which is already in tree |
| Android NDK rustls build issue | Very Low | High | rustls is pure Rust; no NDK link issues; already proven in CI |
| Vendored crate lockfile skew | Low | Medium | Patch resolves; separate lockfiles prevent resolution conflicts |
| OAuth2 breaks if android cfg removed | Low | High | Keep `cfg(not(target_os = "android"))` guard in Cargo.toml |

---

## 8. Recommendations

1. **Upgrade sentry** from 0.32 to 0.36+ first to eliminate rustls 0.21 duplicate -- lowest risk.
2. **Upgrade hyper** from 0.14 to 1.x next -- only affects `metrics_server.rs` (~120 lines). Aligns with Tauri and vendored crates.
3. **Keep reqwest at 0.12** for now unless the vendored crates are also updated to require 0.13. The benefit of consolidating to 0.13 is marginal (two versions vs one, with vendored crates still on 0.12).
4. **Loosen chrono** in project Cargo.toml from `"0.4.38"` to `"0.4"` to match vendored lancedb's =0.4.41 pin more flexibly.
5. **No platform-gated code** requires modification -- the consolidation affects non-gated code uniformly.
6. **Android CI** will not regress from these changes since rustls is used everywhere (no native-tls).

---

## Appendix: File Paths Referenced

- `C:\deep-student\src-tauri\Cargo.toml` -- Main manifest
- `C:\deep-student\src-tauri\Cargo.lock` -- Resolved dependency tree
- `C:\deep-student\src-tauri\.cargo\config.toml` -- Linker/NDK config
- `C:\deep-student\src-tauri\vendor\lancedb\Cargo.toml` -- Vendored lancedb
- `C:\deep-student\src-tauri\vendor\object_store\Cargo.toml` -- Vendored object_store
- `C:\deep-student\src-tauri\src\metrics_server.rs` -- Only user of hyper 0.14
- `C:\deep-student\src-tauri\src\utils\fetch.rs` -- Uses reqwest::blocking
- `C:\deep-student\.github\workflows\rebuild-android.yml` -- Android CI (NDK 27, API 21)
- `C:\deep-student\.github\workflows\build-test.yml` -- CI for all platforms
- `C:\deep-student\scripts\build_android.sh` -- Local Android build script
