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

## AdSense Integration Setup

This application uses Google AdSense for displaying ads. To set up AdSense:

### 1. Create Google AdSense Account

1. Go to [Google AdSense](https://www.google.com/adsense/)
2. Sign in with your Google account
3. Create a new AdSense account
4. Complete the account verification process

### 2. Register Your Application

1. In AdSense dashboard, go to **Sites** → **Add site**
2. Add your application's domain/identifier
3. Wait for AdSense approval (can take 1-2 weeks)

### 3. Create Ad Units

1. Go to **Ads** → **By ad unit** → **Create new ad unit**
2. Choose **Display ads** or **Video ads** (for 15-second rewarded videos)
3. Configure ad size and format
4. Copy your **Publisher ID** (format: `ca-pub-XXXXXXXXXX`)
5. Copy your **Ad Unit ID** (format: `ca-app-pub-XXXXXXXXXX/XXXXXXXXXX`)

### 4. Configure API Keys

1. Open `cache-cleaner-app/ui/index.html`
2. Find the `ADSENSE_CONFIG` object (around line 330)
3. Replace the placeholder values:
   ```javascript
   const ADSENSE_CONFIG = {
     publisherId: 'ca-pub-YOUR_PUBLISHER_ID',
     adUnitId: 'ca-app-pub-YOUR_PUBLISHER_ID/YOUR_AD_UNIT_ID',
     testAdUnitId: 'ca-app-pub-3940256099942544/5224354917', // Keep test ID
     environment: 'development', // Change to 'production' when ready
     adDurationSeconds: 15
   };
   ```
4. Update the AdSense script tag in the `<head>` section:
   ```html
   <script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pub-YOUR_PUBLISHER_ID"
      crossorigin="anonymous"></script>
   ```

### 5. Test Ad Loading

1. In development, the app uses Google's test ad unit (already configured)
2. Test ad loading by calling `testAdLoading()` in the browser console
3. Verify ads load correctly before switching to production

### 6. Production Deployment

1. Change `ADSENSE_CONFIG.environment` to `'production'`
2. Ensure your AdSense account is approved
3. Use your production ad unit IDs
4. Test thoroughly before release

### Important Notes

- **Never commit API keys to version control** - The `.gitignore` already excludes `.env` files
- **Use test ad units during development** - Google provides test ad units that don't generate revenue
- **AdSense approval required** - Your account must be approved before ads will serve in production
- **Privacy compliance** - Ensure your app complies with GDPR, CCPA, and other privacy regulations
- **Ad blocking** - Some users may have ad blockers that prevent ads from loading

### Troubleshooting

- **Ads not loading**: Check browser console for errors, verify API keys are correct
- **Test ads work but production doesn't**: Ensure AdSense account is approved
- **CSP errors**: Check `tauri.conf.json` CSP settings (currently set to `null` for development)

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
