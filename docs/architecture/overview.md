# Architecture Overview

## Separation of Concerns
NOVA is built on a modular architecture to allow future expansion into AI and computer vision without bloating the core desktop application.

### UI Layer
- **Tech**: React, TypeScript, Tailwind CSS, Vite.
- **Responsibility**: Renders the application views, manages local UI state, communicates with the native layer via Tauri IPC.

### Application / Native Logic
- **Tech**: Rust (Tauri 2).
- **Responsibility**: OS integration (tray, windows, notifications), filesystem access, local database interaction, process management.

### Persistence
- **Tech**: SQLite (Planned).
- **Responsibility**: Stores long-term memory, settings, task history. 

### Computer Vision / AI (Future)
- **Tech**: Python, OpenCV, MediaPipe.
- **Responsibility**: Runs as an independent local service/process. Communicates with the Rust backend via sockets/IPC. Does NOT couple with the React UI.
