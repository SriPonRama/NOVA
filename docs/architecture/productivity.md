# Productivity & Focus Architecture (Phase 5A)

## Overview
NOVA includes a dedicated local productivity and task management subsystem. This system operates completely independent from the core memory architecture to ensure that semantic recollections ("memories") are logically separated from actionable, scheduled items ("tasks").

## Task Architecture

### Database Schema (SQLite)
Tasks are persisted to the same `nova.db` SQLite database as memories, but in a dedicated `tasks` table.

- `id` (TEXT, PRIMARY KEY): Unique UUIDv4
- `title` (TEXT, NOT NULL): The headline of the task (validated, max length)
- `description` (TEXT): Optional body/details
- `date` (TEXT, NOT NULL): Date string `YYYY-MM-DD` associated with the user's local timezone
- `estimated_minutes` (INTEGER): Optional duration (0-1440 mins)
- `priority` (TEXT, NOT NULL): Enum (`LOW`, `MEDIUM`, `HIGH`)
- `status` (TEXT, NOT NULL): Enum (`PENDING`, `IN_PROGRESS`, `COMPLETED`, `CANCELLED`)
- `position` (INTEGER, NOT NULL): For user-defined explicit sorting within a single date
- `created_at` (INTEGER, NOT NULL): UNIX timestamp
- `updated_at` (INTEGER, NOT NULL): UNIX timestamp

### Application Domain
The `productivity` module is divided into:
1. **Repository** (`repository.rs`): Executes SQL.
2. **Service** (`service.rs`): Enforces business constraints, bounds checking, and position management.
3. **Commands** (`commands.rs`): Tauri IPC exposed to the React frontend.

## Implementation Details

### Daily Planning & Date Handling
Tasks are tied to specific calendar dates rather than abstract timestamps. This ensures that a task scheduled for "October 3rd" remains on that date UI regardless of when it was actually created or modified. 
- Date strings are stored in `YYYY-MM-DD` format.
- The `Today` UI queries explicitly for tasks matching the current local calendar date.

### Priority & Status Model
- Priorities (`Low`, `Medium`, `High`) are completely user-controlled. Gemini does not arbitrary adjust or score priorities.
- The lifecycle follows a simple flow: `Pending -> In Progress -> Completed`. Tasks can also be toggled or cancelled.

### Ordering & Positioning
- Tasks have a `position` integer column.
- When creating a task, it's appended to the end of that date's list (max position + 1).
- The `reorder_tasks` command takes a desired sequence of IDs and atomically updates all positions for a specific date, driving explicit movement.

### Progress Calculation
- Progress is computed dynamically on the frontend.
- It reflects the percentage of `Completed` tasks vs. total tasks for the day. (E.g., 2 of 4 complete = 50%).
- Estimated time calculations are deferred to future phases.

### Relationship to Memory
Tasks are NOT memories. 
- A memory is unstructured semantic knowledge about the user.
- A task is a structured, actionable item.
- They do not share tables, though they share the same database connection instance for simplicity (`db::DbConnection`).

## Focus Session Architecture (Phase 5B)

### Focus Session Schema (SQLite)
Focus sessions are tracked in a `focus_sessions` table within `nova.db`.
- `id` (TEXT, PRIMARY KEY): Unique UUIDv4
- `task_id` (TEXT): Associated task ID, if any (null for breaks)
- `session_type` (TEXT, NOT NULL): Enum (`FOCUS`, `BREAK`)
- `planned_seconds` (INTEGER, NOT NULL): Total duration in seconds
- `started_at` (INTEGER): UNIX timestamp when session began
- `paused_at` (INTEGER): UNIX timestamp when currently paused
- `ended_at` (INTEGER): UNIX timestamp when finished or cancelled
- `status` (TEXT, NOT NULL): Enum (`RUNNING`, `PAUSED`, `COMPLETED`, `CANCELLED`)
- `created_at` (INTEGER, NOT NULL): UNIX timestamp
- `updated_at` (INTEGER, NOT NULL): UNIX timestamp

### Timer Authority Strategy
The Rust backend is authoritative. Real elapsed time is calculated using UNIX timestamps (Current Time - Started At), taking into account any paused intervals. This ensures timers remain completely accurate even when the window is hidden, minimized, or during normal computer sleep without requiring high-frequency background loops.

### Break System
Breaks follow the exact same schema but lack a `task_id` and have `session_type = 'BREAK'`.

### Future Integration (Deferred)

The following features are **NOT IMPLEMENTED YET**:

- **Desktop/Active-Window Monitoring**
- **Distraction & Drowsiness Detection (Webcam)**
- **Gemini Task Management Tools**: Currently, Gemini has no access to the task layer.
- **AI Scheduling & Intervention**

## Assessment Mode Constraints
The task and focus system natively respect `Assessment Mode`. While Assessment Mode is active, the UI is hidden, preventing user manipulation of the task list or timers, and because Gemini is blocked and background workers are suspended, no productivity intervention occurs.
