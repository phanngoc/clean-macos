# Cache Cleaner for macOS

A fast, safe cache cleaner desktop app built with Rust + Tauri.

## Features

- 📦 **npm Cache** - Clean `~/.npm` directory
- 🌐 **Chrome Cache** - Clean `~/Library/Caches/Google/Chrome`
- 📁 **.cache Directory** - Clean `~/.cache`
- 👁 **Dry-run Preview** - See what will be deleted before cleaning
- ⚠️ **Chrome Detection** - Warns if Chrome is running
- 🔒 **Permission Checks** - Validates file system access

## Requirements

- macOS 10.15+
- Rust 1.70+
- Node.js 18+ (for Tauri CLI)

## Setup

```bash
# Install Tauri CLI
cargo install tauri-cli

# Build and run in development
cd src-tauri
cargo tauri dev

# Build for production
cargo tauri build
```

## Project Structure

```
cache-cleaner-app/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # Tauri entry + commands
│   │   ├── cache/
│   │   │   ├── mod.rs        # Cache types & structs
│   │   │   ├── scanner.rs    # Cache detection
│   │   │   ├── cleaner.rs    # Cache deletion
│   │   │   ├── npm.rs        # npm-specific logic
│   │   │   ├── chrome.rs     # Chrome-specific logic
│   │   │   └── cache_dir.rs  # .cache logic
│   │   └── utils/
│   │       ├── filesystem.rs # File operations
│   │       └── permissions.rs# macOS permissions
│   ├── Cargo.toml
│   └── tauri.conf.json
└── ui/
    └── index.html            # Frontend UI
```

## Building Universal Binary (Intel + Apple Silicon)

```bash
# Add both targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build universal binary
cargo tauri build --target universal-apple-darwin
```

## License

MIT
