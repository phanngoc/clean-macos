# clean-macos

Fast, safe, and efficient macOS system cleanup utilities.

## Overview

`Cache Cleaner App` là ứng dụng desktop dọn dẹp hệ thống macOS được xây dựng với Rust + Tauri. Ứng dụng cung cấp giao diện đồ họa trực quan và an toàn để quản lý và dọn dẹp các cache trên macOS.

![Cache Cleaner App Screenshot 1](./image1.png)

![Cache Cleaner App Screenshot 2](./image2.png)

## Features

Ứng dụng desktop dọn dẹp cache được xây dựng với Rust + Tauri, cung cấp giao diện đồ họa trực quan và an toàn:

- **📦 npm Cache** - Dọn dẹp thư mục `~/.npm`
- **🌐 Chrome Cache** - Dọn dẹp cache Chrome tại `~/Library/Caches/Google/Chrome`
- **📁 .cache Directory** - Dọn dẹp thư mục `~/.cache`
- **💾 IndexedDB Origins** - Quét và dọn dẹp IndexedDB của các website (>10MB)
- **📦 Large Caches** - Tìm và xóa các cache lớn (>1GB) trong `~/Library/Caches`
- **👁 Dry-run Preview** - Xem trước những gì sẽ bị xóa trước khi dọn dẹp
- **⚠️ Chrome Detection** - Cảnh báo nếu Chrome đang chạy
- **🔒 Permission Checks** - Kiểm tra quyền truy cập file system
- **📊 Progress Tracking** - Theo dõi tiến trình dọn dẹp với thanh progress bar
- **✅ Selective Cleaning** - Chọn lọc các mục cần dọn dẹp

## Quick Start

### Requirements

- macOS 10.15+ (Catalina trở lên)
- Rust 1.70+ (để build từ source)
- Node.js 18+ (cho Tauri CLI)

### Installation từ Source

1. Clone repository:
```bash
git clone https://github.com/yourusername/clean-macos.git
cd clean-macos/cache-cleaner-app
```

2. Cài đặt Tauri CLI:
```bash
cargo install tauri-cli
```

3. Chạy ở chế độ development:
```bash
cd src-tauri
cargo tauri dev
```

4. Build cho production:
```bash
cargo tauri build
```

File `.app` sẽ được tạo trong `src-tauri/target/release/bundle/macos/`

### Build Universal Binary (Intel + Apple Silicon)

```bash
# Thêm cả hai targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build universal binary
cargo tauri build --target universal-apple-darwin
```

### Sử dụng App

1. **Mở ứng dụng** - Khởi chạy Cache Cleaner từ Applications hoặc file `.app`

2. **Quét cache** - Nhấn nút "🔍 Scan" để tìm tất cả các cache lớn:
   - IndexedDB Origins (>10MB)
   - Large Caches (>1GB)

3. **Xem trước** - Nhấn "👁 Preview" để xem dung lượng sẽ được giải phóng (không xóa thực sự)

4. **Chọn mục cần dọn** - Tích chọn các mục bạn muốn xóa

5. **Dọn dẹp** - Nhấn "🗑 Clean Selected" và xác nhận để bắt đầu dọn dẹp

### Lưu ý

- ⚠️ **Đóng Chrome trước khi dọn Chrome cache** - App sẽ cảnh báo nếu Chrome đang chạy
- ⚠️ **Xóa IndexedDB sẽ xóa dữ liệu website** - Các website sẽ mất dữ liệu đã lưu
- ✅ **Luôn dùng Preview trước** - Để đảm bảo bạn không xóa nhầm dữ liệu quan trọng

## Documentation

### Cấu trúc Project

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
│   │   │   ├── cache_dir.rs  # .cache logic
│   │   │   ├── indexeddb.rs # IndexedDB scanning
│   │   │   └── large_caches.rs # Large cache detection
│   │   └── utils/
│   │       ├── filesystem.rs # File operations
│   │       └── permissions.rs# macOS permissions
│   ├── Cargo.toml
│   └── tauri.conf.json
└── ui/
    └── index.html            # Frontend UI
```

### Các loại Cache được hỗ trợ

| Loại Cache | Đường dẫn | Mô tả |
|------------|-----------|-------|
| **npm Cache** | `~/.npm` | Cache của npm packages |
| **Chrome Cache** | `~/Library/Caches/Google/Chrome` | Cache của Google Chrome |
| **.cache Directory** | `~/.cache` | Thư mục cache chung |
| **Large Extensions** | `~/Library/.../Chrome/Extensions/` | Chrome extensions lớn (>100MB) |
| **IndexedDB** | `~/Library/.../Chrome/.../IndexedDB/` | Dữ liệu IndexedDB của websites (>10MB) |
| **Large Caches** | `~/Library/Caches/*` | Các cache lớn trong Library/Caches (>1GB) |

### Windows 11

Trên Windows, mỗi loại cache có danh sách vị trí riêng (`src-tauri/src/cache/paths.rs`, module `platform`). Thư mục được làm rỗng (giữ lại thư mục gốc); file bị khóa / đang dùng / cần quyền admin sẽ được **bỏ qua và báo lại**, không làm hỏng cả lượt dọn.

| Loại (`type`) | Vị trí | Ghi chú |
|---|---|---|
| `TempFiles` | `%TEMP%` | Chỉ xóa mục không đổi trong 24h |
| `SystemCaches` | `C:\Windows\Temp` | Cần admin; chỉ mục cũ hơn 24h |
| `WindowsUpdate` | `C:\Windows\SoftwareDistribution\Download` | Cần admin |
| `ThumbnailCache` | `%LOCALAPPDATA%\Microsoft\Windows\Explorer\{thumbcache,iconcache}_*.db` | Explorer tự tạo lại |
| `ErrorReports` | `WER\{ReportArchive,ReportQueue,Temp}` (user + ProgramData) | |
| `CrashDumps` | `%LOCALAPPDATA%\CrashDumps`, `C:\Windows\Minidump`, `MEMORY.DMP` | |
| `ShaderCache` | `D3DSCache`, NVIDIA/AMD/Intel shader cache | |
| `RecycleBin` | `X:\$Recycle.Bin\<SID>` mọi ổ | Không khôi phục được |
| `Chrome`, `Edge` | `User Data\<profile>\{Cache,Code Cache,GPUCache,…}` + shader caches | Không đụng cookies/history |
| `Firefox` | `%LOCALAPPDATA%\Mozilla\Firefox\Profiles\*\{cache2,startupCache,thumbnails}` | |
| `VSCode`, `Cursor` | `%APPDATA%\{Code,Cursor}\{Cache,CachedData,Code Cache,GPUCache,logs,…}` | Không đụng settings/extensions |
| `Npm`, `Yarn`, `Pnpm`, `Pip`, `NuGet`, `Gradle`, `Cargo`, `GoBuild`, `Uv`, `CacheDir` | `%LOCALAPPDATA%\npm-cache`, `Yarn\Cache`, `pnpm\store`, `pip\cache`, `~\.nuget\packages`, `~\.gradle\caches`, `~\.cargo\registry`, `go-build`, `uv\cache`, `~\.cache` | Tải lại khi cần |
| `WslSystemCaches` | Trong mọi distro WSL 2: `/var/cache/apt/archives/*.deb`, journal đã archive >7 ngày, log đã rotate, `/tmp` `/var/tmp` >24h, `/var/crash` | Chạy bằng `wsl -u root`, không cần sudo |
| `WslUserCaches` | `~/.cache`, `~/.npm/_cacache` của root và mọi user trong `/home` | Có thể chứa model/browser tải sẵn |
| `DockerBuildCache`, `DockerImages`, `DockerContainers`, `DockerVolumes` | `docker builder prune -af`, `image prune -af`, `container prune -f`, `volume prune -f` | Dùng `docker` của Windows, hoặc tự chuyển sang Docker Engine trong distro WSL mặc định |

**WSL trả dung lượng về Windows thế nào:** mỗi distro là một file `ext4.vhdx`; xóa file bên trong không làm nhỏ file này trừ khi VHDX là *sparse*. Sau mỗi lượt dọn WSL/Docker, app chạy `fstrim /` trong distro để trả block trống về Windows. Kiểm tra / bật sparse:

```powershell
fsutil sparse queryflag "$env:LOCALAPPDATA\wsl\{<guid>}\ext4.vhdx"
wsl --shutdown; wsl --manage <Distro> --set-sparse true
```

### CLI (không cần webview)

```bash
cd cache-cleaner-app/src-tauri
cargo build --release --no-default-features --bin cache-cleaner-cli

target/release/cache-cleaner-cli scan                      # liệt kê cache + dung lượng
target/release/cache-cleaner-cli clean --dry-run tempfiles  # xem trước
target/release/cache-cleaner-cli clean tempfiles gobuild chrome edge
```

`--no-default-features` bỏ feature `gui` (Tauri), nên build CLI và chạy test nhẹ hơn nhiều: `cargo test --no-default-features`.

### Tính năng Safety

- **Dry-run Mode**: Xem trước dung lượng sẽ được giải phóng mà không xóa thực sự
- **Chrome Detection**: Tự động phát hiện và cảnh báo nếu Chrome đang chạy
- **Permission Checks**: Kiểm tra quyền truy cập file system trước khi thực hiện
- **Confirmation Dialog**: Yêu cầu xác nhận trước khi xóa (đặc biệt với extensions và IndexedDB)
- **Progress Tracking**: Hiển thị thanh tiến trình khi đang dọn dẹp
- **Error Handling**: Xử lý lỗi an toàn và thông báo rõ ràng

### Performance

- **Quét nhanh**: Sử dụng Rust async để quét cache hiệu quả
- **Xóa song song**: Xử lý nhiều mục đồng thời khi có thể
- **UI responsive**: Giao diện không bị đơ trong quá trình xử lý

## Requirements

- macOS 10.15+ (Catalina trở lên) hoặc Windows 10/11 (WebView2)
- Rust 1.70+ (để build từ source)
- Node.js 18+ (cho Tauri CLI)

## Compatibility

- ✅ Apple Silicon (M1/M2/M3)
- ✅ Intel Macs
- ✅ macOS Monterey, Ventura, Sonoma
- ✅ APFS and HFS+ file systems
- ✅ External drives (USB, Thunderbolt)

## Safety & Best Practices

### Before First Use

1. **Luôn dùng Preview trước** - Nhấn nút "👁 Preview" để xem dung lượng sẽ được giải phóng
2. **Đóng Chrome trước khi dọn** - App sẽ cảnh báo nếu Chrome đang chạy
3. **Kiểm tra kỹ các mục được chọn** - Đặc biệt với Extensions và IndexedDB
4. **Bắt đầu với ít mục** - Test với một vài mục nhỏ trước

### Production Use

- Luôn sử dụng Preview mode trước khi dọn dẹp thực sự
- Đóng Chrome trước khi dọn Chrome cache
- Xóa IndexedDB sẽ làm mất dữ liệu website - cẩn thận với các website quan trọng
- Kiểm tra quyền truy cập file system nếu gặp lỗi permission

### Known Limitations

- Chrome phải được đóng trước khi dọn Chrome cache
- Xóa IndexedDB sẽ làm mất dữ liệu đã lưu của websites
- Spotlight có thể reindex sau khi xóa cache lớn

## Contributing

Contributions are welcome! This is an early-stage project with room for expansion.

### Development

To contribute:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Test thoroughly on macOS
5. Submit a pull request


## License

MIT License - see LICENSE file for details

## Acknowledgments

Inspired by the need for efficient cache management on macOS, especially with large cache folders that can take up significant disk space.

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/clean-macos/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/clean-macos/discussions)

## Changelog

### Unreleased

- ✨ Hỗ trợ Windows 11: Temp, Windows Update, thumbnail cache, WER, crash dumps, shader cache, Recycle Bin, Edge, NuGet, Go build cache, uv
- ✨ CLI `cache-cleaner-cli` (`scan`, `types`, `clean [--dry-run]`)
- ✨ WSL 2: dọn cache hệ thống / user trong distro + `fstrim` để VHDX sparse trả dung lượng về Windows
- ✨ Docker: tự dùng Docker Engine trong WSL khi Windows không có `docker.exe`; prune theo loại từ CLI
- ♻️ Scanner/cleaner dùng chung một catalog vị trí theo nền tảng; xóa best-effort (bỏ qua file bị khóa thay vì dừng)

### v1.0.0 (Initial Release)

- ✨ Cache Cleaner Desktop App với giao diện đồ họa
- ✨ Hỗ trợ dọn dẹp npm, Chrome, và .cache directories
- ✨ Quét và dọn dẹp IndexedDB origins (>10MB)
- ✨ Tìm và xóa Large Caches (>1GB)
- ✨ Dry-run preview mode
- ✨ Chrome running detection
- ✨ Progress tracking với progress bar
- ✨ Selective cleaning với checkboxes
- ✨ Apple Silicon và Intel Mac compatibility

---

**Made with ❤️ for macOS power users**
