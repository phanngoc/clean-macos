# Task Progress Tracker

Script Python để theo dõi tiến độ implement các tasks từ `tasks.md`.

## Quick Start

```bash
# 1. Extract tasks từ tasks.md
python track_progress.py extract ../../specs/[feature-name]/tasks.md

# 2. Xem progress tổng thể
python track_progress.py progress

# 3. Lấy task tiếp theo để implement
python track_progress.py next

# 4. Bắt đầu implement một task cụ thể
python track_progress.py implement <task_id>

# 5. Đánh dấu task đã hoàn thành
python track_progress.py done <task_id>
```

## Commands

### Extract Tasks
```bash
python track_progress.py extract <tasks.md> [csv_path]
```
Parse tasks từ `tasks.md` và lưu vào CSV. Nếu CSV đã có tasks, sẽ merge và giữ nguyên status.

### View Progress
```bash
python track_progress.py progress [csv_path]
```
Hiển thị thống kê tổng thể: tổng số tasks, số done, in progress, pending, blocked, và progress theo category.

### List Tasks
```bash
python track_progress.py list [--status=<status>] [--category=<cat>] [csv_path]
```
Liệt kê tasks với optional filtering:
- `--status=pending|in_progress|done|blocked`
- `--category=<category_name>`

### Get Next Task
```bash
python track_progress.py next [csv_path]
```
Lấy task tiếp theo sẵn sàng để implement (tự động check dependencies).

### Implement Task
```bash
python track_progress.py implement <task_id> [csv_path]
```
- Đánh dấu task là `in_progress`
- Hiển thị đầy đủ thông tin task
- Hiển thị hướng dẫn mark done

### Mark Task Done
```bash
python track_progress.py done <task_id> [csv_path]
```
Đánh dấu task là `done`.

### Update Task Status
```bash
python track_progress.py status <task_id> <status> [csv_path]
```
Cập nhật status của task. Status hợp lệ: `pending`, `in_progress`, `done`, `blocked`.

## CSV Structure

File `tasks.csv` chứa các cột:
- `task_id`: Số task từ tasks.md
- `title`: Tiêu đề task
- `category`: Category (Backend, Frontend, Testing, etc.)
- `status`: Status hiện tại (pending, in_progress, done, blocked)
- `complexity`: Độ phức tạp (Low, Medium, High)
- `estimated_time`: Ước tính thời gian
- `dependencies`: Danh sách task IDs phụ thuộc (comma-separated)
- `spec_path`: Đường dẫn đến file tasks.md
- `updated_at`: Timestamp cập nhật lần cuối
- `notes`: Ghi chú (optional)

## Workflow Example

```bash
# Sau khi generate tasks.md
cd .claude/skills/spec-builder/scripts

# Extract tasks
python track_progress.py extract ../../specs/my-feature/tasks.md

# Xem progress
python track_progress.py progress

# Lấy task tiếp theo
python track_progress.py next

# Hoặc implement task cụ thể
python track_progress.py implement 5

# Sau khi implement xong
python track_progress.py done 5

# Check progress lại
python track_progress.py progress
```

## Features

- ✅ Tự động parse tasks từ tasks.md
- ✅ Preserve status khi re-extract
- ✅ Check dependencies khi suggest next task
- ✅ Track progress theo category
- ✅ Filter tasks theo status/category
- ✅ Timestamp tracking

## Notes

- CSV path mặc định: `tasks.csv` trong cùng thư mục script
- Khi extract lại, status và notes được preserve
- Task dependencies được check khi suggest next task
- Script tự động tạo CSV nếu chưa có

