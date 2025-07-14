# Development Patterns and Best Practices

## Leptos-Specific Patterns

### 1. Signal Naming Conventions

The codebase follows a consistent pattern for signal naming:

```rust
// Read-only signals
let (value, set_value) = signal(initial_value);

// Read-write signals  
let rw_signal = RwSignal::new(initial_value);
```

**Best Practice**: Use read-write signals when multiple components need write access. Use split read/write signals when you want to control write access more carefully.

### 2. Component Props Pattern

When components need multiple related signals, pass them individually rather than creating wrapper structs:

```rust
#[component]
pub fn MobileControls(
    playback_manager: LocalResource<RwSignal<PlaybackManager, LocalStorage>>,
    #[prop(into)] active_voices: Signal<BitSet>,
    start_song_index: RwSignal<usize>,
    most_recent_song_index: RwSignal<usize>,
    set_current_cursor_index: WriteSignal<usize>,
) -> impl IntoView
```

**Rationale**: This makes dependencies explicit and allows for fine-grained reactivity.

### 3. Effect Cleanup Pattern

Always clean up event listeners and resources:

```rust
let keydown_handle = window_event_listener(ev::keydown, move |event| {
    // handler code
});
on_cleanup(move || keydown_handle.remove());
```

## State Management Patterns

### 1. Derived State

Use `Signal::derive` for computed state that depends on other signals:

```rust
let active_voices = Signal::derive(move || {
    voice_states
        .get()
        .into_iter()
        .filter(|(_, muted)| !muted.get())
        .collect::<BitSet>()
});
```

### 2. Local vs Global State

- **Local State**: Use `signal_local` for component-specific state
- **Global State**: Pass signals from App component as props
- **Resources**: Use for async data that multiple components need

### 3. State Synchronization

When state needs to be synchronized between components:

```rust
// In parent component
let start_song_index = RwSignal::new(0);
let most_recent_song_index = RwSignal::new(0);

// Pass to multiple children
<KeyboardListener start_song_index=start_song_index most_recent_song_index=most_recent_song_index />
<MobileControls start_song_index=start_song_index most_recent_song_index=most_recent_song_index />
```

## Event Handling Patterns

### 1. Keyboard Events

- Always check for modifier keys to avoid conflicts with browser shortcuts
- Prevent default behavior for handled keys
- Ignore key repeats for actions that shouldn't repeat

### 2. Touch/Pointer Events

Use pointer events for unified mouse/touch handling:

```rust
on:pointerdown=handle_press
on:pointerup=handle_release
on:pointercancel=handle_release  // Important for touch
on:pointerout=handle_release      // Clean up on pointer leave
```

### 3. Debouncing and Throttling

For frequently firing events, consider storing state locally:

```rust
let (is_pressed, set_is_pressed) = signal_local(false);
// Only act on state changes, not every event
```

## Rust/WASM Specific Patterns

### 1. Type Conversions

Be explicit about JavaScript type conversions:

```rust
// Converting binary data to JsString
let u16_data = data.iter().map(|d| *d as u16).collect_vec();
JsString::from_char_code(&u16_data)
```

### 2. Error Handling in WASM

Use Option/Result appropriately:

```rust
let Some(playback_manager) = &*playback_manager else {
    return;
};
```

### 3. JavaScript Interop

Keep JS interactions in dedicated modules:
- `opensheetmusicdisplay_bindings.rs` for OSMD
- `sampler.rs` for Web Audio API

## CSS and Styling Patterns

### 1. Responsive Design

Use CSS media queries for capability detection, not screen size:

```css
/* Good: Detect touch capability */
@media (hover: none) and (pointer: coarse) { }

/* Avoid: Screen size alone */
@media (max-width: 768px) { }
```

### 2. Tailwind Usage

- Use Tailwind classes for common styles
- Create custom CSS classes for complex media queries
- Keep component-specific styles in the component file

### 3. Theme Consistency

Define colors in Tailwind config and reference them consistently:
```rust
class="bg-gray-700 hover:bg-gray-600 active:bg-gray-500"
```

## Testing Strategies

### 1. Manual Testing Checklist

When adding new features:
- [ ] Test keyboard functionality still works
- [ ] Test touch/mobile functionality
- [ ] Test with different songs (varying number of voices)
- [ ] Test state persistence across song changes
- [ ] Test cleanup (no memory leaks, orphaned audio)

### 2. Cross-Device Testing

Critical paths to test:
- Desktop with keyboard
- Mobile/tablet with touch
- Hybrid devices (both input methods)
- Different browsers (Chrome, Firefox, Safari)

## Performance Guidelines

### 1. Minimize Re-renders

- Use memos for expensive computations
- Split signals to avoid unnecessary updates
- Use local state when global isn't needed

### 2. Resource Management

- Clean up event listeners
- Drop audio guards when notes stop
- Avoid keeping large data in signals if derived works

### 3. Bundle Size

- The OSMD library is vendored to control version
- Consider lazy loading for large features
- Use Rust's dead code elimination effectively