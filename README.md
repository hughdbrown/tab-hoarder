# Tab Hoarder

A Chrome extension for managing and organizing browser tabs, built with Rust, WebAssembly, and Yew.

## Features

- **Analyze Domains**: Display top 10 most frequently occurring domains
- **Sort Tabs by Domain**: Organize tabs alphabetically by domain
- **Make Tabs Unique**: Remove duplicate tabs by URL
- **Collapse Tabs**: Save tabs to storage and close them (memory saver)
- **Restore Tabs**: Restore entire sessions or individual tabs
- **Session Management**: View, search, edit, delete, and export collapsed sessions

## Technology Stack

- **Rust** - Core logic and business rules
- **Yew** - UI framework (React-like components)
- **WebAssembly** - Compiled Rust for browser execution
- **Chrome Extension API** - Tab and storage management
- **JavaScript** - Minimal glue code for Chrome APIs

## Project Status

### ✅ Completed Components

1. **Core Rust Logic** (with 22 passing unit tests)
   - Smart domain extraction (handles .co.uk, .com.au, etc.)
   - Domain counting and top-N selection
   - Tab sorting by domain
   - Tab uniqueness detection
   - Storage serialization structures

2. **Build Infrastructure**
   - Cargo.toml with all dependencies
   - build.sh script for WASM compilation
   - Test runner (`cargo test`)

3. **Extension Files**
   - manifest.json (Chrome Extension V3)
   - HTML pages (popup.html, collapsed.html)
   - Background service worker
   - Extension icons (SVG-based)

4. **JavaScript Bridge**
   - Chrome tabs API with batch processing (50 tabs/chunk)
   - Chrome storage API wrapper
   - Progress callback support
   - Storage quota monitoring

### 🚧 Remaining Work

1. **Yew UI Components** (the main remaining task)
   - Popup UI with domain analyzer and action buttons
   - Progress indicators (progress bar + spinner)
   - Collapsed tabs viewer page
   - Search/filter functionality
   - Session editing (rename)
   - Export functionality
   - Storage quota warnings

2. **Integration**
   - Wire up Rust logic to Yew components
   - Connect Yew to JavaScript bridge
   - Handle async operations and state management

3. **Testing**
   - Manual testing in Chrome with various tab counts
   - Performance validation with 1000+ tabs
   - Edge case handling

## Installation

### Prerequisites

```bash
# Install Rust and wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack
```

### Building

```bash
# Run tests
cargo test

# Build WASM
./build.sh
```

### Loading in Chrome

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" (top right toggle)
3. Click "Load unpacked"
4. Select this directory

After making changes:
- Run `./build.sh`
- Click reload icon in Chrome extensions page

## Architecture

### Three-Layer Design

```
┌─────────────────────────────────────────┐
│           Yew UI Components             │
│    (Popup, Viewer, Progress, etc.)      │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│         Rust Core Logic (WASM)          │
│  • Domain extraction                    │
│  • Tab operations (sort, unique)        │
│  • Storage structures                   │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│       JavaScript Bridge (Glue)          │
│  • chrome.tabs API (batch operations)   │
│  • chrome.storage.local API             │
│  • Progress callbacks                   │
└─────────────────────────────────────────┘
```

### Domain Extraction Algorithm

Smart TLD handling for international domains:

```
https://www.google.com       → google.com
https://ai.microsoft.com     → microsoft.com
https://news.bbc.co.uk       → bbc.co.uk
https://shop.example.com.au  → example.com.au
```

### Batch Processing

All tab operations process in chunks of 50 to prevent UI freezing:

1. Split tabs into chunks
2. Process chunk (parallel operations)
3. Update progress indicator
4. Yield control to browser (`setTimeout(0)`)
5. Repeat for next chunk

### Storage Format

Sessions stored in `chrome.storage.local`:

```json
{
  "sessions": [
    {
      "id": "uuid-v4",
      "name": "Research 2024-10-28T14:30:00",
      "timestamp": 1698508200000,
      "tabs": [
        {
          "url": "https://example.com",
          "title": "Example",
          "domain": "example.com",
          "pinned": false
        }
      ]
    }
  ]
}
```

## Development

### File Structure

```
tab-hoarder/
├── src/
│   ├── lib.rs              # WASM entry, exports
│   ├── domain.rs           # Domain extraction (tested)
│   ├── tab_data.rs         # Data structures (tested)
│   ├── operations.rs       # Tab operations (tested)
│   ├── storage.rs          # Storage utils (tested)
│   └── ui/
│       └── mod.rs          # UI components (TODO)
│
├── popup.html              # Extension popup
├── popup.js                # Chrome API bridge
├── collapsed.html          # Collapsed tabs viewer
├── collapsed.js            # Viewer API bridge
├── background.js           # Service worker
├── manifest.json           # Extension manifest
├── build.sh                # Build script
├── Cargo.toml              # Rust dependencies
└── icons/                  # Extension icons
```

### Adding Features

1. Add Rust logic in appropriate module
2. Write unit tests (`cargo test`)
3. Add WASM export in `lib.rs`
4. Add JS bridge function if needed
5. Create/update Yew component
6. Test in Chrome

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test domain::tests

# With output
cargo test -- --nocapture
```

## Next Steps

The extension is **nearly complete**! The remaining work is primarily:

1. **Building Yew UI components** - The Rust logic and JS bridge are done
2. **Wiring everything together** - Connect the layers
3. **Testing and polish** - Manual testing with real tab loads
4. **User registration** - Add user profile editing / registration that is linked to an external server
5. **Payment feature** - Add Rust code to integrate with payment (polar.sh or Stripe)
6. **Tab management features** - Expand the tab management features
7. **CI/CD** - add steps in `.github` to perform tests on push
8. **Registration server** - add registration server with database to manage users
9. **Deployment** - Add `.github` deployment step for registration server
10. **Cosmetic** - Reduce the size of compiled code
11. **Publish extension** - Create user docs for publication, push extension to Chrome extension store

## License

MIT
