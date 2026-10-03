# Memory System Design

## Memory Philosophy
Memory is strictly user-controlled, locally stored (SQLite), and highly transparent. Not everything the user says is permanently recorded; a deliberate memory policy dictates what is saved.

## Memory Categories
- **User Preferences**: How the user likes to work, UI themes, interaction styles.
- **Goals**: Long-term objectives (e.g., "Learn Rust by December").
- **Study Information**: Facts relevant to active learning.
- **Project Information**: Context about active software or work projects.
- **Important Facts**: Personal details explicitly asked to be remembered.
- **Temporary Context**: Short-term chat history (cleared on restart or manually).

## Memory Policy & Interactions
- **Creation**: Users can explicitly say "Remember that..." or the AI can automatically extract facts (subject to configuration).
- **Retrieval**: Before prompting the AI, the Rust backend queries SQLite for relevant memories based on semantic keywords or active context, injecting them into the `AgentContext`.
- **Management**: The React UI provides a "Memory Manager" view where users can search, edit, delete, or wipe all memories.
- **Privacy**: Memories never leave the local SQLite database unless explicitly sent as context to a cloud AI provider (if the user opts-in to cloud AI).
