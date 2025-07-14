# Adding Features to Magic Piano

This guide walks through common feature additions with examples from the codebase.

## Example: Adding Mobile Controls

The mobile controls feature demonstrates several important patterns for adding new UI components.

### 1. Create the Component Module

First, create a new component file and add it to the module exports:

```rust
// src/components/mobile_controls.rs
use leptos::prelude::*;
// ... other imports

#[component]
pub fn MobileControls(/* props */) -> impl IntoView {
    // Implementation
}
```

```rust
// src/components/mod.rs
mod mobile_controls;  // Add this line
```

### 2. Share State with Existing Components

Instead of creating new state, share existing signals:

```rust
// In app.rs - create shared state
let start_song_index = RwSignal::new(0);
let most_recent_song_index = RwSignal::new(0);

// Pass to both components
<KeyboardListener
    start_song_index=start_song_index
    most_recent_song_index=most_recent_song_index
    // ... other props
/>
<MobileControls
    start_song_index=start_song_index
    most_recent_song_index=most_recent_song_index
    // ... other props
/>
```

### 3. Reuse Existing Logic

Look for existing functionality to reuse:

```rust
// Mobile controls reuse the same playback logic as keyboard
let Some((cursor_index, newly_held_notes)) = playback_manager
    .write()
    .start_notes_at_relative_index(song_index, &*active_voices)
else {
    return;
};
```

### 4. Handle Platform-Specific Behavior

Use CSS for platform detection when possible:

```css
.mobile-controls {
    display: none;
}

@media (hover: none) and (pointer: coarse) {
    .mobile-controls {
        display: flex;
    }
}
```

## Common Feature Patterns

### Adding a New Control/Button

1. **Identify the action**: What state does it modify?
2. **Find existing handlers**: Is there similar functionality?
3. **Share state**: Use existing signals where possible
4. **Add the UI**: Follow existing component patterns
5. **Test interactions**: Ensure it works with existing features

### Adding Keyboard Shortcuts

1. Add to the `get_no_modifiers_key_action` function
2. Check for modifier keys to avoid conflicts
3. Decide if the action should repeat on key hold
4. Update the controls documentation in the UI

### Adding New Audio Features

1. Check if `PlaybackManager` needs modification
2. Consider the `SamplerPlaybackGuard` lifecycle
3. Ensure cleanup on component unmount
4. Test with different songs/voice configurations

### Adding Visual Feedback

1. Use derived signals for visual state
2. Apply CSS classes conditionally
3. Consider transitions for smooth UX
4. Test on both desktop and mobile

## Feature Development Workflow

### 1. Planning Phase

Before coding:
- Identify which components will be affected
- Determine what state needs to be shared
- Check for existing patterns to follow
- Consider mobile/desktop differences

### 2. Implementation Phase

Start with the simplest version:
1. Get basic functionality working
2. Add proper state management
3. Implement edge cases
4. Add visual polish

### 3. Integration Phase

Ensure the feature integrates well:
- Test with existing features
- Check for state conflicts
- Verify cleanup/unmounting
- Test keyboard and touch interactions

### 4. Polish Phase

Final touches:
- Add proper ARIA labels
- Ensure responsive design
- Add loading states if needed
- Update documentation

## Common Gotchas

### 1. Initial State Handling

Be careful with "first action" scenarios:

```rust
// Example: First "next" press should play note 0
let (has_moved_next, set_has_moved_next) = signal_local(false);

if has_moved_next.get() {
    // Normal increment
} else {
    // Special first-time behavior
    set_has_moved_next.set(true);
}
```

### 2. Cursor vs Song Index

Always be clear about which you're using:
- **Song Index**: Position in the song data (0-based)
- **Cursor Index**: Visual position in the sheet music

Use `PlaybackManager` methods to convert between them.

### 3. Voice Management

Voices can be muted/soloed dynamically:
- Always check `active_voices` before playing
- Voice indices are derived from their position
- The number of voices can vary by song

### 4. Resource Cleanup

Always clean up resources:
- Event listeners need `on_cleanup`
- Audio guards stop notes when dropped
- Effects automatically clean up their subscriptions

## Testing Your Feature

### Manual Testing Checklist

- [ ] Feature works as expected
- [ ] Keyboard controls still function
- [ ] Mobile controls work on touch devices
- [ ] No console errors
- [ ] State persists correctly
- [ ] Cleanup on unmount works
- [ ] Performance is acceptable

### Cross-Browser Testing

Test in:
- Chrome (desktop + mobile)
- Firefox
- Safari (macOS + iOS)
- Edge

### Device Testing

Test on:
- Desktop with mouse
- Desktop with touchscreen
- Tablet
- Phone
- Devices with both input methods