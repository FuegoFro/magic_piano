# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

- **Development server**: `./develop.sh` or `trunk serve`
  - Starts local dev server with hot reload at http://127.0.0.1:8080
  
- **Production build**: `./deploy.sh`
  - Builds and deploys to GitHub Pages
  - Uses `trunk build --public-url "https://fuegofro.github.io/magic_piano/" --no-sri`

- **Direct trunk commands**:
  - `trunk serve` - Development server
  - `trunk build` - Production build to dist/

## Architecture Overview

Magic Piano is a Rust/WebAssembly web application using Leptos for reactive UI. The app provides an interactive sheet music player with keyboard controls.

### Key Components

- **`src/components/app.rs`**: Main application component that orchestrates the UI, manages song loading, and coordinates between sheet music display and playback
- **`src/components/sheet_music.rs`**: Handles rendering of sheet music using OpenSheetMusicDisplay (OSMD) JavaScript library via WASM bindings
- **`src/playback_manager.rs`**: Manages audio playback timing, cursor position, and coordination between visual and audio elements
- **`src/sampler.rs`**: Audio synthesis using Web Audio API for playing notes
- **`src/opensheetmusicdisplay_bindings.rs`**: WASM bindings to the OSMD JavaScript library

### State Management

The app uses Leptos signals for reactive state management:
- Song selection and loading state
- Current playback position
- Voice mute/solo states
- Zoom level and display settings

### JavaScript Interop

The project integrates with OpenSheetMusicDisplay (vendored in `vendor/opensheetmusicdisplay/`) through `wasm-bindgen`. Key interop points:
- Loading MusicXML files into OSMD
- Cursor positioning and movement
- Voice visibility control
- Sheet music rendering options

### Audio Architecture

- Uses Web Audio API directly through `web-sys` bindings
- Implements custom sampler for note playback
- Supports multiple voices with individual volume/mute control
- Handles tied notes and musical timing

## Project Structure

```
src/
├── main.rs                           # Leptos app entry point
├── components/                       # UI components
│   ├── app.rs                       # Main app component
│   ├── keyboard_listener.rs        # Keyboard input handling
│   ├── sheet_music.rs             # OSMD integration
│   └── voice_control.rs           # Voice mute/solo controls
├── playback_manager.rs             # Audio playback coordination
├── sampler.rs                      # Web Audio synthesis
├── song_data.rs                    # Musical data structures
└── opensheetmusicdisplay_bindings.rs # OSMD WASM bindings
```

## Important Technical Details

- **Rust version**: 1.88.0 (specified in rust-toolchain.toml)
- **WASM target**: wasm32-unknown-unknown
- **Framework**: Leptos 0.8.2 in CSR (client-side rendering) mode
- **Build tool**: Trunk 0.21.14
- **Styling**: TailwindCSS with custom cursor colors

## Common Development Tasks

When modifying the UI:
- Components use Leptos `view!` macro for reactive templates
- State is managed through signals (`create_signal`, `create_rw_signal`)
- Effects use `create_effect` for side effects

When working with audio:
- Audio context is created in `sampler.rs`
- Playback timing is managed by `PlaybackManager`
- Note events come from parsed MusicXML data

When updating OSMD integration:
- Bindings are in `opensheetmusicdisplay_bindings.rs`
- OSMD library is vendored, not from npm
- JavaScript interop uses `wasm_bindgen`