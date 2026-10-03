# Productivity Engine & Mini Puzzles

## Productivity Engine
The productivity subsystem is responsible for task and time management. It is implemented purely in Rust to ensure reliability regardless of UI state.

### Responsibilities
- **Tasks**: Daily tasks, priorities, completion tracking.
- **Focus Sessions**: Pomodoro-style timers.
- **Breaks**: Enforced or suggested rest periods.
- **Reminders**: Time-based or context-based alerts.
- **Schedules**: Planned blocks of work.

### Architecture
The engine is decoupled from the UI:
`UI -> Productivity Service (Rust) -> SQLite Persistence`
The UI merely renders the state (e.g., "Timer at 15:00") and sends intent ("Start Focus"), but the Rust backend actually counts the time and triggers OS notifications.

## Mini Puzzle System (Arcade)
During breaks, NOVA can offer lightweight puzzles to keep the user engaged without the stress of actual work.

### Responsibilities
- **Games**: Logic puzzles, number sequences, pattern recognition.
- **Engine**: The game logic runs entirely in the React frontend, as it is a pure UI experience isolated from core productivity data.
- **State**: Game sessions, timers, and scores.
