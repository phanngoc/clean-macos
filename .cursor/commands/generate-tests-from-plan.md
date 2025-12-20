# Generate Unit Tests from Plan

## Mô tả
Command này sẽ nghiên cứu source code và tạo toàn bộ unit tests cho tính năng mới dựa trên file plan mô tả. Command sẽ:
1. Đọc và phân tích file plan để hiểu các tính năng cần implement
2. Nghiên cứu codebase hiện tại để hiểu patterns, structures, và testing conventions
3. Tạo comprehensive unit tests cho tất cả các functions, structs, và algorithms được mô tả trong plan
4. Đảm bảo tests cover edge cases, error handling, và integration scenarios

## Input
- File plan mô tả tính năng sắp được implement (path tới file `.plan.md`)

## Output
- Toàn bộ unit tests cho new feature sắp được implement, được viết theo patterns và conventions của project
- Tests được thêm vào file source code tương ứng (trong `#[cfg(test)] mod tests` block)

## Cách sử dụng
Khi được gọi, command sẽ:
1. Nhận path tới plan file từ user
2. Tự động thực hiện tất cả các bước dưới đây
3. Tạo/update test code trong các file tương ứng

Ví dụ usage:
```
@generate-tests-from-plan .cursor/plans/smart_scanner_page_feature_2a1f078c.plan.md
```

## Instructions cho AI
Khi user gọi command này với một plan file, bạn PHẢI thực hiện các bước sau:

### Step 1: Parse Input
- Đọc path tới plan file từ user input
- Đọc và parse plan file (YAML frontmatter + markdown content)
- Extract todos, implementation details, algorithms, structs, functions từ plan

### Step 2: Research Codebase
- Tìm các file source code liên quan (dựa trên plan mô tả)
- Đọc các test files hiện có để hiểu patterns:
  - `cache-cleaner-app/src-tauri/src/cache/large_caches.rs` (có tests)
  - `cache-cleaner-app/src-tauri/src/utils/filesystem.rs` (có tests)
  - `cache-cleaner-app/src-tauri/src/cache/cleaner.rs` (có tests)
- Note patterns: `tempfile::TempDir`, helper functions, async tests với `#[tokio::test]`

### Step 3: Identify Test Targets
Từ plan, xác định cần test:
- Mỗi struct/enum mới → test creation, serialization
- Mỗi function mới → test với valid/invalid inputs, edge cases
- Mỗi algorithm → test với các thresholds và boundary conditions
- Integration points → test end-to-end flows

### Step 4: Generate Tests
- Với mỗi file cần test, thêm `#[cfg(test)] mod tests` block (nếu chưa có)
- Viết tests theo đúng patterns đã nghiên cứu
- Đảm bảo mỗi test có descriptive name
- Cover: happy path, error cases, edge cases, boundary conditions

### Step 5: Write Code
- Sử dụng `search_replace` hoặc `write` tool để thêm tests vào file
- Tests phải compile được (syntax đúng, có thể fail nếu implementation chưa có)
- Follow Rust testing conventions

## Quy trình thực hiện

### Bước 1: Đọc và phân tích plan file
- Đọc file plan được chỉ định
- Xác định các components cần implement:
  - Structs/Enums mới
  - Functions mới
  - Algorithms (scoring, feature extraction, etc.)
  - Integration points (Tauri commands, frontend calls)
  - Error handling requirements

### Bước 2: Nghiên cứu codebase
- Tìm và đọc các file liên quan trong codebase
- Hiểu patterns hiện tại:
  - Cấu trúc test modules (`#[cfg(test)] mod tests`)
  - Test utilities (tempfile, helper functions)
  - Assertion patterns
  - Async test patterns (`#[tokio::test]`)
- Xem các test examples tương tự để match style

### Bước 3: Xác định test cases cần tạo
Dựa trên plan, tạo tests cho:

#### Algorithm Tests
- Test scoring functions với các thresholds:
  - Size thresholds (500MB, 1GB, 5GB)
  - Age thresholds (30, 90, 180 days)
  - Location type scoring
- Test feature extraction:
  - Size calculation
  - Age calculation (last accessed, created)
  - Location type detection
  - Depth calculation
- Test reasons generation với các combinations khác nhau
- Test wildcard path expansion

#### Unit Tests cho Functions
- Test từng helper function riêng lẻ
- Test với valid inputs
- Test với invalid inputs (nonexistent paths, permission errors)
- Test với edge cases (empty dirs, very large dirs, etc.)

#### Integration Tests
- Test với folders không tồn tại
- Test với folders có permission issues
- Test với empty selection
- Test progress tracking
- Test với nested folders trong whitelist paths
- Test end-to-end flow (scan → analyze → remove)

#### Struct/Enum Tests
- Test struct creation và serialization
- Test enum matching và conversions

### Bước 4: Tạo test file
- Tạo hoặc cập nhật test module trong file tương ứng
- Viết tests theo patterns đã nghiên cứu:
  - Sử dụng `tempfile::TempDir` cho test directories
  - Sử dụng helper functions để setup test data
  - Sử dụng `#[test]` cho sync tests
  - Sử dụng `#[tokio::test]` cho async tests
  - Assertions rõ ràng với descriptive messages

### Bước 5: Đảm bảo coverage
- Test happy paths
- Test error cases
- Test edge cases
- Test boundary conditions (thresholds)
- Test với real-world scenarios

## Ví dụ Output Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    // Test helper functions
    #[test]
    fn test_expand_wildcard_paths() { ... }
    
    #[test]
    fn test_determine_location_type() { ... }
    
    #[test]
    fn test_calculate_folder_depth() { ... }
    
    // Test feature extraction
    #[test]
    fn test_extract_folder_features() { ... }
    
    // Test scoring algorithm
    #[test]
    fn test_calculate_size_score() { ... }
    
    #[test]
    fn test_calculate_age_score() { ... }
    
    #[test]
    fn test_calculate_location_score() { ... }
    
    #[test]
    fn test_scoring_algorithm_integration() { ... }
    
    // Test reasons generation
    #[test]
    fn test_generate_reasons() { ... }
    
    // Test remove functionality
    #[tokio::test]
    async fn test_remove_suggested_folders() { ... }
    
    // Integration tests
    #[tokio::test]
    async fn test_scan_suggestions_end_to_end() { ... }
}
```

## Notes
- Tests phải compile được (có thể fail nếu implementation chưa có, nhưng syntax phải đúng)
- Tests phải follow Rust testing best practices
- Tests phải có descriptive names
- Tests phải cover cả success và failure cases
- Tests phải sử dụng temp directories để không ảnh hưởng system files

