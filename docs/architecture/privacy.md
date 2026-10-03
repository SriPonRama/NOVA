# Security, Privacy & Desktop Lifecycle

## Privacy Principles
- **No Secrets in Git**: All tokens and keys must use `.env` files or secure OS credential managers.
- **Minimum Permissions**: Services only access what they need.
- **Local-First Storage**: Memories and configurations live strictly in local SQLite databases.
- **No Frame Storage**: Webcam frames are processed in memory and immediately discarded.
- **User Control**: Users can inspect, edit, and delete all stored memory.
- **Explicit Monitoring**: Active window tracking and camera usage require clear UI indicators and opt-in.
- **No Silent Collection**: NOVA does not collect telemetry or unrelated personal data.

## Desktop Lifecycle
NOVA is designed to be a lightweight background companion.
- **Startup**: Can be configured to start with Windows. Boots silently to the System Tray.
- **System Tray**: The primary anchor point. Left-click opens the chat overlay; right-click opens the context menu.
- **Visibility**: The UI can be hidden/shown via hotkeys or tray interactions.
- **Resource Management**: When idle or hidden, React suspends unnecessary rendering. Rust sleeps monitoring threads based on the active state.
- **Assessment Mode**: Kills intensive background services entirely.
- **Shutdown**: Gracefully closes the database connection and terminates the Python CV process before exiting.

## AI News Integration
The future AI News subsystem must:
- Retrieve news from verified RSS or API sources.
- Summarize articles securely.
- Provide explicit source links to prevent hallucinations.
- Maintain a strict boundary between retrieved facts and the AI's internal model knowledge.
