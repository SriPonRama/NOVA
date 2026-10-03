# Architecture Overview

## 1. High-Level Architecture
NOVA is built on a modular architecture separating the user interface, native OS integrations, and heavy processing tasks.

```text
                    NOVA
                     |
        +------------+-------------+
        |            |             |
        v            v             v
   Desktop UI    AI Engine     Core Engine
 (React/Vite)  (Abstraction) (Rust/Tauri)
        |            |             |
        |            |        +----+----+
        |            |        |    |    |
        |            |      Tasks Focus Breaks
        |            |
        |            v
        |       Memory Engine
        |            |
        |            v
        |          SQLite
        |
        +-------------------------------+
        |               |               |
        v               v               v
   CV Engine      Desktop Monitor     Arcade
 (Python/OpenCV)   (Rust Native)    (React Logic)
        |               |               |
   Drowsiness     Active Window        Puzzles
   Detection      Awareness
```

## 2. Frontend / Native Boundary
The boundary between React (Frontend) and Rust (Backend) is enforced via Tauri's IPC (Inter-Process Communication).

### React / TypeScript (Frontend)
The frontend is strictly responsible for presentation and local UI state.
- **UI Components:** Rendering the chat interface, task views, memory managers, settings panels, and puzzle UI.
- **State:** Managing active views, local form states, and caching data for display.
- **Routing:** Navigating between different companion modes.

### Tauri / Rust (Native Core)
The Rust backend is the authoritative source of truth and system interaction.
- **OS Integration:** System tray, native notifications, active-window detection, and application lifecycle (startup/shutdown).
- **Persistence:** Reading and writing to SQLite, managing the filesystem.
- **Process Management:** Spawning and monitoring the external Python CV Engine.
- **Secure Communication:** Exposing specific, secure commands to the frontend via `tauri::command`.

React communicates with Rust exclusively by invoking registered Tauri commands (`invoke('save_memory', { ... })`) and listening for native events (e.g., `listen('drowsiness-detected')`).

## 3. Data Flow
### A. Normal Conversation
1. **User** inputs text in React.
2. **React** sends an IPC command to the Rust **AI Service**.
3. **Rust** queries the **Memory Service** (SQLite) for context.
4. **Rust** formats the prompt and calls the **AI Provider** abstraction.
5. **Rust** receives the response and sends it back to **React** via IPC.

### B. Memory Creation
1. **User** explicitly asks to remember a fact, or the AI decides it's important.
2. The **AI Command** triggers a `store_memory` action.
3. The **Memory Service** (Rust) categorizes and writes it to **SQLite**.
4. The **Memory UI** (React) updates by fetching the new list from Rust.

### C. Focus Reminder
1. A **Task** is started.
2. The **Productivity Engine** (Rust) tracks the timer.
3. The timer expires or an event is triggered.
4. The **Notification Service** (Rust) triggers a native OS notification to the **User**.

### D. Drowsiness Detection
1. **Camera** captures a frame (processed locally).
2. The **Local CV Process** (Python) analyzes the frame and calculates a drowsiness state.
3. The Python process sends a socket/IPC message to the Rust **Event Bus**.
4. Rust triggers a **Reminder** to the user (via native notification or React UI).

### E. Distraction Awareness
1. The user switches to a new **Active Window**.
2. The **Desktop Monitor** (Rust) detects the window change via Windows APIs.
3. Rust checks the **Classification** of the window title against user settings.
4. If the distraction exceeds the **Threshold**, a **Reminder** is fired.

### F. Assessment Mode
1. User enables **Assessment Mode** in the UI.
2. The UI sends a command to the Rust **Global Application State**.
3. Rust stops the CV Engine, pauses the Desktop Monitor, disables background AI, and hides the companion UI.
4. Rust disables all native notifications.

## 4. Future Project Structure
The repository will be structured to maintain these boundaries:
- `src/`: React UI, Tailwind styles, frontend state, and IPC clients.
- `src-tauri/src/`: Rust backend entry points.
- `src-tauri/src/ai/`: AI Engine abstraction and provider implementations.
- `src-tauri/src/memory/`: SQLite persistence and memory models.
- `src-tauri/src/productivity/`: Focus, tasks, and break logic.
- `src-tauri/src/monitor/`: Windows native API wrappers for active window detection.
- `src-cv/`: External Python project for OpenCV/MediaPipe drowsiness detection.
- `shared/`: Shared type definitions (e.g., TypeScript interfaces generated from Rust structs).
