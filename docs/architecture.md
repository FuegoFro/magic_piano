# Magic Piano Architecture

## Overview

Magic Piano is a Rust/WebAssembly application that provides an interactive sheet music player with keyboard controls. The application uses Leptos for reactive UI and integrates with OpenSheetMusicDisplay (OSMD) for rendering musical notation.

## Key Architectural Patterns

### 1. Signal-Based State Management

The application extensively uses Leptos signals for state management:

```rust
// Example from app.rs
let start_song_index = RwSignal::new(0);
let most_recent_song_index = RwSignal::new(0);
```

**Key Learning**: When multiple components need to share state (like `KeyboardListener` and `MobileControls`), create the signals in the parent component (`App`) and pass them down as props.

### 2. Resource Management for Async Operations

The app uses `LocalResource` for managing async operations:

```rust
let playback_manager: LocalResource<RwSignal<PlaybackManager, LocalStorage>> =
    LocalResource::new(|| async { RwSignal::new_local(PlaybackManager::initialize().await) });
```

**Key Learning**: Resources in Leptos handle the loading state automatically. Use `Signal::derive` to create loading indicators based on resource availability.

### 3. JavaScript Interop via WASM Bindings

The project integrates with JavaScript libraries through `wasm-bindgen`:

```rust
// From opensheetmusicdisplay_bindings.rs
#[wasm_bindgen]
extern "C" {
    pub type OpenSheetMusicDisplay;
    // ...
}
```

**Key Learning**: When integrating with JS libraries, create a dedicated bindings module to encapsulate all the external calls.

## Component Hierarchy

```
App
├── KeyboardListener (handles keyboard input)
├── SheetMusic (renders musical notation via OSMD)
├── VoiceControl (manages individual voice settings)
└── MobileControls (touch-friendly navigation)
```

### Component Communication Patterns

1. **Shared State via Props**: Components like `KeyboardListener` and `MobileControls` receive the same signals as props to stay synchronized.

2. **Effects for Cross-Component Updates**: Use `create_effect` to update related state:
   ```rust
   create_effect(move |_| {
       set_start_cursor_index.set(start_song_index.get());
   });
   ```

3. **Trigger Pattern for Resets**: The `Trigger` type is used to notify components of events like song changes without passing data.

## Audio Architecture

The audio system consists of three main components:

1. **Sampler**: Low-level Web Audio API interface for playing individual notes
2. **PlaybackManager**: Coordinates between visual cursor position and audio playback
3. **SamplerPlaybackGuard**: RAII pattern for managing note lifetime (notes stop when guard is dropped)

**Key Learning**: The guard pattern ensures notes don't play indefinitely if the UI fails to send a stop signal.

## Mobile/Touch Support Patterns

The mobile controls demonstrate several important patterns:

1. **Touch Event Handling**: Use `pointerdown`/`pointerup` instead of touch-specific events for better compatibility
2. **State Management for Hold Actions**: Store active guards in a HashMap to manage held notes
3. **CSS-Based Device Detection**: Use media queries rather than JS device detection:
   ```css
   @media (hover: none) and (pointer: coarse) {
       .mobile-controls { display: flex; }
   }
   ```

## Performance Considerations

1. **Memo for Computed Values**: Use `Memo` for expensive computations that depend on signals
2. **Local Signals**: Use `signal_local` for component-local state that doesn't need reactivity outside the component
3. **Batch Updates**: Group related state updates to minimize re-renders

## Common Pitfalls and Solutions

1. **Cursor Index vs Song Index**: The codebase maintains both cursor positions (visual) and song indices (data). Always use the PlaybackManager to convert between them.

2. **Voice Indexing**: Voices are indexed by their position in a flattened list, not by staff/voice IDs. Use `VoiceIndexMapping` for conversions.

3. **Initial State Handling**: Be careful with "first action" scenarios (like the first "next" button press). Consider using flags to track initialization state.