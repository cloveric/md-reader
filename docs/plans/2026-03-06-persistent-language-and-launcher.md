# Persistent Language And Launcher Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make language selection persist reliably across launches by fixing WebView storage location, while keeping the executable in the project folder and using a desktop shortcut for launching.

**Architecture:** Reuse the existing `localStorage` locale flow in the editor shell, but anchor WebView state to a stable application data directory via `wry::WebContext`. Keep GitHub release packaging dual-platform and handle the desktop shortcut as a local post-build step.

**Tech Stack:** Rust, Wry/WebView2, embedded HTML shell, PowerShell, GitHub Actions

---

### Task 1: Add a failing regression test for stable WebView persistence wiring

**Files:**
- Modify: `tests/editor_shell_i18n.rs`
- Create: `tests/runtime_persistence.rs`

**Step 1: Write the failing test**

Add a test that reads `src/main.rs` and fails unless the app constructs the webview with a shared `WebContext` and a stable data-directory helper.

**Step 2: Run test to verify it fails**

Run: `cargo test configures_stable_webview_data_directory --test runtime_persistence`
Expected: FAIL because `main.rs` does not yet use `WebContext`.

**Step 3: Write minimal implementation**

Add the runtime path helper and wire `main.rs` to `WebViewBuilder::with_web_context`.

**Step 4: Run test to verify it passes**

Run: `cargo test configures_stable_webview_data_directory --test runtime_persistence`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/runtime_persistence.rs src/runtime_paths.rs src/lib.rs src/main.rs
git commit -m "fix: persist webview storage in app data"
```

### Task 2: Cover path selection logic with unit tests

**Files:**
- Create: `src/runtime_paths.rs`

**Step 1: Write the failing test**

Add unit tests for Windows and macOS path selection using pure helper inputs.

**Step 2: Run test to verify it fails**

Run: `cargo test runtime_paths`
Expected: FAIL until the helper returns the expected stable application data paths.

**Step 3: Write minimal implementation**

Implement the helper so Windows uses `LOCALAPPDATA\\md-bider\\webview`, macOS uses `~/Library/Application Support/md-bider/webview`, and other platforms fall back sensibly.

**Step 4: Run test to verify it passes**

Run: `cargo test runtime_paths`
Expected: PASS

**Step 5: Commit**

```bash
git add src/runtime_paths.rs
git commit -m "test: cover runtime path selection"
```

### Task 3: Verify the full app and refresh the local launcher

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

**Step 1: Write the release metadata**

Bump the version and document the persistence fix in the changelog.

**Step 2: Run verification**

Run: `cargo test`
Expected: PASS

Run: `cargo build --release`
Expected: PASS

**Step 3: Refresh the local launcher**

Keep `target/release/md-bider.exe` as the canonical local binary and create/update a desktop `.lnk` shortcut that points to it.

**Step 4: Publish**

Commit, push `main`, tag the new version, and let `.github/workflows/release.yml` publish both Windows and macOS assets.
