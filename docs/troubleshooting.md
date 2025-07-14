# Troubleshooting Guide

## Common Issues and Solutions

### Build Issues

#### "trunk: command not found"
**Solution**: Install trunk with:
```bash
cargo install trunk
```

#### WASM target not installed
**Solution**: Add the WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

#### Build fails with OpenSSL errors
**Solution**: This often happens on macOS. Install OpenSSL:
```bash
brew install openssl
```

### Runtime Issues

#### Audio doesn't play

1. **Check browser console**: Look for Web Audio API errors
2. **Verify browser support**: Ensure Web Audio API is supported
3. **Check active voices**: Ensure voices aren't all muted
4. **Volume settings**: Check both overall and per-voice volume

#### Sheet music doesn't render

1. **Check OSMD loading**: Look for errors in console about `opensheetmusicdisplay.min.js`
2. **Verify file format**: Ensure MusicXML files are valid
3. **Check song data**: Use browser DevTools to verify song data is loaded

#### Keyboard controls not working

1. **Check focus**: Ensure the browser window has focus
2. **Modifier keys**: Ensure no modifier keys are pressed
3. **Key conflicts**: Check if browser extensions are intercepting keys

#### Mobile controls not showing

1. **Device detection**: Check if your device reports `hover: none` and `pointer: coarse`
2. **CSS loading**: Ensure `main.css` is loaded and compiled
3. **Browser compatibility**: Some browsers may not support these media queries

### Performance Issues

#### Laggy note playback

1. **Check CPU usage**: Web Audio can be CPU intensive
2. **Reduce active voices**: Try muting some voices
3. **Browser limitations**: Some mobile browsers have Web Audio limitations

#### Slow sheet music rendering

1. **Song complexity**: Very complex scores render slowly
2. **Zoom level**: High zoom levels impact performance
3. **Browser rendering**: Try different browsers

### State Management Issues

#### Controls not syncing

1. **Signal connections**: Verify signals are properly connected
2. **Effect dependencies**: Check that effects have correct dependencies
3. **Component hierarchy**: Ensure props are passed correctly

#### State not persisting

1. **LocalStorage**: Check if `use_local_storage` is working
2. **Signal types**: Ensure using appropriate signal types
3. **Reset triggers**: Check if something is resetting state

### Development Workflow Issues

#### Hot reload not working

1. **File watching**: Ensure `trunk serve` is running
2. **Browser cache**: Try hard refresh (Ctrl+Shift+R)
3. **Port conflicts**: Check if port 8080 is available

#### Changes not reflecting

1. **Cargo cache**: Try `cargo clean`
2. **Trunk cache**: Delete `dist/` directory
3. **Browser cache**: Clear browser cache

## Debugging Techniques

### 1. Console Logging

Add debug logging in Rust:
```rust
web_sys::console::log_1(&format!("Debug: {}", value).into());
```

### 2. Browser DevTools

- **Network tab**: Check if resources load correctly
- **Console**: Look for JavaScript errors
- **Elements**: Inspect generated HTML
- **Performance**: Profile for bottlenecks

### 3. Leptos DevTools

Use Leptos-specific debugging:
```rust
// Log signal updates
create_effect(move |_| {
    log!("Signal updated: {:?}", signal.get());
});
```

### 4. WASM Debugging

- Use browser WASM debugging tools
- Add source maps in development builds
- Use `wasm-bindgen` debug features

## Common Error Messages

### "No audio context available"
**Cause**: Browser security prevents audio context creation
**Fix**: Ensure audio is initiated by user interaction

### "Failed to compile vertex shader"
**Cause**: WebGL issues with OSMD rendering
**Fix**: Update graphics drivers or try different browser

### "Maximum call stack exceeded"
**Cause**: Infinite loop in effects or circular dependencies
**Fix**: Check effect dependencies and signal updates

### "Out of memory"
**Cause**: Large songs or memory leaks
**Fix**: Check for proper cleanup, reduce song complexity

## Platform-Specific Issues

### iOS Safari
- **Audio restrictions**: Requires user interaction to start
- **Touch events**: May behave differently than other browsers
- **Performance**: Web Audio can be limited

### Android Chrome
- **Performance**: Varies greatly by device
- **Touch handling**: Check pointer event support
- **Audio latency**: Can be higher than desktop

### Desktop vs Mobile
- **Input methods**: Test both keyboard and touch
- **Performance**: Mobile devices have less power
- **Screen size**: Ensure responsive design works

## Getting Help

If you're still stuck:

1. **Check existing issues**: Look at GitHub issues
2. **Browser console**: Always check for errors
3. **Minimal reproduction**: Create a simple test case
4. **Version information**: Note browser, OS, and package versions

### Information to Provide

When reporting issues, include:
- Browser and version
- Operating system
- Error messages from console
- Steps to reproduce
- Expected vs actual behavior