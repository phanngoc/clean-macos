#!/usr/bin/env python3
"""
Task Progress Tracker for Spec Builder Skill

This script helps track implementation progress of tasks from tasks.md files.
It can:
- Extract tasks from tasks.md and save to CSV
- Mark tasks as done
- Show progress statistics
- Filter tasks by status/category
- Get specific task for implementation
"""

import csv
import re
import sys
import os
from pathlib import Path
from typing import List, Dict, Optional
from datetime import datetime

# CSV column names
CSV_COLUMNS = [
    'task_id',
    'title',
    'category',
    'status',
    'complexity',
    'estimated_time',
    'dependencies',
    'spec_path',
    'updated_at',
    'notes'
]

# Task statuses
STATUS_PENDING = 'pending'
STATUS_IN_PROGRESS = 'in_progress'
STATUS_DONE = 'done'
STATUS_BLOCKED = 'blocked'

VALID_STATUSES = [STATUS_PENDING, STATUS_IN_PROGRESS, STATUS_DONE, STATUS_BLOCKED]


def parse_tasks_md(file_path: str) -> List[Dict]:
    """Parse tasks.md file and extract all tasks."""
    tasks = []
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Pattern to match task sections
    # Matches: ### Task #N: [Title]
    task_pattern = r'### Task #(\d+):\s*(.+?)\n\n(.*?)(?=\n### Task #|\n## 📦|\Z)'
    
    matches = re.finditer(task_pattern, content, re.DOTALL)
    
    for match in matches:
        task_id = match.group(1)
        title = match.group(2).strip()
        task_content = match.group(3)
        
        # Extract category
        category_match = re.search(r'\*\*Category\*\*:\s*(.+?)\n', task_content)
        category = category_match.group(1).strip() if category_match else 'Unknown'
        
        # Extract complexity
        complexity_match = re.search(r'\*\*Complexity\*\*:\s*(.+?)\n', task_content)
        complexity = complexity_match.group(1).strip() if complexity_match else 'Unknown'
        
        # Extract estimated time
        time_match = re.search(r'\*\*Estimated Time\*\*:\s*(.+?)\n', task_content)
        estimated_time = time_match.group(1).strip() if time_match else 'Unknown'
        
        # Extract dependencies
        deps_match = re.search(r'\*\*Dependencies\*\*:\s*(.+?)\n', task_content, re.DOTALL)
        if deps_match:
            deps_text = deps_match.group(1).strip()
            # Extract task numbers from dependencies
            dep_tasks = re.findall(r'Task #(\d+)', deps_text)
            dependencies = ', '.join(dep_tasks) if dep_tasks else 'None'
        else:
            dependencies = 'None'
        
        tasks.append({
            'task_id': task_id,
            'title': title,
            'category': category,
            'status': STATUS_PENDING,
            'complexity': complexity,
            'estimated_time': estimated_time,
            'dependencies': dependencies,
            'spec_path': file_path,
            'updated_at': datetime.now().isoformat(),
            'notes': ''
        })
    
    return tasks


def load_tasks_csv(csv_path: str) -> List[Dict]:
    """Load tasks from CSV file."""
    if not os.path.exists(csv_path):
        return []
    
    tasks = []
    with open(csv_path, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            tasks.append(row)
    
    return tasks


def save_tasks_csv(csv_path: str, tasks: List[Dict]):
    """Save tasks to CSV file."""
    # Ensure directory exists
    os.makedirs(os.path.dirname(csv_path), exist_ok=True)
    
    with open(csv_path, 'w', encoding='utf-8', newline='') as f:
        writer = csv.DictWriter(f, fieldnames=CSV_COLUMNS)
        writer.writeheader()
        for task in tasks:
            writer.writerow(task)


def merge_tasks(existing_tasks: List[Dict], new_tasks: List[Dict]) -> List[Dict]:
    """Merge new tasks with existing tasks, preserving status."""
    # Create a map of existing tasks by (task_id, spec_path)
    existing_map = {}
    for task in existing_tasks:
        key = (task['task_id'], task['spec_path'])
        existing_map[key] = task
    
    # Merge: use existing status if task exists, otherwise use new task
    merged = []
    for new_task in new_tasks:
        key = (new_task['task_id'], new_task['spec_path'])
        if key in existing_map:
            # Preserve status and notes from existing
            existing = existing_map[key]
            new_task['status'] = existing['status']
            new_task['notes'] = existing['notes']
            new_task['updated_at'] = existing['updated_at']
        merged.append(new_task)
    
    return merged


def mark_task_done(csv_path: str, task_id: str, spec_path: Optional[str] = None):
    """Mark a task as done."""
    tasks = load_tasks_csv(csv_path)
    
    updated = False
    for task in tasks:
        if task['task_id'] == task_id:
            if spec_path is None or task['spec_path'] == spec_path:
                task['status'] = STATUS_DONE
                task['updated_at'] = datetime.now().isoformat()
                updated = True
                break
    
    if not updated:
        print(f"Error: Task #{task_id} not found in CSV")
        return False
    
    save_tasks_csv(csv_path, tasks)
    print(f"✅ Task #{task_id} marked as done")
    return True


def mark_task_status(csv_path: str, task_id: str, status: str, spec_path: Optional[str] = None):
    """Mark a task with a specific status."""
    if status not in VALID_STATUSES:
        print(f"Error: Invalid status '{status}'. Valid statuses: {', '.join(VALID_STATUSES)}")
        return False
    
    tasks = load_tasks_csv(csv_path)
    
    updated = False
    for task in tasks:
        if task['task_id'] == task_id:
            if spec_path is None or task['spec_path'] == spec_path:
                task['status'] = status
                task['updated_at'] = datetime.now().isoformat()
                updated = True
                break
    
    if not updated:
        print(f"Error: Task #{task_id} not found in CSV")
        return False
    
    save_tasks_csv(csv_path, tasks)
    print(f"✅ Task #{task_id} marked as {status}")
    return True


def show_progress(csv_path: str):
    """Show progress statistics."""
    tasks = load_tasks_csv(csv_path)
    
    if not tasks:
        print("No tasks found in CSV. Run 'extract' command first.")
        return
    
    total = len(tasks)
    done = sum(1 for t in tasks if t['status'] == STATUS_DONE)
    in_progress = sum(1 for t in tasks if t['status'] == STATUS_IN_PROGRESS)
    pending = sum(1 for t in tasks if t['status'] == STATUS_PENDING)
    blocked = sum(1 for t in tasks if t['status'] == STATUS_BLOCKED)
    
    progress_pct = (done / total * 100) if total > 0 else 0
    
    print("\n📊 Task Progress Summary")
    print("=" * 50)
    print(f"Total Tasks:     {total}")
    print(f"✅ Done:         {done} ({done/total*100:.1f}%)")
    print(f"🔄 In Progress:  {in_progress}")
    print(f"⏳ Pending:      {pending}")
    print(f"🚫 Blocked:      {blocked}")
    print(f"\nOverall Progress: {progress_pct:.1f}%")
    print("=" * 50)
    
    # Show by category
    categories = {}
    for task in tasks:
        cat = task['category']
        if cat not in categories:
            categories[cat] = {'total': 0, 'done': 0}
        categories[cat]['total'] += 1
        if task['status'] == STATUS_DONE:
            categories[cat]['done'] += 1
    
    if categories:
        print("\n📦 Progress by Category:")
        for cat, stats in sorted(categories.items()):
            pct = (stats['done'] / stats['total'] * 100) if stats['total'] > 0 else 0
            print(f"  {cat}: {stats['done']}/{stats['total']} ({pct:.1f}%)")


def list_tasks(csv_path: str, status: Optional[str] = None, category: Optional[str] = None):
    """List tasks with optional filtering."""
    tasks = load_tasks_csv(csv_path)
    
    if not tasks:
        print("No tasks found in CSV. Run 'extract' command first.")
        return
    
    # Filter tasks
    filtered = tasks
    if status:
        filtered = [t for t in filtered if t['status'] == status]
    if category:
        filtered = [t for t in filtered if t['category'].lower() == category.lower()]
    
    if not filtered:
        print("No tasks match the filter criteria.")
        return
    
    print(f"\n📋 Tasks ({len(filtered)} found)")
    print("=" * 80)
    
    for task in filtered:
        status_icon = {
            STATUS_DONE: '✅',
            STATUS_IN_PROGRESS: '🔄',
            STATUS_PENDING: '⏳',
            STATUS_BLOCKED: '🚫'
        }.get(task['status'], '❓')
        
        print(f"\n{status_icon} Task #{task['task_id']}: {task['title']}")
        print(f"   Category: {task['category']}")
        print(f"   Status: {task['status']}")
        print(f"   Complexity: {task['complexity']} | Time: {task['estimated_time']}")
        if task['dependencies'] != 'None':
            print(f"   Dependencies: {task['dependencies']}")
        if task['notes']:
            print(f"   Notes: {task['notes']}")


def get_task_for_implementation(csv_path: str, task_id: Optional[str] = None) -> Optional[Dict]:
    """Get a task ready for implementation."""
    tasks = load_tasks_csv(csv_path)
    
    if not tasks:
        print("No tasks found in CSV. Run 'extract' command first.")
        return None
    
    if task_id:
        # Get specific task
        for task in tasks:
            if task['task_id'] == task_id:
                return task
        print(f"Task #{task_id} not found.")
        return None
    
    # Find next pending task with no pending dependencies
    pending_tasks = [t for t in tasks if t['status'] == STATUS_PENDING]
    
    for task in pending_tasks:
        # Check if dependencies are done
        deps = task['dependencies']
        if deps == 'None' or not deps:
            return task
        
        # Check each dependency
        dep_ids = [d.strip() for d in deps.split(',')]
        all_deps_done = True
        for dep_id in dep_ids:
            dep_task = next((t for t in tasks if t['task_id'] == dep_id), None)
            if not dep_task or dep_task['status'] != STATUS_DONE:
                all_deps_done = False
                break
        
        if all_deps_done:
            return task
    
    # If no task with all deps done, return first pending
    if pending_tasks:
        return pending_tasks[0]
    
    print("No pending tasks available.")
    return None


def show_task_for_implementation(csv_path: str, task_id: Optional[str] = None):
    """Show task details for implementation."""
    task = get_task_for_implementation(csv_path, task_id)
    
    if not task:
        return
    
    print("\n🎯 Task Ready for Implementation")
    print("=" * 80)
    print(f"\nTask #{task['task_id']}: {task['title']}")
    print(f"Category: {task['category']}")
    print(f"Complexity: {task['complexity']}")
    print(f"Estimated Time: {task['estimated_time']}")
    print(f"Dependencies: {task['dependencies']}")
    print(f"Status: {task['status']}")
    print(f"Spec Path: {task['spec_path']}")
    
    if task['notes']:
        print(f"\nNotes: {task['notes']}")
    
    print("\n" + "=" * 80)
    print(f"\nTo mark this task as done, run:")
    print(f"  python track_progress.py done {task['task_id']}")
    print(f"\nTo mark as in progress, run:")
    print(f"  python track_progress.py status {task['task_id']} in_progress")


def main():
    """Main CLI entry point."""
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python track_progress.py extract <tasks.md> [csv_path]")
        print("  python track_progress.py done <task_id> [csv_path]")
        print("  python track_progress.py status <task_id> <status> [csv_path]")
        print("  python track_progress.py progress [csv_path]")
        print("  python track_progress.py list [--status=<status>] [--category=<cat>] [csv_path]")
        print("  python track_progress.py next [csv_path]")
        print("  python track_progress.py implement <task_id> [csv_path]")
        print("\nStatuses: pending, in_progress, done, blocked")
        sys.exit(1)
    
    command = sys.argv[1]
    
    # Default CSV path
    default_csv = os.path.join(os.path.dirname(__file__), 'tasks.csv')
    
    if command == 'extract':
        if len(sys.argv) < 3:
            print("Error: Please provide tasks.md path")
            sys.exit(1)
        
        tasks_md = sys.argv[2]
        csv_path = sys.argv[3] if len(sys.argv) > 3 else default_csv
        
        if not os.path.exists(tasks_md):
            print(f"Error: File not found: {tasks_md}")
            sys.exit(1)
        
        print(f"Extracting tasks from {tasks_md}...")
        new_tasks = parse_tasks_md(tasks_md)
        existing_tasks = load_tasks_csv(csv_path)
        merged_tasks = merge_tasks(existing_tasks, new_tasks)
        save_tasks_csv(csv_path, merged_tasks)
        print(f"✅ Extracted {len(new_tasks)} tasks to {csv_path}")
        print(f"   Total tasks in CSV: {len(merged_tasks)}")
    
    elif command == 'done':
        if len(sys.argv) < 3:
            print("Error: Please provide task_id")
            sys.exit(1)
        
        task_id = sys.argv[2]
        csv_path = sys.argv[3] if len(sys.argv) > 3 else default_csv
        mark_task_done(csv_path, task_id)
    
    elif command == 'status':
        if len(sys.argv) < 4:
            print("Error: Please provide task_id and status")
            sys.exit(1)
        
        task_id = sys.argv[2]
        status = sys.argv[3]
        csv_path = sys.argv[4] if len(sys.argv) > 4 else default_csv
        mark_task_status(csv_path, task_id, status)
    
    elif command == 'progress':
        csv_path = sys.argv[2] if len(sys.argv) > 2 else default_csv
        show_progress(csv_path)
    
    elif command == 'list':
        csv_path = default_csv
        status = None
        category = None
        
        # Parse arguments
        args = sys.argv[2:]
        for arg in args:
            if arg.startswith('--status='):
                status = arg.split('=', 1)[1]
            elif arg.startswith('--category='):
                category = arg.split('=', 1)[1]
            elif not arg.startswith('--'):
                csv_path = arg
        
        list_tasks(csv_path, status, category)
    
    elif command == 'next':
        csv_path = sys.argv[2] if len(sys.argv) > 2 else default_csv
        show_task_for_implementation(csv_path)
    
    elif command == 'implement':
        if len(sys.argv) < 3:
            print("Error: Please provide task_id")
            sys.exit(1)
        
        task_id = sys.argv[2]
        csv_path = sys.argv[3] if len(sys.argv) > 3 else default_csv
        
        # Mark as in_progress and show details
        mark_task_status(csv_path, task_id, STATUS_IN_PROGRESS)
        show_task_for_implementation(csv_path, task_id)
    
    else:
        print(f"Error: Unknown command '{command}'")
        sys.exit(1)


if __name__ == '__main__':
    main()

