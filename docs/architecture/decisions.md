# Architecture Decision Records (ADRs)

## ADR-001: Why Tauri 2 instead of Electron
- **Context:** NOVA requires a persistent background desktop presence.
- **Decision:** Use Tauri 2 instead of Electron.
- **Reasoning:** Electron bundles a full Chromium instance and Node.js environment, leading to high memory usage (>150MB baseline) and large binaries. Tauri uses the system's native webview (WebView2 on Windows), resulting in a significantly smaller memory footprint (~20-30MB) and binary size, which is critical for an always-on companion.
- **Consequences:** We must write system-level code in Rust rather than Node.js.

## ADR-002: Why React/TypeScript for the UI
- **Context:** The companion UI needs to be highly interactive, responsive, and maintainable.
- **Decision:** Use React with TypeScript and Vite.
- **Reasoning:** React has a massive ecosystem and excellent component modularity. TypeScript ensures type safety, especially when communicating with the Rust backend. Vite provides extremely fast HMR (Hot Module Replacement) during development.
- **Consequences:** We depend on the React ecosystem, but this is a standard and highly supported path.

## ADR-003: Why Rust handles native OS integration
- **Context:** NOVA needs to interact with the Windows OS (system tray, active windows, notifications).
- **Decision:** Rust handles all native OS interactions.
- **Reasoning:** Rust provides safe, high-performance, and direct access to the Win32 API. Keeping this in the Tauri backend rather than relying on brittle Node.js native addons ensures stability and security.
- **Consequences:** Developers working on native integrations must be proficient in Rust and the `windows-rs` crate.

## ADR-004: Why SQLite is planned for local persistence
- **Context:** NOVA needs to store long-term memories, user settings, and task history securely and locally.
- **Decision:** Use SQLite via a Rust ORM/driver (like `rusqlite` or `sqlx`).
- **Reasoning:** SQLite is a self-contained, serverless, zero-configuration, transactional SQL database engine. It perfectly fits the "local-first" privacy principle, keeping user data entirely on their machine in a single file.
- **Consequences:** We need to handle database migrations and schema updates in Rust.

## ADR-005: Why computer vision is separated from the desktop UI
- **Context:** Drowsiness detection requires heavy CPU/GPU processing of webcam feeds.
- **Decision:** Run CV tasks in a separate Python process.
- **Reasoning:** Embedding OpenCV/MediaPipe into Rust or JavaScript bloats the main application and can cause UI stuttering or freezes. A separate Python process isolates the heavy ML workloads and potential crashes from the core companion experience.
- **Consequences:** We must implement a robust IPC mechanism (e.g., sockets or standard I/O) between Rust and Python.

## ADR-006: Why AI providers are abstracted
- **Context:** AI models and providers (OpenAI, Anthropic, local LLMs like Llama 3) change rapidly.
- **Decision:** Abstract the AI provider logic behind a generic `AIEngine` interface in Rust.
- **Reasoning:** Hardcoding a specific API makes the application brittle. By defining core concepts (`AIRequest`, `AIResponse`, `ToolCall`), we can swap out the underlying model or provider at runtime (e.g., switching to local-only inference for privacy) without rewriting the application logic.
- **Consequences:** We must design a lowest-common-denominator abstraction for interacting with various LLMs.

## ADR-007: Why Assessment Mode is a first-class application state
- **Context:** Users need a guaranteed way to disable all tracking during sensitive tasks (e.g., taking an online exam or joining a private meeting).
- **Decision:** Implement Assessment Mode as a core state machine state at the Rust level.
- **Reasoning:** Handling this merely in the UI is insecure. By making it a native state, the Rust backend can decisively kill the Python CV process, detach from window monitoring hooks, and silence all events, ensuring absolute privacy compliance.
- **Consequences:** All native services must check the global state or be explicitly stoppable.

## ADR-008: Privacy-first/local-first design
- **Context:** An AI companion has access to highly sensitive context (screen time, memories, camera).
- **Decision:** NOVA will prioritize local-first storage and explicit consent.
- **Reasoning:** User trust is paramount. Webcam frames will NEVER be saved to disk by default. Memories will be fully visible and editable by the user. Secrets will never be committed to Git.
- **Consequences:** Feature development may be harder (e.g., requiring local models instead of cloud APIs for sensitive data), but it guarantees user security.
