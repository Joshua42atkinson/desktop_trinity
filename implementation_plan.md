# Implementation Plan - Simplifying Deployment (Single Executable)

## Goal

Create a "Portable Executable" for the Daydream application that bundles the frontend and backend into a single `.exe` file for easy distribution on Windows.

## User Review Required
>
> [!IMPORTANT]
> This approach embeds the frontend static files directly into the backend binary. This increases the binary size but makes distribution trivial (just one file).

## Proposed Changes

### Backend Dependencies

#### [MODIFY] [backend/Cargo.toml](file:///c:/Users/Trinity/Documents/daydream/Day_Dream/backend/Cargo.toml)

- Add `rust-embed` (for embedding files).
- Add `mime_guess` (for serving correct content types).

### Backend Logic

#### [NEW] [backend/src/static_assets.rs](file:///c:/Users/Trinity/Documents/daydream/Day_Dream/backend/src/static_assets.rs)

- Define `Asset` struct using `#[derive(RustEmbed)]`.
- Point it to the `../frontend/dist` directory.

#### [MODIFY] [backend/src/main.rs](file:///c:/Users/Trinity/Documents/daydream/Day_Dream/backend/src/main.rs)

- Add a handler to serve static assets from the embedded folder.
- Implement SPA routing (fallback to `index.html` for unknown routes not starting with `/api`).

### Build Automation

#### [NEW] [build_release.ps1](file:///c:/Users/Trinity/Documents/daydream/Day_Dream/build_release.ps1)

- PowerShell script to:
    1. Run `trunk build --release` in `frontend`.
    2. Run `cargo build --release` in `backend`.
    3. Copy the final `backend.exe` to a `release` folder.
    4. Rename it to `Daydream.exe`.

## Verification Plan

### Automated Tests

- None for the build process itself, but the script will report success/failure.

### Manual Verification

1. Run `.\build_release.ps1`.
2. Navigate to the `release` folder.
3. Double-click `Daydream.exe`.
4. Open browser to `http://localhost:3000` (or whatever port is configured).
5. Verify the app loads without `trunk serve` running.
