# Memory Architecture (Phase 4A)

## Overview
NOVA uses a local SQLite database to persist user memories and preferences. This allows NOVA to eventually recall context across sessions without relying on cloud storage or external memory providers, adhering strictly to a privacy-first, local-first design.

## Database Technology & Location
- **Technology**: `rusqlite` (bundled feature) is used to natively interface with SQLite from the Rust backend.
- **Location**: The database is stored securely in the OS-specific application data directory provided by Tauri (`tauri::AppHandle::path().app_data_dir()`). On Windows, this resolves to `%APPDATA%\com.sriponrama.nova\nova.db`.

## Memory Schema
The schema is designed to be simple but extensible:
- `id` (TEXT, PRIMARY KEY): A unique UUIDv4.
- `content` (TEXT): The actual memory string.
- `category` (TEXT): A classified tag (`USER_PREFERENCE`, `GOAL`, `STUDY`, `PROJECT`, `IMPORTANT`, `TEMPORARY`).
- `created_at` (INTEGER): Unix timestamp (seconds).
- `updated_at` (INTEGER): Unix timestamp (seconds).

## Domain Architecture
The implementation separates concerns into:
1. **Repository (`repository.rs`)**: Handles SQLite connections, SQL execution, and migrations.
2. **Service (`service.rs`)**: Enforces business logic (e.g., rejecting empty strings, enforcing max lengths, auto-generating timestamps and UUIDs).
3. **Commands (`commands.rs`)**: The Tauri IPC boundary exposing CRUD operations securely to React.

## Migration Strategy
The application safely creates the `memories` table on startup using `CREATE TABLE IF NOT EXISTS`. This prevents the application from failing or destroying existing user data if the database already exists. Future schema changes will require standard ALTER statements mapped to application versions.

## Search Strategy
For Phase 4A, memory search is implemented using a simple SQLite `LIKE '%query%'` text search. 
- **Why no semantic search / embeddings?**: The goal of this phase is strictly establishing the persistent foundation. Semantic search requires shipping embedding models (e.g., SentenceTransformers) or calling external APIs, which significantly increases complexity, binary size, and latency. A simple string match is sufficient for manual retrieval right now.

## Privacy Model
- Memories are stored locally on the user's disk.
- No memory synchronization to the cloud.
- Memory content is NOT logged to the terminal or debug files.
- The `.db` files are explicitly excluded via `.gitignore`.

## AI Integration (Phase 4B Complete)
Gemini now integrates with the Memory Service via a strictly controlled **ToolExecutor**.
- The AI can automatically call `create_memory`, `search_memory`, `get_memory`, and `update_memory`.
- **Safety**: `delete_memory` is exposed to the AI, but it is **not** immediately executed. Instead, it triggers a UI confirmation prompt. Only if the user explicitly approves does the memory get permanently deleted.
- **Assessment Mode**: When Assessment Mode is active, the Rust backend explicitly hard-blocks the entire AI invocation loop and any memory tool execution, maintaining focus and privacy.
