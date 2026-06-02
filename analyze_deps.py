#!/usr/bin/env python3
"""Analyze Rust dependency graph under src-tauri/src."""

import os
import re
import sys
from collections import defaultdict, deque

SRC_DIR = r"C:\deep-student\src-tauri\src"

# ── Step 1: Map every .rs file to its module path ──────────────────────────

def path_to_module(rel_path):
    """Convert a relative file path to its Rust module path.

    E.g. "chat_v2/tools/mod.rs" -> "crate::chat_v2::tools"
         "chat_v2/tools/executor.rs" -> "crate::chat_v2::tools::executor"
         "lib.rs" -> "crate"
         "main.rs" -> "crate"
    """
    # Remove extension
    no_ext = rel_path.replace(".rs", "")
    parts = no_ext.replace("\\", "/").split("/")

    # Handle mod.rs and lib.rs/main.rs
    if parts[-1] == "mod" or parts[-1] == "lib" or parts[-1] == "main":
        parts = parts[:-1]
        if not parts:
            return "crate"
        return "crate::" + "::".join(parts)
    return "crate::" + "::".join(parts)


def module_to_filename(module_path):
    """Convert a module path back to an absolute file path."""
    if module_path == "crate":
        return os.path.join(SRC_DIR, "lib.rs")
    # Remove "crate::" prefix
    relative = module_path[7:].replace("::", "/")
    # Try mod.rs first, then .rs
    candidates = [
        os.path.join(SRC_DIR, relative, "mod.rs"),
        os.path.join(SRC_DIR, relative + ".rs"),
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return None


# Map: module_path -> file_path
module_to_file = {}
# Reverse map: file_path -> module_path
file_to_module = {}

for root, dirs, files in os.walk(SRC_DIR):
    for f in files:
        if f.endswith(".rs"):
            full_path = os.path.join(root, f)
            rel_path = os.path.relpath(full_path, SRC_DIR)
            mp = path_to_module(rel_path)
            module_to_file[mp] = full_path
            file_to_module[full_path] = mp

print(f"Total files found: {len(module_to_file)}")

# ── Step 2: Extract imports from each file ────────────────────────────────

# Pattern for "use crate::...." and "use super::..."
CRATE_USE_RE = re.compile(r'^\s*use\s+(crate::[\w:]+)')
SUPER_USE_RE = re.compile(r'^\s*use\s+(super(?:::\w+)+)')
CRATE_USE_BRACE_RE = re.compile(r'^\s*use\s+(crate::[\w:]+)::\{([^}]+)\}')

def resolve_super(module_path, super_path):
    """Resolve 'super::foo::bar' from a given module path."""
    parts = module_path.split("::")
    # super count = number of "super" segments
    supers = super_path.split("::")
    count = 0
    for s in supers:
        if s == "super":
            count += 1
        else:
            break
    # Pop 'crate' and then 'count' more segments
    # Actually module_path starts with crate
    tail = supers[count:]  # remaining after super::
    if count >= len(parts):
        return None
    resolved = "::".join(parts[:-count] + tail)
    return resolved


def file_name_to_module(file_path):
    """Get module name from a file path (for mod declarations)."""
    return file_to_module.get(file_path, "")


# Parse all files
# forward map: module_path -> set of module_paths it imports
forward = defaultdict(set)
# also track what each file uses from its own crate
file_imports = defaultdict(set)  # file_path -> set of module paths

for mod_path, file_path in module_to_file.items():
    try:
        with open(file_path, "r", encoding="utf-8") as fh:
            content = fh.read()
    except Exception as e:
        print(f"  Error reading {file_path}: {e}", file=sys.stderr)
        continue

    imports = set()
    for line in content.split("\n"):
        line = line.strip()

        # Skip comments and non-use lines
        if not line.startswith("use "):
            continue
        if line.startswith("use std") or line.startswith("use serde") or \
           line.startswith("use tokio") or line.startswith("use tauri") or \
           line.startswith("use chrono") or line.startswith("use rusqlite") or \
           line.startswith("use anyhow") or line.startswith("use thiserror") or \
           line.startswith("use log") or line.startswith("use tracing") or \
           line.startswith("use futures") or line.startswith("use uuid") or \
           line.startswith("use regex") or line.startswith("use serde_json") or \
           line.startswith("use base64") or line.startswith("use sha2") or \
           line.startswith("use reqwest") or line.startswith("use parking_lot") or \
           line.startswith("use lazy_static") or line.startswith("use once_cell") or \
           line.startswith("use rand") or line.startswith("use url") or \
           line.startswith("use clap") or line.startswith("use mime") or \
           line.startswith("use tempfile") or line.startswith("use walkdir") or \
           line.startswith("use notify") or line.startswith("use image") or \
           line.startswith("use encoding") or line.startswith("use html2text") or \
           line.startswith("use human_format") or line.startswith("use itertools") or \
           line.startswith("use jsonwebtoken") or line.startswith("use lance") or \
           line.startswith("use moka") or line.startswith("use p256") or \
           line.startswith("use pulldown_cmark") or line.startswith("use resvg") or \
           line.startswith("use scraper") or line.startswith("use smallvec") or \
           line.startswith("use time") or line.startswith("use tokio_stream") or \
           line.startswith("use tower_http") or line.startswith("use typst") or \
           line.startswith("use unicode") or line.startswith("use xxhash_rust") or \
           line.startswith("use zstd") or line.startswith("use p384") or \
           line.startswith("use hmac") or line.startswith("use aes") or \
           line.startswith("use pbkdf2") or line.startswith("use hex") or \
           line.startswith("use zeroize") or line.startswith("use arrow") or \
           line.startswith("use comfy_table") or line.startswith("use nanoid") or \
           line.startswith("use textwrap") or line.startswith("use infer") or \
           line.startswith("use flate2") or line.startswith("use aws") or \
           line.startswith("use backtrace") or line.startswith("use bytes") or \
           line.startswith("use crypto") or line.startswith("use dashmap") or \
           line.startswith("use data_encoding") or \
           line.startswith("use dircpy") or line.startswith("use digest") or \
           line.startswith("use easy_reader") or line.startswith("use either") or \
           line.startswith("use encoding_rs") or line.startswith("use fancy_regex") or \
           line.startswith("use git2") or line.startswith("use glob") or \
           line.startswith("use google_cloud") or line.startswith("use gstreamer") or \
           line.startswith("use html_escape") or line.startswith("use indicatif") or \
           line.startswith("use libc") or line.startswith("use lru") or \
           line.startswith("use napi") or line.startswith("use nom") or \
           line.startswith("use num_cpus") or line.startswith("use num_traits") or \
           line.startswith("use open") or line.startswith("use opencv") or \
           line.startswith("use percent_encoding") or \
           line.startswith("use png") or line.startswith("use pretty_assertions") or \
           line.startswith("use pulldown_cmark") or line.startswith("use quick_xml") or \
           line.startswith("use ring") or line.startswith("use rsa") or \
           line.startswith("use scopeguard") or line.startswith("use serde") or \
           line.startswith("use signal_hook") or line.startswith("use sqlx") or \
           line.startswith("use ssd1306") or line.startswith("use strsim") or \
           line.startswith("use syn") or line.startswith("use sysinfo") or \
           line.startswith("use tantivy") or line.startswith("use termion") or \
           line.startswith("use tinytemplate") or line.startswith("use toml") or \
           line.startswith("use unicode_segmentation") or \
           line.startswith("use unicode_width") or line.startswith("use utf8_read") or \
           line.startswith("use webp") or line.startswith("use wry") or \
           line.startswith("use xml") or line.startswith("use yaml") or \
           line.startswith("use zip"):
            continue

        # Skip test imports
        if "test" in line.lower() and ("test" in mod_path or "test" in line):
            continue

        # Handle brace imports: use crate::module::{Foo, Bar}
        # First, check if it starts with "use crate::..."
        m = CRATE_USE_BRACE_RE.match(line)
        if m:
            base = m.group(1)
            # Check each item in the braces — but for dependency tracking,
            # the base module is enough
            if base in module_to_file:
                imports.add(base)
            continue

        # Simple "use crate::foo::bar"
        m = CRATE_USE_RE.match(line)
        if m:
            target = m.group(1)
            # Find the longest prefix that maps to a module
            parts = target.split("::")
            # parts[0] = "crate"
            for i in range(len(parts), 1, -1):
                candidate = "::".join(parts[:i])
                if candidate in module_to_file:
                    imports.add(candidate)
                    break
            continue

        # Handle "use super::..."
        m = SUPER_USE_RE.match(line)
        if m:
            resolved = resolve_super(mod_path, m.group(1))
            if resolved and resolved in module_to_file:
                imports.add(resolved)
            continue

    # Also check for `mod foo;` declarations inside the file to establish
    # parent-child relationships
    # Actually children are tracked via the file system structure already
    # because path_to_module creates the right paths.

    forward[mod_path] = imports
    file_imports[file_path] = imports

print(f"Parsed {len(forward)} files for imports")

# ── Step 3: Build reverse dependency map ──────────────────────────────────

reverse = defaultdict(set)  # module_path -> set of module_paths that depend on it

for mod_path, deps in forward.items():
    for dep in deps:
        reverse[dep].add(mod_path)

# ── Step 4: Detect circular dependencies ──────────────────────────────────

def find_cycles(graph, max_cycle_length=10):
    """Find all simple cycles in a directed graph using Tarjan's algorithm."""
    # Use Johnson's algorithm for finding all elementary circuits
    # But the graph is large, so let's use a simpler DFS-based approach
    # that finds strongly connected components first.

    # Kosaraju or Tarjan to find SCCs
    index = 0
    indices = {}
    lowlink = {}
    on_stack = {}
    stack = []
    sccs = []

    def strongconnect(v, graph):
        nonlocal index
        indices[v] = index
        lowlink[v] = index
        index += 1
        stack.append(v)
        on_stack[v] = True

        for w in graph.get(v, set()):
            if w not in indices:
                strongconnect(w, graph)
                lowlink[v] = min(lowlink[v], lowlink[w])
            elif on_stack.get(w, False):
                lowlink[v] = min(lowlink[v], indices[w])

        if lowlink[v] == indices[v]:
            scc = []
            while True:
                w = stack.pop()
                on_stack[w] = False
                scc.append(w)
                if w == v:
                    break
            if len(scc) > 1:
                sccs.append(scc)

    for v in graph:
        if v not in indices:
            strongconnect(v, forward)

    # Now find cycles within each SCC (heuristic: just list the SCC)
    cycles = []
    for scc in sccs:
        if len(scc) >= 2:
            cycles.append(sorted(scc))

    return cycles

cycles = find_cycles(forward)
print(f"Found {len(cycles)} strongly connected components with cycles")

# ── Step 5: Compute fan-in and fan-out ────────────────────────────────────

# Fan-in = number of modules that import this module
fan_in = {}
for mod_path in module_to_file:
    fan_in[mod_path] = len(reverse.get(mod_path, set()))

# Fan-out = number of internal modules this module imports
fan_out = {}
for mod_path in module_to_file:
    fan_out[mod_path] = len(forward.get(mod_path, set()))

# ── Step 6: Sort and display results ──────────────────────────────────────

# Most depended-on modules
sorted_by_fan_in = sorted(fan_in.items(), key=lambda x: -x[1])

print("\n=== Top 20 Most Depended-On Modules (Fan-In) ===")
for mod, count in sorted_by_fan_in[:20]:
    file_path = module_to_file.get(mod, "")
    print(f"  {count:3d}  {mod}  ({os.path.relpath(file_path, SRC_DIR) if file_path else '?'})")

print("\n=== Modules with Most Dependencies (Fan-Out) ===")
sorted_by_fan_out = sorted(fan_out.items(), key=lambda x: -x[1])
for mod, count in sorted_by_fan_out[:20]:
    file_path = module_to_file.get(mod, "")
    print(f"  {count:3d}  {mod}  ({os.path.relpath(file_path, SRC_DIR) if file_path else '?'})")

print("\n=== All Circular Dependency Chains ===")
for i, cycle in enumerate(cycles):
    print(f"  Cycle {i+1} ({len(cycle)} nodes):")
    for node in cycle:
        file_path = module_to_file.get(node, "")
        print(f"    {node}")
        for dep in forward.get(node, set()):
            if dep in cycle:
                print(f"      -> {dep}")
    print()

# ── Step 7: Check for excessive dependencies (>20 internal deps) ──────────

print("\n=== Modules with Excessive Internal Dependencies (>20) ===")
flagged = []
for mod_path, count in sorted_by_fan_out:
    if count > 20:
        file_path = module_to_file.get(mod_path, "")
        flagged.append((mod_path, count, file_path))
        print(f"  {count:3d}  {mod_path}  ({os.path.relpath(file_path, SRC_DIR) if file_path else '?'})")

if not flagged:
    print("  (none)")

# ── Step 8: Output machine-readable data for the report ───────────────────

print("\n\n=== MACHINE-READABLE ===")
print(f"TOTAL_MODULES:{len(module_to_file)}")
print(f"TOTAL_EDGES:{sum(len(d) for d in forward.values())}")

# Print all modules with their fan-in and fan-out
print("MODULES:")
for mod_path in sorted(module_to_file.keys()):
    f_in = fan_in[mod_path]
    f_out = fan_out[mod_path]
    file_path = module_to_file[mod_path]
    print(f"  {f_in}|{f_out}|{mod_path}|{os.path.relpath(file_path, SRC_DIR)}")

# Print cycles
print("CYCLES:")
for i, cycle in enumerate(cycles):
    print(f"  CYCLE:{i+1}:{','.join(cycle)}")

# Print flagged modules
print("FLAGGED:")
for mod_path, count, file_path in flagged:
    print(f"  FLAG:{count}:{mod_path}:{os.path.relpath(file_path, SRC_DIR)}")
