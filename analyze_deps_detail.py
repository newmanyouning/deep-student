#!/usr/bin/env python3
"""Detailed cycle and edge analysis for Rust dependencies."""

import os
import re
import sys
from collections import defaultdict

SRC_DIR = r"C:\deep-student\src-tauri\src"

def path_to_module(rel_path):
    no_ext = rel_path.replace(".rs", "")
    parts = no_ext.replace("\\", "/").split("/")
    if parts[-1] == "mod" or parts[-1] == "lib" or parts[-1] == "main":
        parts = parts[:-1]
        if not parts:
            return "crate"
        return "crate::" + "::".join(parts)
    return "crate::" + "::".join(parts)

module_to_file = {}
file_to_module = {}
for root, dirs, files in os.walk(SRC_DIR):
    for f in files:
        if f.endswith(".rs"):
            full_path = os.path.join(root, f)
            rel_path = os.path.relpath(full_path, SRC_DIR)
            mp = path_to_module(rel_path)
            module_to_file[mp] = full_path
            file_to_module[full_path] = mp

def resolve_super(module_path, super_path):
    parts = module_path.split("::")
    supers = super_path.split("::")
    count = 0
    for s in supers:
        if s == "super":
            count += 1
        else:
            break
    tail = supers[count:]
    if count >= len(parts):
        return None
    return "::".join(parts[:-count] + tail)

# Parse all imports with full detail
forward = defaultdict(set)
forward_detail = defaultdict(list)  # module_path -> list of (line, target)

for mod_path, file_path in module_to_file.items():
    try:
        with open(file_path, "r", encoding="utf-8") as fh:
            content = fh.read()
    except:
        continue

    for line in content.split("\n"):
        stripped = line.strip()
        if not stripped.startswith("use "):
            continue

        # Skip external crates
        if any(stripped.startswith(f"use {e}") for e in ["std", "serde", "tokio", "tauri", "chrono",
            "rusqlite", "anyhow", "thiserror", "log", "tracing", "futures", "uuid", "regex",
            "serde_json", "base64", "sha2", "reqwest", "parking_lot", "lazy_static", "once_cell",
            "rand", "url", "clap", "mime", "tempfile", "walkdir", "notify", "image",
            "encoding", "html2text", "human_format", "itertools", "jsonwebtoken", "lance",
            "moka", "p256", "pulldown_cmark", "resvg", "scraper", "smallvec", "time",
            "tokio_stream", "tower_http", "typst", "unicode", "xxhash_rust", "zstd",
            "p384", "hmac", "aes", "pbkdf2", "hex", "zeroize", "arrow", "comfy_table",
            "nanoid", "textwrap", "infer", "flate2", "aws", "backtrace", "bytes",
            "crypto", "dashmap", "data_encoding", "dircpy", "digest", "easy_reader",
            "either", "encoding_rs", "fancy_regex", "git2", "glob", "google_cloud",
            "gstreamer", "html_escape", "indicatif", "libc", "lru", "napi", "nom",
            "num_cpus", "num_traits", "open", "opencv", "percent_encoding", "png",
            "pretty_assertions", "quick_xml", "ring", "rsa", "scopeguard",
            "signal_hook", "sqlx", "ssd1306", "strsim", "syn", "sysinfo", "tantivy",
            "termion", "tinytemplate", "toml", "unicode_segmentation", "unicode_width",
            "utf8_read", "webp", "wry", "xml", "yaml", "zip"]):
            continue

        # Also skip test-related lines in non-test files
        if "test" in stripped.lower() and "test" not in file_path:
            continue

        # Handle brace imports
        m = re.match(r'^\s*use\s+(crate::[\w:]+)::\{([^}]+)\}', stripped)
        if m:
            base = m.group(1)
            if base in module_to_file:
                forward[mod_path].add(base)
                forward_detail[mod_path].append((stripped, base))
            continue

        m = re.match(r'^\s*use\s+(crate::[\w:]+)', stripped)
        if m:
            target = m.group(1)
            parts = target.split("::")
            for i in range(len(parts), 1, -1):
                candidate = "::".join(parts[:i])
                if candidate in module_to_file:
                    forward[mod_path].add(candidate)
                    forward_detail[mod_path].append((stripped, candidate))
                    break
            continue

        m = re.match(r'^\s*use\s+(super(?:::\w+)+)', stripped)
        if m:
            resolved = resolve_super(mod_path, m.group(1))
            if resolved and resolved in module_to_file:
                forward[mod_path].add(resolved)
                forward_detail[mod_path].append((stripped, resolved))

# Build reverse
reverse = defaultdict(set)
for mod_path, deps in forward.items():
    for dep in deps:
        reverse[dep].add(mod_path)

# ── Analyze each cycle ──

cycles_info = [
    (["crate::vfs::repos::folder_repo", "crate::vfs::repos::path_cache_repo"], "2-node cycle"),
    (["crate::question_sync_service", "crate::vfs::repos::question_repo"], "2-node cycle"),
    (["crate::vfs::embedding_service", "crate::vfs::indexing", "crate::vfs::pdf_processing_service"], "3-node cycle"),
    (["crate::data_governance::commands_backup", "crate::data_governance::commands_restore"], "2-node cycle"),
]

print("=" * 80)
print("DETAILED CYCLE ANALYSIS")
print("=" * 80)

for cycle, desc in cycles_info:
    print(f"\n--- Cycle: {desc} ---")
    for node in cycle:
        file_path = module_to_file.get(node, "")
        print(f"\n  {node} ({os.path.relpath(file_path, SRC_DIR) if file_path else '?'})")
        print(f"  Depends on (internal only):")
        for dep in sorted(forward.get(node, set())):
            arrow = " <-- CYCLE " if dep in cycle else ""
            dep_file = module_to_file.get(dep, "")
            print(f"    -> {dep}{arrow}")
            # Show the actual import line
            for line_text, tgt in forward_detail.get(node, []):
                if tgt == dep:
                    print(f"       {line_text}")

# ── Analyze lib.rs ──

print("\n" + "=" * 80)
print("LIB.RS DEPENDENCIES (the 'glue' layer)")
print("=" * 80)
lib_path = os.path.join(SRC_DIR, "lib.rs")
if os.path.exists(lib_path):
    with open(lib_path, "r", encoding="utf-8") as fh:
        content = fh.read()
    for line in content.split("\n"):
        if "mod " in line and not line.strip().startswith("//"):
            print(f"  {line.strip()}")

# ── LLM Manager adapters analysis ──

print("\n" + "=" * 80)
print("LLM MANAGER ADAPTERS IMPORT ANALYSIS")
print("=" * 80)

for mod_path in sorted(module_to_file.keys()):
    if "llm_manager" in mod_path:
        file_path = module_to_file[mod_path]
        imports = forward.get(mod_path, set())
        print(f"\n  {mod_path}")
        for dep in sorted(imports):
            print(f"    -> {dep}")

# ── Big fan-out modules: what they import ──

print("\n" + "=" * 80)
print("MODULES WITH HIGH FAN-OUT: DETAILED IMPORTS")
print("=" * 80)

high_fanout = sorted(
    [(mp, len(forward.get(mp, set()))) for mp in module_to_file],
    key=lambda x: -x[1]
)

for mod_path, count in high_fanout:
    if count >= 10:
        print(f"\n  {mod_path} (fan-out={count})")
        file_path = module_to_file.get(mod_path, "")
        print(f"  File: {os.path.relpath(file_path, SRC_DIR)}")
        for dep in sorted(forward.get(mod_path, set())):
            print(f"    -> {dep}")
        # Show import lines
        for line_text, tgt in forward_detail.get(mod_path, []):
            print(f"       {line_text}")

# ── Top fan-in modules: who imports them ──

print("\n" + "=" * 80)
print("TOP FAN-IN MODULES: WHO IMPORTS THEM")
print("=" * 80)

high_fanin = sorted(
    [(mp, len(reverse.get(mp, set()))) for mp in module_to_file],
    key=lambda x: -x[1]
)

for mod_path, count in high_fanin[:10]:
    print(f"\n  {mod_path} (fan-in={count})")
    file_path = module_to_file.get(mod_path, "")
    print(f"  File: {os.path.relpath(file_path, SRC_DIR)}")
    for dep in sorted(reverse.get(mod_path, set())):
        print(f"    <- {dep}")
