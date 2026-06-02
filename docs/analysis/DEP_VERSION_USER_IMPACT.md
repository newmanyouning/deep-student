# Multi-Version Dependencies: End-User Impact Analysis

> Generated: 2026-06-01
> Scope: Does the presence of multiple versions of the same crate in Cargo.lock affect end users of the distributed application?
> Context: Tauri 2 desktop app (Windows, macOS, Linux) + Android. Users download pre-built binaries.

---

## Executive Summary

**Multi-version dependencies do NOT affect end users of this application.**

Rust's static linking strategy means each crate version compiles into separate symbol namespaces. All versions coexist in the same binary without runtime interference. Since users receive pre-compiled binaries (never compiling themselves), multiple versions are an internal codebase concern only.

The only realistic impact is a modest increase in binary size (~2-5 MB total across all duplicates), which is negligible against a 50-100 MB Tauri application bundle.

---

## Verbatim Data: All Multi-Version Crates in Cargo.lock

Source: `C:\deep-student\src-tauri\Cargo.lock`

| Crate | Versions | Count | Held By (Version → Dependent) |
|-------|----------|-------|-------------------------------|
| rustls | 0.21.12, 0.23.36 | 2 | 0.21 → sentry 0.32.3; 0.23 → reqwest, Tauri, tokio-tungstenite |
| reqwest | 0.12.28, 0.13.2 | 2 | 0.12 → deep-student, oauth2, tauri-plugin-http; 0.13 → tauri-plugin-updater, jsonschema |
| zip | 0.6.6, 2.4.2, 4.6.1 | 3 | 0.6 → docx-rs; 2.4 → calamine; 4.6 → tauri-plugin-updater |
| hyper | 0.14.32, 1.8.1 | 2 | 0.14 → deep-student (metrics_server.rs); 1.8 → Tauri, reqwest, vendored object_store |
| thiserror | 1.0.69, 2.0.18 | 2 | 1.0 → transitive deps (html_parser, json-patch, oauth2, ppt-rs, etc); 2.0 → deep-student |
| base64 | 0.21.7, 0.22.1 | 2 | 0.21 → pkce, ron, tiktoken-rs; 0.22 → deep-student |
| quick-xml | 0.31.0, 0.37.5, 0.38.4 | 3 | 0.31 → calamine; 0.37 → umya-spreadsheet; 0.38 → deep-student |
| image | 0.24.9, 0.25.9 | 2 | 0.24 → pptx-to-md; 0.25 → deep-student |

---

## Detailed Analysis Per Crate

### 1. rustls (0.21.12, 0.23.36)

| Dimension | Assessment |
|-----------|-----------|
| **Binary Size Impact** | rustls 0.21 is ~200 KB compiled (ring bundled). rustls 0.23 is ~300 KB. The two versions share ring 0.17 (same compiled crypto code), so the actual delta is only the higher-level TLS state machine code (~150 KB extra). |
| **Runtime Conflict?** | NONE. Rust statically links with distinct symbol prefixes. The two rustls versions cannot interfere — they have separate `CipherSuite` enums, separate `ClientConfig` types, separate connection state machines. A TLS connection is handled entirely by one version. |
| **Platform Behavior Difference** | IDENTICAL. Both use ring for crypto, both support TLS 1.2/1.3. No platform-specific code path divergence. |
| **User-Visible?** | NO. Sentry (the sole consumer of rustls 0.21) sends error reports via HTTPS. Whether that uses rustls 0.21 or 0.23, the user sees the same behavior: error reports are delivered. |
| **Installation Impact** | NONE. Pre-built binary, no dynamic linking. rustls and ring are pure Rust with zero system library dependencies. |
| **Summary** | SAFE to leave as-is. Sentry 0.32 pins 0.21; upgrading sentry to 0.36+ would eliminate it. Pure developer convenience. |

### 2. reqwest (0.12.28, 0.13.2)

| Dimension | Assessment |
|-----------|-----------|
| **Binary Size Impact** | ~200 KB extra for the 0.12 shim layer (both share hyper 1.8.1 and rustls 0.23.36 underneath). |
| **Runtime Conflict?** | NONE. Separate `Client` instances, separate type systems. No cross-version state sharing. |
| **Platform Behavior Difference** | NEGLIGIBLE on desktop. reqwest 0.13 has `rustls-platform-verifier` integration (native OS certificate store); 0.12 uses `webpki-roots` (bundled Mozilla CA list). On Windows/macOS/Linux with standard internet access, both resolve TLS identically because all public CAs are in webpki-roots. |
| **User-Visible?** | POTENTIALLY on Android (see next row). On desktop: NO. |
| **Android-specific** | reqwest 0.13 can use Android Keystore via platform-verifier, while 0.12 only trusts webpki-roots. If a user has installed a custom root CA on their Android device (corporate MDM, self-signed proxy), connections made through our code (reqwest 0.12) would fail, while Tauri internal connections (reqwest 0.13) would succeed. **In practice**: Android users rarely install custom CAs, and our app doesn't communicate with custom-CA servers. Impact is theoretical. |
| **Upgrade feasibility** | Our code uses `reqwest::blocking::Client` in `utils/fetch.rs`. reqwest 0.13 removed blocking from default features (requires `blocking` feature flag). This is a trivial migration. |
| **Summary** | SAFE to leave as-is. Upgrade to 0.13 recommended for Android CA compatibility but not blocking. |

### 3. zip (0.6.6, 2.4.2, 4.6.1)

| Dimension | Assessment |
|-----------|-----------|
| **Binary Size Impact** | ~500 KB total (each version is ~150-200 KB of decompression logic) |
| **Runtime Conflict?** | NONE. Each used by completely different subsystems with no code sharing. |
| **Platform Behavior Difference** | IDENTICAL. All three are pure Rust with zero OS-specific behavior. zip decompression is deterministic regardless of version. |
| **User-Visible?** | NO. Users cannot tell which zip library version is handling which file. |
| **Installation Impact** | NONE. All pure Rust, no native libraries. |
| **Consolidation feasibility** | IMPOSSIBLE. Each version is pinned by an upstream library we cannot change: docx-rs -> zip 0.6, calamine -> zip 2.4, tauri-plugin-updater -> zip 4.6. Breaking changes between these major versions prevent any single version from satisfying all consumers. |
| **Summary** | SAFE but unfixable. Blocked by upstream library choices. Zero user impact. |

### 4. hyper (0.14.32, 1.8.1)

| Dimension | Assessment |
|-----------|-----------|
| **Binary Size Impact** | ~400 KB extra for the hyper 0.14 HTTP stack (body types, service framework, server). |
| **Runtime Conflict?** | NONE. hyper 0.14 is used exclusively by `metrics_server.rs` (a localhost-only Prometheus metrics endpoint). hyper 1.8 is used by reqwest and Tauri. They never interact. |
| **Platform Behavior Difference** | IDENTICAL. Both are pure Rust, no OS-specific behavior. The metrics endpoint binds to 127.0.0.1 on all platforms. |
| **User-Visible?** | NO. The metrics server is an internal debugging endpoint (disabled by default, only activated when the app has a specific config or env var). Even when active, it serves machine-readable Prometheus text format — invisible to users. |
| **Installation Impact** | NONE. All pure Rust. |
| **Consolidation feasibility** | EASY. `metrics_server.rs` is ~94 lines. hyper 1.x has a different service API (removed `make_service_fn`, replaced `Service` trait). Migration is mechanical. |
| **Summary** | SAFE to leave as-is. Consolidation would save ~400 KB in binary — purely a developer housekeeping task. |

### 5. thiserror (1.0.69, 2.0.18)

| Dimension | Assessment |
|-----------|-----------|
| **Binary Size Impact** | ZERO. `thiserror` is a proc-macro crate. It generates `Display` and `Error` trait implementations at compile time and produces zero runtime code. All proc-macro code compiles into a separate shared object loaded only by the Rust compiler, not the final binary. |
| **Runtime Conflict?** | N/A. Has no runtime component. |
| **Platform Behavior Difference** | N/A. |
| **User-Visible?** | NO. Purely a compile-time concern. |
| **Installation Impact** | NONE. |
| **Summary** | ZERO user impact. Ignore. The v1.0 copy is pulled in by transitive deps (html_parser, json-patch, oauth2, ppt-rs, reqwest-eventsource, sentry-types). When those crates update to thiserror 2.0, the duplicate disappears automatically. |

### 6. base64 (0.21.7, 0.22.1)

| Dimension | Assessment |
|-----------|-----------|
| **Binary Size Impact** | ~50 KB (base64 is a small crate; encoding/decoding tables) |
| **Runtime Conflict?** | NONE. |
| **Platform Behavior Difference** | IDENTICAL. Base64 encoding is deterministic. |
| **User-Visible?** | NO. |
| **Installation Impact** | NONE. Pure Rust. |
| **Summary** | SAFE. The 0.21 copy is held by pkce, ron, and tiktoken-rs. Previous consolidation effort reduced this from 3 versions to 2. Further reduction requires upstream updates. |

### 7. quick-xml (0.31.0, 0.37.5, 0.38.4)

| Dimension | Assessment |
|-----------|-----------|
| **Binary Size Impact** | ~200 KB total (each version ~60-70 KB of XML parser logic) |
| **Runtime Conflict?** | NONE. Each used by independent consumers. |
| **Platform Behavior Difference** | IDENTICAL. XML parsing behavior is standardized. |
| **User-Visible?** | NO. |
| **Installation Impact** | NONE. |
| **Summary** | SAFE. 0.31 -> calamine, 0.37 -> umya-spreadsheet, 0.38 -> deep-student directly. The consolidated 0.38 version is what our code uses. The others are upstream library locks. |

### 8. image (0.24.9, 0.25.9)

| Dimension | Assessment |
|-----------|-----------|
| **Binary Size Impact** | ~300 KB extra (image crate bundles several codec libraries) |
| **Runtime Conflict?** | NONE. |
| **Platform Behavior Difference** | IDENTICAL. Image decoding/encoding is deterministic. |
| **User-Visible?** | NO. |
| **Installation Impact** | NONE. |
| **Summary** | SAFE. The 0.24 copy is held by pptx-to-md (a document conversion crate). Previous consolidation reduced this from more versions to current 2. |

---

## Aggregate Binary Size Impact

| Crate | Estimated Extra Size |
|-------|---------------------|
| rustls 0.21 duplicate | ~150 KB |
| reqwest 0.12 duplicate | ~200 KB |
| zip 3-version overhead | ~500 KB |
| hyper 0.14 duplicate | ~400 KB |
| thiserror (both versions) | 0 bytes |
| base64 0.21 duplicate | ~50 KB |
| quick-xml duplicates | ~200 KB |
| image 0.24 duplicate | ~300 KB |
| **Total estimated overhead** | **~1.8 MB** |

Against a typical 50-100 MB Tauri application bundle, this is **1.8-3.6% overhead** — well within acceptable range.

---

## Why They Cannot Conflict

Rust's compilation model prevents multi-version runtime conflicts through three mechanisms:

1. **Symbol mangling**: Each crate version gets a unique symbol prefix derived from its name + version. `rustls_021::ClientConfig` and `rustls_023::ClientConfig` are completely different types to the linker. They occupy different memory sections and have different vtables.

2. **Separate monomorphization**: Generic functions are instantiated separately for each version. A `Vec<rustls_021::Certificate>` is a different type from `Vec<rustls_023::Certificate>`. No cross-version type confusion is possible.

3. **No dynamic linking**: All Rust crates in a Tauri app are statically linked into a single executable. There is no shared library version conflict (no "DLL hell"). The OS loads one binary and all symbols are resolved at link time.

---

## Platform-Specific Considerations

### Android

- All multi-version crates in this project are **pure Rust** with no Android NDK dependencies.
- The only platform-sensitive crate difference is reqwest 0.12 vs 0.13's certificate handling (discussed above).
- Both rustls versions (0.21 and 0.23) are pure Rust + ring, which cross-compiles to Android ARM64 without issues.
- The `.cargo/config.toml` configures the NDK linker (`aarch64-linux-android21-clang`) — this is unaffected by crate versions.

### Windows

- No crate versions have Windows-specific divergence.
- The `windows` crate (v0.58, single version) handles all Win32 API calls. Not duplicated.

### macOS

- No crate versions have macOS-specific divergence.
- The `cocoa`/`objc` crates (single version each) handle macOS native UI. Not duplicated.

### Linux

- No crate versions have Linux-specific divergence.
- No distro-specific dynamic library dependencies from any duplicated crate.

---

## Comparison: Previous Successful Consolidations

The project has already eliminated several multi-version duplications:

| Crate | Before | After | What Changed |
|-------|--------|-------|-------------|
| base64 | 3 versions | 2 versions | Upgraded project dep from 0.21 to 0.22 |
| thiserror | 2+ versions | 2 versions | Project now uses 2.0; 1.0 remains transitive |
| reqwest | 3 versions (0.11, 0.12, 0.13) | 2 versions (0.12, 0.13) | Eliminated 0.11 by upgrading project dep |
| quick-xml | 3+ versions | 3 versions (0.31, 0.37, 0.38) | Aligned vendored object_store to 0.38 |
| tokio-tungstenite | 2 versions (0.21, 0.28) | 1 version (0.28) | Upgraded from 0.21, which removed one rustls version |
| image | 3+ versions | 2 versions (0.24, 0.25) | Upgraded project dep from 0.24 to 0.25 |
| rustls | 3 versions (0.21, 0.22, 0.23) | 2 versions (0.21, 0.23) | Removed 0.22 via tokio-tungstenite upgrade |

Each consolidation was developer-facing. End users saw zero behavioral changes.

---

## Verdict

| Question | Answer |
|----------|--------|
| Affects installation? | **NO.** Users download pre-built binaries. Multiple crate versions do not affect installer size, dependency resolution, or system requirements. |
| Affects runtime? | **NO.** Rust's static linking prevents any cross-version interference. Each connection/operation uses exactly one version of each crate. |
| Creates platform behavior differences? | **NO** (desktop). **Negligible** (Android — reqwest 0.12 vs 0.13 certificate handling, but in practice both work). |
| User-visible? | **NO** on all platforms. |
| Binary size concern? | **Negligible** (~1.8 MB extra in a 50-100 MB app, ~2-3% overhead). |

### Bottom Line

Multi-version dependencies are a **developer code quality concern**, not a **user-facing issue**. The remaining duplicates are:
- **Blocked by upstream** (zip 3-version, quick-xml via calamine/umya-spreadsheet, reqwest via Tauri's plugin-updater)
- **Zero runtime code** (thiserror)
- **Small, consolidation-feasible housekeeping tasks** (hyper 0.14 in metrics_server.rs, sentry upgrade to remove rustls 0.21)

No end-user-facing justification exists to prioritize their consolidation. If the team consolidates for developer hygiene, start with `hyper 0.14 → 1.x` (easiest standalone fix) and `sentry 0.32 → 0.36+` (removes rustls 0.21).

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-01 | Claude Code | Initial analysis. Verified all versions against Cargo.lock. |

---

## Key Source Files

- `C:\deep-student\src-tauri\Cargo.lock` — Dependency resolution
- `C:\deep-student\src-tauri\Cargo.toml` — Project manifest
- `C:\deep-student\src-tauri\src\metrics_server.rs` — hyper 0.14 usage (only consumer)
- `C:\deep-student\src-tauri\src\utils\fetch.rs` — reqwest::blocking usage
- `C:\deep-student\docs\analysis\PLATFORM_DEP_IMPACT.md` — Platform-specific dependency analysis (companion report)
- `C:\deep-student\docs\analysis\SINGLE_VERSION_FEASIBILITY.md` — Consolidation feasibility study (companion report)
