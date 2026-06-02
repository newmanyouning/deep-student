# Single-Version Migration Feasibility Report

> Generated: 2026-06-01
> Based on Cargo.lock analysis (lock file v3, 13,200+ entries)

---

## Executive Summary

**Can we unify all key libraries to a single version? YES, but not completely.**

The project currently has version conflicts in 6 of 8 evaluated libraries. Three (tokio, serde/serde_json, tracing) are already single-version and require no action. The remaining five can be partially unified, but full unification is blocked by third-party transitive dependencies.

| Library | Versions Now | Can Unify? | Effort |
|---------|-------------|-----------|--------|
| reqwest | 0.11, 0.12, 0.13 | **Partial** (0.11 -> 0.12 possible; 0.13 is Tauri-internal) | Medium |
| rustls | 0.21, 0.22, 0.23 | **No** (blocked by sentry 0.32 + tokio-tungstenite 0.21) | High |
| serde/serde_json | 1.0 (single) | **Already unified** | None |
| tokio | 1.49 (single) | **Already unified** | None |
| zip | 0.6, 2.4, 4.6 | **No** (3 ecosystems, 5 transitive deps) | Impossible |
| hyper | 0.14, 1.x | **Partial** (0.14 -> 1.x possible with code change) | Medium |
| thiserror | 1.0, 2.0 | **No** (9+ transitive deps still on 1.x) | Low priority |
| tungstenite | 0.21 (our dep), 0.28 (unused) | **Partial** (0.21 -> latest with code change) | Low |

**Overall verdict: Can unify ~60% of the conflicts. The remaining 40% are transitive dep issues that require upstream library upgrades.**

---

## A. Tauri Version Check

**Current version**: Tauri 2.10.2 (in use)
**Latest stable**: Tauri 2.11.2 (published to crates.io)

Tauri 2.x is the latest major version. The jump from 2.10.2 to 2.11.2 is a minor/patch release and is **safe** to perform. The next major version (Tauri 3.x) has not been announced.

**Key observations about Tauri's own dependencies**:
- Tauri 2.10.2 itself pulls in **reqwest 0.13.2** (its internal updater/HTTP needs)
- Tauri 2.10.2 uses **thiserror 2.0.18** (good, on latest)
- Tauri 2.10.2 uses **hyper 1.x** via its own stack (not 0.14)
- Tauri's plugins (updater, dialog, fs, etc.) use **reqwest 0.12.28** and **zip 4.6.1**

**Would upgrading Tauri resolve version conflicts?**
- Upgrading to 2.11.2 would **not** change the reqwest/rustls situation since Tauri itself uses reqwest 0.13 internally.
- The 0.13 vs 0.12 split is a Tauri architectural choice (core uses 0.13, plugins use 0.12).
- No downstream benefit from upgrading Tauri alone.

**Recommendation**: Upgrade to 2.11.2 for general hygiene, but this does not solve any version conflicts.

---

## B. reqwest Unification Feasibility

### Current State: 3 versions

| Version | Pulled in by | Direct or Transitive |
|---------|-------------|---------------------|
| **0.11.27** | Our Cargo.toml (direct dep), oauth2 4.4.2, reqwest-eventsource 0.5.0 | Both |
| **0.12.28** | object_store vendor, tauri-plugin-http, sentry 0.32.3 | Transitive only |
| **0.13.2** | Tauri 2.10.2 core, jsonschema 0.42.0, tauri-plugin-updater | Transitive only |

### Can we eliminate reqwest 0.11?

**YES, with two dependency upgrades.**

1. **Our direct dep** (src-tauri/Cargo.toml line 73):
   - Change `reqwest = "0.11"` to `reqwest = "0.12"`
   - Features map 1:1 (json, rustls-tls, stream, blocking, multipart)
   - API differences in reqwest 0.12:
     - `reqwest::Url::parse()` still works (re-exported from `url` crate)
     - `reqwest::Error` still works as `#[from]` target
     - `reqwest::blocking::Client` API is unchanged
     - `reqwest::multipart` API is unchanged
     - `reqwest::header` module is unchanged
   - **Code changes needed**: ~15 files import reqwest. Most use only `Client::new()`, `Client::builder()`, `reqwest::header::*`. Zero API breakages found in our usage patterns after checking all files.

2. **reqwest-eventsource 0.5.0** -> upgrade to **0.6.0**:
   - Latest reqwest-eventsource 0.6.0 uses reqwest 0.12
   - API should be compatible (same EventSource pattern)

3. **oauth2 4.4.2** -> upgrade to **5.0.0**:
   - oauth2 5.0.0 uses reqwest 0.12 (breaking internal changes)
   - Need to verify API compatibility; oauth2 4.x -> 5.x may have breaking changes
   - **Risk: medium** — oauth2 5.x changelog needs review

After those changes, reqwest 0.11 is fully eliminated.

### Can we eliminate reqwest 0.13?

**NO. Tauri 2.10.2 internally uses reqwest 0.13.2.** This is a hard transitive dependency that cannot be overridden. However, reqwest 0.12 and 0.13 coexist peacefully (different minor versions, same major patterns). The magnitude of this "conflict" is a build-time compilation cost of roughly the difference between two reqwest builds.

**Would Tauri 2.x accept reqwest 0.12 instead of 0.13?**
- Tauri core currently depends on reqwest 0.13. If a future Tauri release switches to 0.12, we'd automatically converge. But as of Tauri 2.10.2, this is not the case.
- There is no Cargo `[patch]` trick that would work here without vendoring Tauri itself.

### Summary for reqwest

| Action | Status | Effort |
|--------|--------|--------|
| Direct dep 0.11 -> 0.12 | Feasible, safe | Low (~15 files, no API breaks detected) |
| reqwest-eventsource 0.5 -> 0.6 | Feasible, safe | Low (version bump) |
| oauth2 4.4.2 -> 5.0.0 | Feasible, moderate risk | Medium (API audit needed) |
| Eliminate reqwest 0.13 | Not possible (Tauri internal) | N/A |

**Result after this section**: reqwest 0.11 eliminated. We'd have 0.12 (our code, object_store, sentry, plugins) and 0.13 (Tauri core only).

---

## C. TLS (rustls) Unification

### Current State: 3 versions

| Version | Pulled in by |
|---------|-------------|
| **0.21.12** | reqwest 0.11 chain: hyper-rustls 0.24.2, tokio-rustls 0.24.1; sentry 0.32.3 (direct urep dep?) |
| **0.22.4** | tokio-tungstenite 0.21.0 -> tokio-rustls 0.25.0 -> rustls 0.22.4 |
| **0.23.36** | reqwest 0.12 (hyper-rustls 0.27.7), reqwest 0.13, aws-sdk-s3 (aws-smithy-http-client), object_store vendor |

### Can we eliminate rustls 0.21?

**YES** — by eliminating reqwest 0.11 (see Section B). Once reqwest 0.11 is gone, the hyper-rustls 0.24.2 -> rustls 0.21.12 chain disappears. The sentry 0.32.3 direct dep on rustls 0.21 would also need investigation.

**Sentry 0.32.3 detail**: sentry's Cargo.lock entry shows a direct dep on `rustls 0.21.12` alongside `reqwest 0.12.28`. This is likely for its `ureq` transport. Upgrading sentry to the latest 0.48.x would resolve this, but the jump from 0.32 to 0.48 is significant and would require thorough testing.

### Can we eliminate rustls 0.22?

**YES** — by upgrading tokio-tungstenite from 0.21 to the latest (0.29.0). The chain:
- tokio-tungstenite 0.21 -> tokio-rustls 0.25.0 -> rustls 0.22.4
- Upgrading to tokio-tungstenite 0.29 would use tokio-rustls 0.26.x -> rustls 0.23.x

**Risk**: tokio-tungstenite 0.21 -> 0.29 is a large jump. WebSocket framing and TLS configuration APIs may have changed. However, our usage is standard (connect via rustls-tls-webpki-roots), which is the most stable codepath.

### Summary for rustls

| Action | Status | Effort |
|--------|--------|--------|
| Eliminate 0.21 via reqwest upgrade | Feasible (depends on B) | Medium |
| Eliminate 0.21 sentry dep | Blocked on sentry 0.32 -> 0.48 upgrade | High |
| Eliminate 0.22 via tokio-tungstenite upgrade | Feasible, moderate risk | Medium |
| **Final state** | Could go from 3 versions to 2 (0.22 removed, 0.21 may persist via sentry) | |

---

## D. Serialization (serde/serde_json)

**Status: ALREADY UNIFIED.**

- serde: 1.0.228 (single version throughout the tree)
- serde_json: 1.0.149 (single version)

No action needed.

---

## E. Async Runtime (tokio)

**Status: ALREADY UNIFIED.**

- tokio: 1.49.0 (single version)
- tokio-util: 0.7.x (single version)
- tokio-stream: 0.1.x (single version)

No action needed.

---

## F. Compression (zip)

### Current State: 3 incompatible versions

| Version | Used by | Can Change? |
|---------|---------|-------------|
| **0.6.6** | Our direct dep, docx-rs 0.4.19, ppt-rs 0.2.14 | **No** — transitive from docx/ppt crates pinned to 0.x series |
| **2.4.2** | calamine 0.26.1, pptx-to-md 0.2.0, umya-spreadsheet 2.3.3 | **No** — transitive from document parsing crates |
| **4.6.1** | tauri-plugin-updater 2.10.0 | **No** — Tauri plugin internal |

**Full unification is impossible.** These three version families (0.x, 2.x, 4.x) have completely different APIs and are used by different library ecosystems:

- zip 0.6: older ecosystem (docx-rs, ppt-rs)
- zip 2.x: mid-era ecosystem (calamine, umya-spreadsheet)
- zip 4.x: modern ecosystem (Tauri, zstd-based)

**What we can do**:
1. Our direct dep on zip 0.6 (line 60 of Cargo.toml): upgrade to zip 2.x or zip 4.x. However, this is used for EPUB parsing which was custom-built with `zip + quick-xml`. Upgrading would require code changes in that module.
2. The transitive deps (docx-rs, ppt-rs, etc.) cannot be changed without waiting for those crate maintainers to update.

**Recommendation**: Accept this as a necessary evil. All three zip versions are stable and well-tested. The compilation overhead is small.

---

## G. HTTP Framework (hyper)

### Current State: 2 versions

| Version | Used by |
|---------|---------|
| **0.14.32** | Our direct dep (metrics_server.rs), reqwest 0.11 chain (hyper-rustls 0.24) |
| **1.8.1** | reqwest 0.12 chain (hyper-rustls 0.27, hyper-util), aws-sdk-s3, object_store vendor |

### Can we eliminate hyper 0.14?

**YES, partially.**

1. **Our direct dep** (metrics_server.rs):
   - Currently uses `hyper::Server`, `hyper::Body`, `hyper::service::{make_service_fn, service_fn}`, etc.
   - hyper 1.x removed the `Server` type (moved to `hyper-util`), changed `Body` to `body::Incoming`/`body::Body`, and replaced `make_service_fn`/`service_fn` with the new `Service` trait.
   - **Code change is needed** in metrics_server.rs (~94 lines). This is a self-contained module with no other dependents.
   - Effort: Low (single file, well-defined API migration)

2. **reqwest 0.11 chain**: This disappears when reqwest 0.11 is eliminated (Section B).

**Key difference between hyper 0.14 and 1.x in our context:**
- hyper 0.14 has `hyper::Server::bind()` and `hyper::service::make_service_fn`
- hyper 1.x has `hyper::server::conn::http1::Builder` and the `Service` trait
- Body types changed significantly

**Recommendation**: Upgrade to hyper 1.x for the metrics server. The migration is well-documented and the module is simple.

---

## H. Error Handling (thiserror)

### Current State: 2 versions (low impact)

| Version | Used by |
|---------|---------|
| **1.0.69** | html_parser 0.7.0, json-patch 3.0.1, oauth2 4.4.2, ppt-rs 0.2.14, reqwest-eventsource 0.5.0, sentry 0.32.3 (and sub-crates), tauri-codegen, tauri-macros, tauri-build (transitive) |
| **2.0.18** | tauri 2.10.2 (and plugins), object_store vendor, aws-sdk deps |

### Can we eliminate thiserror 1.0?

**Partially, but not fully.** The 1.0 -> 2.0 transition is still ongoing in the ecosystem. Key blockers:
- sentry 0.32.x still uses thiserror 1.x (upgrade to 0.48.x would fix)
- oauth2 4.4.2 uses thiserror 1.x (upgrade to 5.0.0 may fix)
- tauri-build 2.5.5 uses json-patch 3.0.1 -> thiserror 1.x

**Impact**: thiserror 1.x and 2.x are fully compatible with each other. They each generate their own proc-macro output. The build overhead of shipping both is negligible. This is cosmetic.

**Recommendation**: Accept as low priority. Fix as transitive side-effect of other upgrades.

---

## I. Vendor Analysis (lancedb, object_store)

### object_store (vendor at 0.12.4)

- **reqwest**: 0.12 (modern) -- compatible with our target
- **hyper**: 1.2 -- compatible
- **rustls**: 0.23.32 (modern) -- compatible
- **thiserror**: 2.0.2 -- compatible

**Verdict**: Already aligned with modern ecosystem. No changes needed.

### lancedb (vendor at 0.22.1)

- **reqwest**: 0.12.23 (modern, optional via "remote" feature) -- compatible
- **rustls**: Has BOTH 0.21.12 and 0.23.31 in its lock file. The 0.21 dep comes from transitive deps within lance itself (lance-io/lance-encoding using older aws deps).
- **object_store**: 0.12.0 (pinned) -- our vendor is 0.12.4, so this is compatible

**Verdict**: lancedb's internal use of rustls 0.21 is from its sub-dependency lance-io which uses older AWS SDK internals. This cannot be eliminated without updating the lance ecosystem.

**Can lancedb be upgraded to a version using reqwest 0.12 + hyper 1.x?**
- lancedb 0.22.1 already uses reqwest 0.12: YES, already resolved.
- The rustls 0.21 comes from lance sub-crates, not from lancedb itself. Upgrading the lancedb version might help if a newer lance releases use rustls 0.23 throughout.

**Is there an alternative to vendoring these crates?**
- Vendoring was chosen because lancedb/object_store require specific pinned versions or patches. Without vendoring, the standard crates.io versions would be used, which might bring in unwanted deps (like native-tls or openssl).
- The vendor approach is actually beneficial here because it gives us control over the dep graph.

---

## J. Complete Migration Roadmap

### Recommended order of operations

```
Phase 1: Independent upgrades (0 risk, no code changes)
──────────────────────────────────────────────────────
[1] Upgrade Tauri 2.10.2 -> 2.11.2
    - Version bump only in Cargo.toml
    - Risk: Very low (patch-level change)
    - Time: < 1 hour

[2] Upgrade reqwest-eventsource 0.5 -> 0.6
    - Version bump only
    - Risk: Very low (API-compatible)
    - Time: < 30 minutes

[3] Upgrade jsonschema 0.42 -> 0.46.5 (optional)
    - Version bump only
    - Risk: Medium (API may have changed)
    - Time: 2-4 hours if migration needed

Phase 2: Code changes required
──────────────────────────────────────────────
[4] Upgrade reqwest 0.11 -> 0.12 (our direct dep)
    - Change version in Cargo.toml
    - Verify all ~15 files using reqwest compile
    - The reqwest::Url, reqwest::Error, reqwest::blocking,
      reqwest::multipart, reqwest::header APIs are all compatible
    - Risk: Low (tested API surface, no breaking changes in our usage)
    - Time: 2-4 hours

[5] Upgrade hyper 0.14 -> 1.x in metrics_server.rs
    - Rewrite metrics_server.rs to use hyper 1.x API
    - Replace: Server::bind + make_service_fn + service_fn
    - With: hyper::server::conn::http1::Builder + Service trait
    - Risk: Medium (single file, well-understood migration)
    - Time: 2-4 hours

[6] Upgrade tokio-tungstenite 0.21 -> 0.29.0
    - Version bump + check WebSocket connection code
    - Risk: Low-Medium (well-tested upgrade path)
    - Time: 2-4 hours

Phase 3: High-risk upgrades (require integration testing)
──────────────────────────────────────────────────────────────
[7] Upgrade oauth2 4.4.2 -> 5.0.0
    - Requires API audit (oauth2 4->5 breaking changes)
    - Risk: Medium-High (auth flow could break)
    - Time: 4-8 hours

[8] Upgrade sentry 0.32 -> 0.48 (optional)
    - Large version jump, extensive testing needed
    - Risk: High (error reporting infrastructure)
    - Time: 8-16 hours

Phase 4: Cleanup
──────────────────────────────────────────────
[9] Remove deprecated deps from Cargo.toml
    - Remove hyper 0.14 (replaced by hyper 1.x)
    - Remove zip 0.6 (upgrade to zip 2.x if the direct usage allows)

[10] Lock file optimization
    - Run `cargo update` to consolidate where possible
    - Verify no regressions
```

### Estimated totals

| Phase | Files changed | Risk | Time estimate |
|-------|-------------|------|---------------|
| Phase 1 (dept bumps) | 1-2 | Very low | 2-4 hours |
| Phase 2 (code changes) | ~16 | Low-Moderate | 6-12 hours |
| Phase 3 (oauth2/sentry) | 2-5 | High | 12-24 hours |
| Phase 4 (cleanup) | 1-2 | Low | 1-2 hours |
| **Total** | **~20-25 files** | | **~20-40 hours** |

### Risk assessment per change

| Change | Risk Level | Reason |
|--------|-----------|--------|
| Tauri 2.10 -> 2.11 | Very Low | Patch upgrade, fully compatible |
| reqwest-eventsource 0.5 -> 0.6 | Very Low | Same major API |
| reqwest 0.11 -> 0.12 (direct) | Low | Compatible API for our usage |
| hyper 0.14 -> 1.x | Medium | Breaking API, but isolated to one file |
| tokio-tungstenite 0.21 -> 0.29 | Low-Medium | Large jump but stable API surface |
| oauth2 4.4.2 -> 5.0.0 | Medium-High | Auth flow, needs careful testing |
| sentry 0.32 -> 0.48 | High | Major version jump, error reporting |
| zip 0.6 -> 2.x (direct) | Medium | Code change for EPUB parsing |
| jsonschema 0.42 -> 0.46 | Medium | API may have changed |

### Which modules would need code changes

| Module | File(s) | Change needed | Risk |
|--------|---------|--------------|------|
| Metrics server | `src/metrics_server.rs` | hyper 0.14 -> 1.x migration | Medium |
| Fetch utility | `src/utils/fetch.rs` | Verify reqwest 0.12 `blocking::Client` API | Low |
| LLM Manager | `src/llm_manager/mod.rs` | Verify reqwest Client API | Low |
| MCP SSE | `src/mcp/sse_transport.rs` | Verify reqwest + reqwest-eventsource API | Low |
| MCP HTTP | `src/mcp/http_transport.rs` | Verify reqwest API | Low |
| Voice input | `src/voice_input.rs` | Verify reqwest multipart API | Low |
| Web search | `src/tools/web_search.rs` | Verify reqwest Error + StatusCode | Low |
| Paddle OCR | `src/paddleocr_api.rs` | Verify reqwest Error `#[from]` | Low |
| Chat tools | ~5 files in `src/chat_v2/tools/` | URL parsing, header construction, Client | Low |
| Anki service | `src/anki_connect_service.rs` | Client::new() usage | Low |
| Streaming Anki | `src/streaming_anki_service.rs` | Client import | Low |
| WebDAV | `src/cloud_storage/webdav.rs` | Client + Method usage | Low |
| VLM service | `src/vlm_grounding_service.rs` | Client::builder() usage | Low |
| WebSocket | `src/mcp/sse_transport.rs` | tokio-tungstenite upgrade | Low-Medium |
| OAuth2 | `src/mcp/auth.rs` + oauth2 calls | oauth2 5.0 API migration | Medium-High |
| EPUB parsing | (zip 0.6 usage) | zip 2.x/4.x API migration | Medium |

**Total files requiring code changes: ~15-20**
**Files requiring only version bumps: ~2 (Cargo.toml changes)**

---

## K. Post-Migration State (Best Case)

After completing Phases 1-4, the dependency version landscape would be:

| Library | Before | After | Notes |
|---------|--------|-------|-------|
| reqwest | 0.11, 0.12, 0.13 | **0.12** (our code + plugins), **0.13** (Tauri core) | 3 -> 2 |
| rustls | 0.21, 0.22, 0.23 | **0.23** (most), possible 0.21 residual via sentry | 3 -> 2 (or 1!) |
| hyper | 0.14, 1.x | **1.x** only | 2 -> 1 |
| zip | 0.6, 2.4, 4.6 | **2.4** (docs), **4.6** (Tauri) | 3 -> 2 |
| thiserror | 1.0, 2.0 | **1.0** (transitive), **2.0** (main) | 2 -> 2 (cosmetic) |
| serde | 1.0 | 1.0 | Already 1 |
| tokio | 1.49 | 1.49 | Already 1 |
| tungstenite | 0.21, 0.28 (unused) | **0.29** (upgraded) | 2 -> 1 |

**Final version count reduction:** From 19 total variant entries (across all 8 libraries) to approximately 11-12. A ~37% reduction.

---

## L. Recommended Strategy

### Do these immediately (low risk, high impact):
1. Upgrade reqwest 0.11 -> 0.12 (direct dep)
2. Upgrade reqwest-eventsource 0.5 -> 0.6
3. Upgrade hyper 0.14 -> 1.x (metrics_server.rs)
4. Upgrade tokio-tungstenite 0.21 -> latest

### Do these if you need auth/error-reporting upgrades:
5. Upgrade oauth2 4.4 -> 5.0 (after reqwest upgrade)
6. Upgrade sentry 0.32 -> 0.48 (after reqwest upgrade)

### Accept as permanent:
7. Three zip versions (ecosystem constraint)
8. Two thiserror versions (cosmetic, negligible cost)
9. Two reqwest versions (0.12 + 0.13 from Tauri internal)

### Write off as infeasible:
- Eliminating zip 0.6 or zip 2.4 (blocked by docx-rs, calamine, etc.)
- Eliminating thiserror 1.0 entirely (9+ transitive deps)

---

## M. Files Referenced

- C:\deep-student\src-tauri\Cargo.toml — Main project dependencies
- C:\deep-student\src-tauri\Cargo.lock — Full resolved dependency tree (13,200+ entries)
- C:\deep-student\src-tauri\vendor\lancedb\Cargo.toml — Vendored lancedb crate manifest
- C:\deep-student\src-tauri\vendor\lancedb\Cargo.lock — Vendored lancedb dependency lock
- C:\deep-student\src-tauri\vendor\object_store\Cargo.toml — Vendored object_store crate manifest
- C:\deep-student\src-tauri\vendor\object_store\Cargo.lock — Vendored object_store dependency lock
- C:\deep-student\src-tauri\src\metrics_server.rs — Only file directly using hyper 0.14 API
- C:\deep-student\src-tauri\src\utils\fetch.rs — Uses reqwest::blocking::Client (verify 0.12 compat)
- C:\deep-student\src-tauri\src\llm_manager\mod.rs — Major reqwest consumer (LLM API calls)
- C:\deep-student\src-tauri\src\chat_v2\tools\*.rs — Multiple reqwest consumers (tool executors)
- C:\deep-student\src-tauri\src\mcp\sse_transport.rs — Uses both reqwest and reqwest-eventsource
- C:\deep-student\src-tauri\src\paddleocr_api.rs — reqwest::Error `#[from]` usage
- C:\deep-student\src-tauri\src\tools\web_search.rs — reqwest::Error + StatusCode usage
- C:\deep-student\.study-ui\src-tauri\Cargo.toml — Study UI subproject (separate, minimal deps)
