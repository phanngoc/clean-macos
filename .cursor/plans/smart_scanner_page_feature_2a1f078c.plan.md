---
name: Smart Scanner Page Feature
overview: Tạo một tab/page riêng cho Smart Scanner với khả năng hiển thị và xóa các folder được đề xuất, giúp user dễ dàng dọn dẹp máy tính.
todos:
  - id: backend-whitelist-paths
    content: "Cập nhật SCAN_LOCATIONS thành WHITELIST_PATHS với đầy đủ paths: ~/Library/Caches, ~/Library/Logs, ~/Library/Application Support/*/Cache, ~/Library/Containers/*/Data/Library/Caches, ~/.npm, ~/.yarn, ~/.cache, ~/Library/Developer/Xcode/DerivedData"
    status: done
  - id: backend-feature-structs
    content: Thêm FolderFeatures struct và LocationType enum để extract và store features (size_mb, last_accessed_days, created_days, location_type, depth)
    status: done
  - id: backend-path-helpers
    content: Implement expand_wildcard_paths(), determine_location_type(), calculate_folder_depth() để support whitelist paths với wildcards
    status: done
  - id: backend-extract-features
    content: Implement extract_folder_features() để collect size, age, location, depth từ folder path
    status: done
  - id: backend-scoring-algorithm
    content: "Refactor analyze_folder() với scoring algorithm mới: size_score * 0.4 + age_score * 0.4 + location_score * 0.2, với thresholds (5GB/1GB/500MB cho size, 180/90/30 days cho age)"
    status: done
  - id: backend-reasons-engine
    content: "Implement generate_reasons() để tạo reasons từ FolderFeatures với format: 'Large size: X GB', 'Not accessed for X days', 'Located in Cache directory', etc."
    status: done
  - id: backend-clean-result
    content: "Thêm struct SmartSuggestionsCleanResult vào smart_suggestions.rs với fields: total_freed_bytes, items_removed, success, message"
    status: done
  - id: backend-remove-function
    content: Thêm function remove_suggested_folders() trong smart_suggestions.rs để xóa folders và tính freed bytes, với error handling
    status: done
  - id: backend-command
    content: Thêm Tauri command remove_smart_suggestions trong main.rs và register trong invoke_handler
    status: done
  - id: frontend-tab-navigation
    content: Thêm tab navigation UI (Cache Cleaner | Smart Scanner) ở đầu index.html với JavaScript để switch tabs
    status: done
  - id: frontend-smart-scanner-section
    content: Tạo section HTML cho Smart Scanner với filter controls, scan button, folder list container, delete button, và progress bar
    status: done
  - id: frontend-scan-logic
    content: Implement JavaScript function scanSmartSuggestions() để gọi scan_smart_suggestions và render folder list với đầy đủ thông tin (name, path, size, score, reasons, last accessed)
    status: done
  - id: frontend-render-folders
    content: Implement renderSmartScannerFolders() để hiển thị folder items với checkbox, formatted size, score badge, reasons tags, và last accessed info
    status: done
  - id: frontend-delete-logic
    content: Implement deleteSmartScannerFolders() với confirmation dialog, progress tracking, và gọi remove_smart_suggestions command
    status: done
  - id: frontend-styling
    content: Thêm CSS styling cho tab navigation, score badges, reasons tags, và Smart Scanner section để match với design hiện tại
    status: done
---

# Smart Scanner Page Feature

## Tổng quan

Thêm một tab/page riêng cho Smart Scanner trong ứng dụng Cache Cleaner, cho phép user:

- Xem danh sách các folder được đề xuất xóa (dựa trên size, age, location)
- Hiển thị thông tin chi tiết: score, size, reasons, last accessed
- Chọn và xóa các folder được đề xuất
- Filter theo min size và max age

## Kiến trúc

```javascript
Frontend (index.html)
├── Tab Navigation (Cache Cleaner | Smart Scanner)
├── Cache Cleaner Tab (existing)
└── Smart Scanner Tab (new)
    ├── Scan button với filter options
    ├── Folder list với checkboxes
    ├── Delete selected button
    └── Progress tracking

Backend (main.rs)
├── scan_smart_suggestions (existing)
└── remove_smart_suggestions (new)
    └── Uses filesystem::remove_dir_all
```



## Core Algorithm: Smart Scoring Model (MVP)

### 3.1 Candidate Selection (Rule-based Whitelist Paths)

**KHÔNG scan toàn ổ đĩa** - Chỉ scan theo whitelist paths (80% giá trị – 20% effort):

```rust
const WHITELIST_PATHS: &[&str] = &[
    "~/Library/Caches/",
    "~/Library/Logs/",
    "~/Library/Application Support/*/Cache",
    "~/Library/Containers/*/Data/Library/Caches",
    "~/.npm",
    "~/.yarn",
    "~/.cache",
    "~/Library/Developer/Xcode/DerivedData",
];
```

**Implementation:**

- Expand wildcards (`*`) trong paths để tìm tất cả matching folders
- Chỉ scan direct children của whitelist paths (không recursive deep scan)
- Skip files, chỉ process directories

### 3.2 Feature Extraction

**Struct: `FolderFeatures`** (internal, không serialize ra frontend):

```rust
struct FolderFeatures {
    path: PathBuf,
    size_mb: u64,
    last_accessed_days: u64,
    created_days: u64,
    location_type: LocationType,
    depth: u8,
}

enum LocationType {
    Cache,      // ~/Library/Caches, ~/.cache, etc.
    Log,        // ~/Library/Logs
    Dev,        // Xcode DerivedData
    AppSupport, // ~/Library/Application Support/*/Cache
    Unknown,
}
```

**Feature Collection:**

- `size_mb`: Calculate từ `filesystem::calculate_dir_size()`
- `last_accessed_days`: Từ `access_tracker::days_since_access()` (fallback to `modified_time` nếu macOS chặn atime)
- `created_days`: Từ metadata creation time (hoặc modified nếu không có)
- `location_type`: Determine từ path pattern matching
- `depth`: Count path components từ whitelist root

### 3.3 Scoring Algorithm (Deterministic MVP)

**Công thức:**

```javascript
score = size_score * 0.4 + age_score * 0.4 + location_score * 0.2
```

**Size Score (0.0 - 1.0):**

```rust
fn calculate_size_score(size_mb: u64) -> f64 {
    if size_mb >= 5120 {  // > 5GB
        1.0
    } else if size_mb >= 1024 {  // > 1GB
        0.7
    } else if size_mb >= 500 {  // > 500MB
        0.4
    } else {
        (size_mb as f64 / 500.0) * 0.4  // Linear interpolation
    }
}
```

**Age Score (0.0 - 1.0):**

```rust
fn calculate_age_score(last_accessed_days: u64) -> f64 {
    if last_accessed_days >= 180 {
        1.0
    } else if last_accessed_days >= 90 {
        0.6
    } else if last_accessed_days >= 30 {
        0.3
    } else {
        (last_accessed_days as f64 / 30.0) * 0.3
    }
}
```

**Location Score (0.0 - 1.0):**

```rust
fn calculate_location_score(location_type: &LocationType) -> f64 {
    match location_type {
        LocationType::Cache | LocationType::Log | LocationType::Dev => 1.0,
        LocationType::AppSupport => 0.6,
        LocationType::Unknown => 0.2,
    }
}
```

**Final Score:** Normalize về range [0.0, 1.0] để hiển thị trên UI

### 3.4 Reasons Engine (Critical for UX)

**Reasons Generation:**

```rust
fn generate_reasons(features: &FolderFeatures) -> Vec<String> {
    let mut reasons = Vec::new();
    
    // Size reason
    if features.size_mb >= 1024 {
        reasons.push(format!("Large size: {:.1} GB", features.size_mb as f64 / 1024.0));
    } else {
        reasons.push(format!("Size: {} MB", features.size_mb));
    }
    
    // Age reason
    if features.last_accessed_days >= 180 {
        reasons.push(format!("Not accessed for {} days", features.last_accessed_days));
    } else if features.last_accessed_days >= 90 {
        reasons.push(format!("Not accessed for {} days", features.last_accessed_days));
    }
    
    // Location reason
    match features.location_type {
        LocationType::Cache => reasons.push("Located in Cache directory".to_string()),
        LocationType::Log => reasons.push("Located in Log directory".to_string()),
        LocationType::Dev => reasons.push("Development cache".to_string()),
        _ => {}
    }
    
    reasons
}
```

**Reasons Formatting:**

- Mỗi reason là một string ngắn gọn, dễ hiểu
- Hiển thị trên UI dạng tags/chips
- Quan trọng để user quyết định có dám Delete hay không

## Implementation Details

### Backend Changes

**File: [src-tauri/src/main.rs](cache-cleaner-app/src-tauri/src/main.rs)**

- Thêm Tauri command `remove_smart_suggestions` để xóa các folder được chọn
- Command nhận `Vec<String>` paths và trả về `SmartSuggestionsCleanResult`
- Sử dụng `std::fs::remove_dir_all` để xóa folder (tương tự `remove_large_caches`)

**File: [src-tauri/src/cache/smart_suggestions.rs](cache-cleaner-app/src-tauri/src/cache/smart_suggestions.rs)****Refactor existing code:**

1. **Update `SCAN_LOCATIONS`** → `WHITELIST_PATHS` với đầy đủ paths theo spec
2. **Add `FolderFeatures` struct** và `LocationType` enum (internal use)
3. **Add helper functions:**

- `expand_wildcard_paths()`: Expand `*` trong paths
- `determine_location_type(path: &Path) -> LocationType`
- `calculate_folder_depth(path: &Path, root: &Path) -> u8`
- `extract_folder_features(path: &PathBuf) -> Result<FolderFeatures>`

4. **Refactor `analyze_folder()`:**

- Extract features trước
- Tính scores theo công thức mới (40/40/20)
- Generate reasons từ features
- Normalize score về [0.0, 1.0]

5. **Add struct `SmartSuggestionsCleanResult`:**
   ```rust
      pub struct SmartSuggestionsCleanResult {
          pub total_freed_bytes: u64,
          pub items_removed: usize,
          pub success: bool,
          pub message: String,
      }
   ```




6. **Add function `remove_suggested_folders()`:**

- Nhận `Vec<String>` paths
- Calculate size trước khi delete
- Remove folders với error handling
- Return `SmartSuggestionsCleanResult`

### Frontend Changes

**File: [ui/index.html](cache-cleaner-app/ui/index.html)**

- Thêm tab navigation UI ở đầu trang
- Ẩn/hiện các sections dựa trên tab được chọn
- Thêm section mới cho Smart Scanner với:
- Filter controls (min size MB, max age days) - có thể là input fields hoặc defaults
- Scan button để gọi `scan_smart_suggestions`
- Danh sách folders với:
    - Checkbox để chọn
    - Folder name và path
    - Size (formatted)
    - Score (hiển thị dạng badge/indicator)
    - Reasons (list các lý do)
    - Last accessed (nếu có)
- Delete selected button
- Progress bar cho scan và delete operations
- Total size display

## Data Flow

1. User click "Smart Scanner" tab → hiển thị Smart Scanner section
2. User click "Scan" → gọi `scan_smart_suggestions(min_size_mb, max_age_days)` → hiển thị danh sách
3. User chọn folders → click "Delete Selected" → confirmation dialog → gọi `remove_smart_suggestions(paths)` → progress tracking → refresh list

## UI/UX Considerations

- Score hiển thị dạng badge với màu sắc (cao = đỏ, thấp = vàng)
- Reasons hiển thị dạng tags/chips
- Folder items có styling tương tự các section khác để consistency
- Progress bar cho cả scan và delete operations
- Confirmation dialog trước khi xóa (giống các section khác)
- Auto-refresh sau khi xóa thành công

## Algorithm Mapping với Code Hiện Tại

### Current State Analysis

**File: [smart_suggestions.rs](cache-cleaner-app/src-tauri/src/cache/smart_suggestions.rs)****Hiện tại:**

- `SCAN_LOCATIONS`: Chỉ có 4 paths cơ bản
- `analyze_folder()`: Scoring đơn giản (40% size, 30% age, 20% location, 10% item_count)
- Reasons: Basic, chưa có format chuẩn
- Chưa có `FolderFeatures` struct
- Chưa có location type classification

**Cần thay đổi:**

1. Mở rộng `SCAN_LOCATIONS` → `WHITELIST_PATHS` với wildcard support
2. Refactor scoring để match spec mới (40/40/20, không có item_count)
3. Thêm `FolderFeatures` extraction layer
4. Cải thiện reasons generation
5. Normalize score về [0.0, 1.0]

### Migration Strategy

1. **Phase 1: Add new structures** (không breaking changes)

- Add `FolderFeatures`, `LocationType`
- Add helper functions (expand_wildcard_paths, etc.)

2. **Phase 2: Refactor scoring** (backward compatible)

- Update `analyze_folder()` để dùng `FolderFeatures`
- Update scoring formula
- Update reasons generation

3. **Phase 3: Update scan paths**

- Replace `SCAN_LOCATIONS` với `WHITELIST_PATHS`
- Add wildcard expansion logic

4. **Phase 4: Add delete functionality**

- Add `remove_suggested_folders()`
- Add `SmartSuggestionsCleanResult`
- Add Tauri command

## Testing Considerations

### Algorithm Tests

- Test scoring với các size thresholds (500MB, 1GB, 5GB)
- Test scoring với các age thresholds (30, 90, 180 days)
- Test location type detection cho các paths khác nhau
- Test wildcard path expansion
- Test reasons generation với các combinations khác nhau

### Integration Tests

- Test với folders không tồn tại
- Test với folders có permission issues
- Test với empty selection
- Test progress tracking
- Test với nested folders trong whitelist paths