# NOVA
**Early Development**

NOVA is a mature personal AI desktop companion for Windows, designed with privacy and modularity in mind.

## Capabilities Status

- **IMPLEMENTED**: Persistent desktop UI shell (Tauri/React scaffolding)
- **PLANNED**: Conversational AI
- **PLANNED**: User-controlled long-term memory (SQLite)
- **PLANNED**: Explicit Assessment Mode (completely disables NOVA)
- **FUTURE**: Study/task management & Focus sessions
- **FUTURE**: Distraction awareness (based on active applications)
- **FUTURE**: Current AI news briefing
- **FUTURE**: Mini logic/puzzle games
- **FUTURE**: Intelligent reminders
- **FUTURE**: Optional local drowsiness detection (Computer Vision)
- **FUTURE**: Voice interaction

## Technology Stack
- **Desktop/Native Layer**: Tauri 2, Rust
- **Frontend**: React, TypeScript, Vite, Tailwind CSS
- **Future Capabilities**: SQLite (local database), Python / OpenCV / MediaPipe (computer vision)

## High-Level Architecture
NOVA separates the UI (React/TS), the desktop/native shell (Rust/Tauri), and future AI/Computer Vision systems into distinct modular boundaries. 

For detailed architecture blueprints and ADRs, see the `docs/architecture/` directory.

## Development Setup
Ensure you have Node.js and Rust installed.
1. `npm install`
2. `npm run tauri dev`

## Privacy Principles
- Local-first architecture where practical.
- Webcam frames are NOT stored by default.
- Camera processing happens locally.
- Users have full transparency into memories and can edit/delete them.
- No silent collection of unnecessary telemetry.
- **Assessment Mode**: Completely disables all monitoring and interactions when enabled.
