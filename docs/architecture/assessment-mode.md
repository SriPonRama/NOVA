# Assessment Mode

## Concept
Assessment Mode is a mandatory privacy/safety feature. It acts as a global kill-switch for all intelligent and monitoring behaviors.

## States
The application state machine in Rust dictates active features:
- `ACTIVE`: Normal operation.
- `FOCUS`: Notifications suppressed except for productivity alerts.
- `BREAK`: Puzzles enabled, task reminders suppressed.
- `ASSESSMENT`: **Total lockdown.**

## Assessment State Constraints
When `ASSESSMENT` is triggered:
- The desktop companion UI is hidden.
- Native notifications are suppressed.
- AI chat interactions are disabled.
- The Python CV Engine is forcibly terminated.
- Active-window monitoring loops are suspended.
- Background AI/News processing is halted.

## Transitions
To prevent accidental background monitoring, Assessment Mode is enforced at the root of the Rust application loop. Services must check the global state (`is_assessment_mode()`) before executing any polling or processing tick. Exiting Assessment Mode requires explicit user action.
