# Dependency Version Bloat — Root Cause Analysis

> Generated: 2026-06-01
> Scope: `C:\deep-student\src-tauri` (Cargo.toml + Cargo.lock)
> Total unique packages: 992
> Multi-version packages: **125** (12.6% of all deps)

---

## Executive Summary

The `deep-student` Rust workspace has **125 packages appearing at 2+ versions**, driven by **three primary root causes**:

1. **Two vendored crates** (`lancedb` 0.22.1 and `object_store` 0.12.4) — these are the dominant source, pulling in a completely separate dependency ecosystem (reqwest 0.12, hyper 1.x, base64 0.22, rand 0.9, thiserror 2.0, etc.) that conflicts with the project's own choices.
2. **oauth2 crate** — locks the project into reqwest 0.11 / hyper 0.14 / http 0.2 / base64 0.13, preventing upgrades.
3. **tauri-utils legacy HTML stack** — `kuchikiki` -> `selectors` -> `phf 0.8` -> `rand 0.7`, an ancient CSS parser chain that forces four separate `phf` versions and `rand 0.7`.

---

## Root Cause A: Vendored Crate Analysis

### A.1 Why Vendored?

The `[patch.crates-io]` section in `Cargo.toml` (lines 246-248) replaces published crates with local copies:

```toml
[patch.crates-io]
lancedb = { path = "vendor/lancedb" }
object_store = { path = "vendor/object_store" }
```

Both were introduced in the initial commit (e8749d4d, `Initial release: DeepStudent v0.9.2`). There is no README or comment explaining WHY they are vendored. Likely reasons:

- **lancedb 0.22.1** was never published to crates.io at the time of development (the published version lagged).
- **object_store 0.12.4** was vendored because `lancedb` depends on it with `version = "0.12.0"`, and the project needed to prevent an unintended upgrade to a newer minor version that might break.
- Both `.cargo-ok` files exist, confirming they are proper vendored cargo sources.

### A.2 Vendored Version Pins

| Crate | Vendor Version | Key Dependencies Pinned |
|-------|---------------|------------------------|
| **lancedb** | 0.22.1 | chrono `=0.4.41`, lance `=0.37.0`, reqwest `0.12.0`, object_store `0.12.0`, datafusion `49.0`, rand `0.9`, http `1` |
| **object_store** | 0.12.4 | reqwest `0.12`, hyper `1.2`, base64 `0.22`, thiserror `2.0`, rand `0.9`, quick-xml `0.38`, http `1.2` |

### A.3 How Vendored Crates Cascade Bloat

```
lancedb (vendor)
  └── lance =0.37.0
        ├── lance-core -> object_store (vendor) [reqwest 0.12, hyper 1, base64 0.22, thiserror 2.0, quick-xml 0.38, rand 0.9]
        ├── lance-io -> object_store (vendor) [same]
        ├── lance-index -> tantivy [thiserror 2.0]
        └── datafusion 49.0
              ├── datafusion-common [base64 0.22, dashmap 6.1]
              ├── datafusion-catalog [dashmap 6.1]
              └── datafusion-execution [dashmap 6.1, rand 0.9]
```

The vendored `object_store` is particularly impactful because it sits at the root of the cloud-storage abstraction for the entire lance/datafusion ecosystem, forcing:

- **reqwest 0.12** (vs project's 0.11) -> requires hyper 1.x -> requires h2 0.4 -> requires http 1.x
- **base64 0.22** (vs project's 0.21.7 and oauth2's 0.13.1)
- **thiserror 2.0** (vs project's 1.0)
- **rand 0.9** (vs project's 0.8)

### A.4 Is Vendoring Still Necessary?

**Possibly not for object_store**. The published `object_store` on crates.io is at 0.12.x+ and the vendored version is 0.12.4. If the published version is compatible, the patch could be removed. However, there may have been breaking changes in later 0.12.x patches.

**For lancedb**: The published version on crates.io may still lag behind 0.22.1, or the project may have local patches. A check of `cargo search lancedb` or the repo would confirm. The `.cargo_vcs_info.json` file suggests it was cloned from git, not crates.io.

---

## Root Cause B: oauth2 Crate — The Legacy HTTP Stack Anchor

The `oauth2 = "4.4"` dependency (line 148) is the **single largest blocker** for upgrading the HTTP stack:

| Dependency | Version locked by oauth2 | Project's desired version |
|-----------|--------------------------|--------------------------|
| reqwest | 0.11 | 0.12+ |
| hyper | 0.14 (via reqwest 0.11) | 1.x |
| hyper-rustls | 0.24 (via reqwest 0.11) | 0.27+ |
| http | 0.2 (via hyper 0.14) | 1.x |
| h2 | 0.3 (via hyper 0.14) | 0.4 |
| base64 | 0.13 | 0.22 |
| tokio-rustls | 0.24 (via hyper-rustls 0.24) | 0.26 |

**Chain:** `deep-student` -> `oauth2 4.4` -> `reqwest 0.11` -> `hyper 0.14 + hyper-rustls 0.24` -> `http 0.2 + h2 0.3 + tokio-rustls 0.24`

Since oauth2 is a **direct** dependency, upgrading it to a version that supports reqwest 0.12 (oauth2 >= 5.0?) would eliminate this entire bloat category.

---

## Root Cause C: tauri-utils Legacy HTML/CSS Parser Stack

`tauri-utils` uses `kuchikiki` (a fork of the kuchiki HTML parser), which depends on `selectors 0.24`, which depends on `phf 0.8`. This pulls in:

```
tauri-utils 2.8.2
  └── kuchikiki 0.8.8-speedreader
        ├── html5ever 0.29 (also: html5ever 0.38 from markup5ever 0.38)
        ├── selectors 0.24
        │     ├── phf 0.8
        │     │     └── phf_generator 0.8 -> rand 0.7
        │     └── phf_codegen 0.8
        └── markup5ever 0.14 (also: markup5ever 0.38)
```

This forces **5 versions of `phf`** (0.8, 0.10, 0.11, 0.12, 0.13), **3 versions of `phf_generator`**, **6 versions of `windows-sys`**, and **rand 0.7**. The root cause is that `tauri-utils` has not migrated away from `kuchikiki` (an unmaintained fork) to a modern HTML parser.

---

## Detailed Root Cause Mapping

### C1: rustls (3 versions: 0.21, 0.22, 0.23)

| Version | Pullers | Chain | Fixable? |
|---------|---------|-------|----------|
| **0.21.12** | `reqwest 0.11` (direct), `sentry` (direct) | oauth2 + direct dep -> reqwest 0.11 -> hyper-rustls 0.24 -> rustls 0.21 | Yes — upgrade reqwest to 0.12+ and sentry to use rustls 0.23 |
| **0.22.4** | `tokio-tungstenite 0.21` (direct), `tungstenite 0.21` (direct) | tokio-tungstenite -> tokio-rustls 0.25 -> rustls 0.22 | Yes — upgrade tokio-tungstenite to 0.28+ |
| **0.23.36** | `reqwest 0.12`/`0.13`, `aws-sdk`, `jsonschema`, `tauri-plugin-updater` | Modern ecosystem, this is the target | Keep |

**Security note:** rustls 0.21 and 0.22 are unmaintained. CVE fixes only land in 0.23.

### C2: reqwest (3 versions: 0.11, 0.12, 0.13)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.11.27** | Project direct dep, `oauth2 4.4`, `reqwest-eventsource 0.5` | Upgrade project dep to 0.12; requires oauth2 upgrade |
| **0.12.28** | `sentry 0.32`, `tauri-plugin-http 2` | Already at 0.12 — compatibility dep |
| **0.13.2** | `tauri 2.10`, `tauri-plugin-updater 2`, `jsonschema 0.42` | Tauri is at 0.13 — this is the target |

**Recommendation:** Upgrade project from `reqwest 0.11` to `0.12` (or `0.13` if tauri allows). This requires upgrading oauth2 and possibly reqwest-eventsource.

### C3: zip (3 versions: 0.6, 2.4, 4.6)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.6.6** | Project direct dep, `docx-rs 0.4`, `ppt-rs 0.2` | Upgrade zip in Cargo.toml from `0.6` to `2.x`; check if docx-rs/ppt-rs accept newer zip |
| **2.4.2** | `calamine 0.26`, `pptx-to-md 0.2`, `umya-spreadsheet 2.3` | Cannot change — transitive |
| **4.6.1** | `tauri-plugin-updater 2` | Cannot change — tauri dictates this |

**Feasibility:** Low effort to upgrade zip 0.6 to 2.x — the API is similar. docx-rs and ppt-rs would need checking.

### C4: hyper (2 versions: 0.14, 1.x)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.14.32** | Project direct dep (line 78), `reqwest 0.11` via `hyper-rustls 0.24` | Upgrade project to use hyper 1.x directly (available since 1.0); upgrade reqwest to 0.12 |
| **1.8.1** | `aws-sdk`, `reqwest 0.12`/`0.13`, `mockito` | This is the target |

**Note:** The project directly depends on `hyper = { version = "0.14", features = ["full"] }` for "lightweight built-in metrics service." This can be upgraded to hyper 1.x directly.

### C5: thiserror (2 versions: 1.0, 2.0)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **1.0.69** | Project direct dep (line 41), `oauth2 4.4`, most of `tauri-*` ecosystem | Upgrade project from `1.0` to `2.0` |
| **2.0.18** | `object_store` (vendored), `datafusion`, `lance`, `tauri 2.10+` | This is the target |

**Note:** The project pins `thiserror = "1.0"` directly. Upgrading to 2.0 is straightforward (it's a proc-macro crate with near-perfect backward compatibility). However, transitive deps pulling 1.0 (oauth2, some tauri sub-crates, tungstenite 0.21) would still keep 1.0 around until those are upgraded too.

### C6: base64 (3 versions: 0.13, 0.21, 0.22)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.13.1** | `oauth2 4.4` | Upgrade oauth2 |
| **0.21.7** | Project direct dep (line 37), `reqwest 0.11`, `tiktoken-rs`, `config` | Upgrade project to 0.22; upgrade reqwest |
| **0.22.1** | `object_store` (vendored), `arrow-cast`, `datafusion-common`, `docx-rs`, `reqwest 0.12`/`0.13`, tauri tooling | This is the target |

### C7: rand (3 versions: 0.7, 0.8, 0.9)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.7.3** | `phf_generator 0.8` -> `selectors 0.24` -> `kuchikiki` -> `tauri-utils` | Requires tauri-utils to drop kuchikiki |
| **0.8.5** | Project direct dep (line 49), most of ecosystem | Upgrade project to 0.9 |
| **0.9.2** | `object_store`, `lancedb`/`datafusion`, `aws-sdk` | This is the target |

### C8: tokio-rustls (3 versions: 0.24, 0.25, 0.26)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.24.1** | `reqwest 0.11` -> `hyper-rustls 0.24` | Upgrade reqwest |
| **0.25.0** | `tokio-tungstenite 0.21` (direct) | Upgrade tokio-tungstenite |
| **0.26.4** | `reqwest 0.12`/`0.13`, `aws-sdk` | Keep |

### C9: h2 (2 versions: 0.3, 0.4)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.3.27** | `hyper 0.14` (from reqwest 0.11 chain) | Upgrade reqwest |
| **0.4.13** | `hyper 1.x` ecosystem | Keep |

### C10: http (2 versions: 0.2, 1.x)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.2.12** | `hyper 0.14`, `reqwest 0.11`, `oauth2`, `aws-config` (old aws-smithy-http 0.60) | Upgrade reqwest; aws-smithy-http 0.60 vs 0.62 version gap |
| **1.4.0** | Modern ecosystem (tauri, reqwest 0.12+, aws-smithy-http-client 1.x) | Keep |

### C11: aws-smithy-http (2 versions: 0.60, 0.62)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.60.12** | `aws-config 1.5` | aws-config itself pins an older aws-smithy-http than aws-sdk-s3 |
| **0.62.5** | `aws-sdk-s3`, `aws-runtime`, `aws-sigv4` | This is the target |

This is an **internal version gap within the AWS SDK ecosystem** — aws-config 1.5.16 pulls aws-smithy-http 0.60 while aws-sdk-s3 1.111 pulls 0.62. This duplicates `http 0.2` usage since 0.60 still uses http 0.2. Upgrading `aws-config` to a newer minor might resolve this.

### C12: phf (5 versions: 0.8, 0.10, 0.11, 0.12, 0.13)

Five versions of `phf` coexist because:
- `phf 0.8`: `selectors 0.24` (from `kuchikiki` in `tauri-utils`)
- `phf 0.10`: `phf_macros 0.10` (from `string_cache` older stack)
- `phf 0.11`: `string_cache_codegen 0.5` (from `string_cache` 0.8)
- `phf 0.12`: Unused in main deps? Check if dead weight.
- `phf 0.13`: `string_cache_codegen 0.6` + `string_cache 0.9` (modern stack)

All five could collapse to one if `tauri-utils` dropped the `kuchikiki`/`selectors 0.24` dependency and modernized `markup5ever`.

### C13: image (2 versions: 0.24, 0.25)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.24.9** | Project direct dep (line 63), `pdfium-render` (pins image_024 feature) | Upgrade project to 0.25; check pdfium-render for image_025 feature |
| **0.25.9** | `arboard` -> `tauri-plugin-clipboard-manager`, `docx-rs` | Cannot change |

### C14: quick-xml (3 versions: 0.31, 0.37, 0.38)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.31.0** | `calamine 0.26` | Cannot change — calamine pins 0.31 |
| **0.37.5** | Project direct dep (line 90) | Upgrade project to 0.38 |
| **0.38.4** | vendored `object_store`, `plist`, `wayland-scanner` | Keep |

### C15: tokio-tungstenite / tungstenite (0.21, 0.28)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.21.0** | Project direct dep (line 103-104) | Upgrade project to 0.28 |
| **0.28.0** | `tauri-plugin-mcp-bridge 0.8` (optional, feature-gated) | Already the target but only used with `mcp-debug` feature |

### C16: zstd (0.11, 0.13)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.11.2+zstd.1.5.2** | `zip 0.6` (project direct dep) | Upgrade zip to 2.x (which uses zstd 0.13) |
| **0.13.3** | Project direct dep (line 137) | Keep |

### C17: dashmap (5.5, 6.1)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **5.5.3** | Project direct dep (line 96) | Upgrade project to 6.x |
| **6.1.0** | `datafusion` ecosystem (through vendored lancedb) | Keep |

### C18: windows (0.58, 0.61)

| Version | Pulled by | Fix |
|---------|-----------|-----|
| **0.58.0** | Project direct dep (line 186) — Windows OCR API | Upgrade to 0.61 if API-compatible |
| **0.61.3** | `tao` -> `tauri-runtime-wry` | Cannot change |

---

## Upgrade Feasibility Matrix

| Crate | Current (specified) | Target Version | Primary Blocker | Effort | Priority |
|-------|--------------------|-------------- |-----------------|--------|----------|
| **reqwest** | 0.11 | 0.13 | oauth2 4.4 pins 0.11 | Medium | P0 |
| **hyper** | 0.14 | 1.x | Metrics service code uses 0.14 API | Medium | P0 |
| **zip** | 0.6 | 2.x | docx-rs, ppt-rs also use 0.6 | Low | P1 |
| **thiserror** | 1.0 | 2.0 | oauth2, tungstenite 0.21 pin 1.0 | Low | P1 |
| **base64** | 0.21.7 | 0.22 | oauth2 pins 0.13; some libs on 0.21 | Medium | P1 |
| **rustls** | (transitive) | 0.23 only | reqwest 0.11 + tokio-tungstenite 0.21 | Medium (covered by reqwest/oauth2 upgrades) | **P0 (security)** |
| **rand** | 0.8 | 0.9 | phf 0.8 -> selectors -> kuchikiki pins 0.7 | Hard | P2 |
| **image** | 0.24 | 0.25 | pdfium-render pins image_024 | Low | P2 |
| **quick-xml** | 0.37 | 0.38 | calamine pins 0.31 (unfixable) | Low | P2 |
| **tokio-tungstenite** | 0.21 | 0.28 | Just a direct dep upgrade | Low | P1 |
| **oauth2** | 4.4 | 5.x? | Upstream compatibility + API changes | Medium | **P0** |
| **dashmap** | 5.5.3 | 6.x | API compatible? | Low | P2 |
| **windows** | 0.58 | 0.61 | OCR API compat | Low | P2 |

---

## Quick Wins (Low Effort, High Impact)

1. **Upgrade `thiserror` from 1.0 to 2.0** (line 41 of Cargo.toml). The crate's own code uses 2.0-style derives already (thiserror 2.0 has been stable for over a year). This won't remove the duplicate entirely (tauri-ecosystem sub-crates still depend on 1.0) but aligns the project. Estimated effort: 15 min.

2. **Upgrade `zip` from 0.6 to 2.x** (line 60). The `zip` 2.x API is largely compatible with 0.6. Check `docx-rs` and `ppt-rs` for compatibility. If they accept zip 2.x, this removes the old zstd 0.11 chain. Estimated effort: 1-2 hours.

3. **Upgrade `quick-xml` from 0.37 to 0.38** (line 90). API is near-identical. Estimated effort: 30 min.

4. **Upgrade `tokio-tungstenite` from 0.21 to 0.28** (line 103). This removes rustls 0.22 and tokio-rustls 0.25. Estimated effort: 1 hour (API changes in tungstenite 0.28).

5. **Upgrade `base64` from 0.21 to 0.22** (line 37). The 0.22 API changed from `encode()`/`decode()` to `Engine` trait, but `use base64::Engine;` and `engine::general_purpose::STANDARD.encode()` is the new pattern. Estimated effort: 2-3 hours across the codebase.

6. **Upgrade `image` from 0.24 to 0.25** (line 63). Change pdfium-render feature from `image_024` to `image_025`. Estimated effort: 30 min.

---

## Medium-Term Fixes

7. **Replace or fork `oauth2`**: This is the single highest-impact change. oauth2 4.4 is stuck on reqwest 0.11. oauth2 5.x (if available) or a direct fork with updated deps would collapse 5+ duplicate crate pairs. Estimated effort: 4-8 hours (depends on API breakage).

8. **Upgrade `reqwest` to 0.12** across the project. After fixing oauth2, this is straightforward. The HTTP API is stable between 0.11 and 0.12. Estimated effort: 2-4 hours.

9. **Upgrade `hyper` from 0.14 to 1.x**: The project's metrics service (which uses hyper 0.14 directly) needs an API migration. hyper 1.x changed significantly from 0.14 (the `Service` trait, body types). Estimated effort: 4-8 hours.

---

## Hard Problems (High Effort, Lower Impact)

10. **tauri-utils `kuchikiki` dependency**: This requires Tauri upstream to migrate away from the unmaintained `kuchikiki`/`selectors 0.24` stack. Not actionable without a Tauri PR. This single chain forces phf 0.8 (and rand 0.7)

11. **Removing vendored crates**: The `lancedb` and `object_store` vendored copies are the largest source of bloat. To remove the patches:
    - `object_store` 0.12.x published on crates.io may be compatible — test by removing the patch first
    - `lancedb` needs checking on crates.io; if 0.22.1+ is published, the patch can be removed
    - Even if removed, the upstream lancedb/object_store naturally require reqwest 0.12, hyper 1, etc., so the version gaps for HTTP-related crates would persist

12. **`calamine` pins `quick-xml 0.31` and `zip 2.4`**: Cannot fix without upstream PRs.

---

## Summary: Can We Reach 0 Multi-Version?

**No, not completely.** Approximately 85% of the 125 duplicates could be eliminated with reasonable effort, but some version splits are inherent:

- **PHF/five versions** -> down to 2 (requires tauri-utils upstream fix)
- **windows-sys/six versions** -> inherent platform matrix, not fixable
- **calamine's quick-xml 0.31** -> unfixable without calamine upgrade
- **aws-smithy-http 0.60 vs 0.62** -> may persist due to aws-config internal versioning

**Realistic target**: ~35 multi-version packages (down from 125) by executing the quick wins and oauth2 upgrade.

---

## Appendix: All 125 Multi-Version Packages by Root Cause Category

| Root Cause | # of multi-version pkgs | Examples |
|-----------|----------------------|----------|
| **oauth2 lock-in** (reqwest 0.11 / hyper 0.14) | ~15 | rustls 0.21, reqwest 0.11, hyper 0.14, h2 0.3, http 0.2, hyper-rustls 0.24, tokio-rustls 0.24, base64 0.13 |
| **tauri-utils legacy HTML** (kuchikiki) | ~12 | phf 0.8/0.10/0.11/0.12/0.13, phf_generator 0.8/0.10/0.11/0.13, rand 0.7, html5ever 0.29/0.38, markup5ever 0.14/0.38 |
| **Vendored lancedb+object_store** | ~20 | dashmap 6.1, rand 0.9, base64 0.22, thiserror 2.0, http 1.x, quick-xml 0.38, serde_spanned 1.0, toml 0.9, toml_edit 0.23 |
| **AWS SDK version gaps** | ~5 | aws-smithy-http 0.60/0.62, http 0.2 persistence |
| **Format libraries** (docx-rs, calamine, etc.) | ~8 | zip 0.6/2.4, image 0.25, zstd 0.11, quick-xml 0.31 |
| **Platform matrix** (windows-sys family) | ~12 | windows-targets 4 archs x 4 versions = inherent |
| **tauri-plugin-updater** | ~5 | zip 4.6, rustls 0.23, reqwest 0.13 |
| **Cargo build-tooling** (toml_edit, proc-macro-crate) | ~4 | toml_edit 0.19/0.20/0.22/0.23 |
| **Other minor transitive** | ~44 | ahash, bit-set, bit-vec, etc. (usually harmless) |
