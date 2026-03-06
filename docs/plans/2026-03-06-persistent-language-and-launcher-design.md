# Persistent Language And Launcher Design

## Context

`md-bider` stores the selected UI language in the embedded editor shell via `localStorage`. On Windows, the current WebView2 user data directory follows the executable location and filename, so copying or renaming the executable creates a new storage silo. That makes the language preference appear to reset even though the front-end save logic already exists.

The current local delivery flow also leaves standalone executables on the desktop, which encourages launching different binaries and further fragments WebView2 state.

## Decision

1. Keep the existing front-end locale persistence logic in `assets/editor_shell.html`.
2. Make the embedded WebView use a stable application data directory that does not depend on the executable path or filename.
3. Keep the built executable in the project output directory and create a desktop `.lnk` shortcut that points to that stable executable.
4. Keep both Windows and macOS release artifacts in GitHub releases.

## Approach

Add a small runtime-path helper module that computes the application data directory for Windows, macOS, and a sensible fallback for other platforms. Use `wry::WebContext` in `src/main.rs` so WebView storage is anchored under that stable path.

Add regression coverage for the stable WebView context wiring and path selection. Leave the release workflow dual-platform so macOS packaging remains part of every tagged release.

## Success Criteria

- Selecting Chinese persists across app restarts.
- Persistence survives launching via a desktop shortcut instead of a copied executable.
- The repository still produces Windows and macOS release assets.
- The desktop entry point becomes a shortcut rather than another copied executable.
