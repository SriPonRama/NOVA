# NOVA
**Early Development**

NOVA is a mature personal AI desktop companion for Windows, designed with privacy and modularity in mind.

## Planned Capabilities
- Persistent desktop presence
- Conversational AI
- User-controlled long-term memory
- Study/task management
- Focus sessions & break management
- Intelligent reminders
- Optional local drowsiness detection
- Distraction awareness (based on active applications)
- Current AI news briefing
- Mini logic/puzzle games
- Voice interaction
- Explicit Assessment Mode (completely disables NOVA)
- Privacy-first / local-first architecture

## Technology Stack
- **Desktop/Native Layer**: Tauri 2, Rust
- **Frontend**: React, TypeScript, Vite, Tailwind CSS
- **Future Capabilities**: SQLite (local database), Python / OpenCV / MediaPipe (computer vision)

## High-Level Architecture
NOVA separates the UI (React/TS), the desktop/native shell (Rust/Tauri), and future AI/Computer Vision systems into distinct modular boundaries. 

## Development Setup
Ensure you have Node.js and Rust installed.
1. `npm install`
2. `npm run tauri dev`

## Git Workflow
We use a standard branching and pull request workflow. Commit messages follow conventional commits (e.g., `feat:`, `fix:`, `chore:`, `docs:`).

## Privacy Principles
- Local-first architecture where practical.
- Webcam frames are NOT stored by default.
- Camera processing happens locally.
- Users have full transparency into memories and can edit/delete them.
- No silent collection of unnecessary telemetry.
- **Assessment Mode**: Completely disables all monitoring and interactions when enabled.

## Roadmap
- MVP: Desktop UI shell and scaffold
- Phase 1: Local memory and settings integration
- Phase 2: Focus and task management
- Phase 3: Conversational AI and logic modules
- Phase 4: Local drowsiness and distraction monitoring
