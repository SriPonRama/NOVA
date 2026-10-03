# AI Engine Design

## Concept
The AI layer must be abstracted to prevent hard-coupling to any specific provider (e.g., OpenAI, local LLMs). The AI must NOT have unrestricted access to the operating system; it communicates purely through a controlled tool boundary.

## Core Abstractions
The Rust backend will define the following models:
- **`AIProvider`**: A trait/interface defining methods like `generate_response()` and `stream_response()`.
- **`AIRequest`**: A standard struct containing the user prompt, system prompt, conversation history, and available tools.
- **`AIResponse`**: A standard struct returning text and optional tool invocations.
- **`ConversationContext`**: Temporary short-term memory (the current chat history).
- **`ToolCall`**: A structured request from the AI to perform a specific action.
- **`AgentContext`**: The injected long-term memory and relevant productivity state.

## Controlled Tool Boundary
The AI cannot execute arbitrary bash commands or read arbitrary files. It is strictly limited to pre-registered application services.

### Execution Flow
1. **User** asks a question.
2. **AI** determines it needs to use a tool (e.g., `create_memory`).
3. **AI** outputs an **Allowed Tool Call**.
4. The **Rust Application Service (ToolExecutor)** intercepts the tool call, validates it, and executes the native logic.
5. The **Result** is fed back to the AI in the same conversation turn.
6. The AI loops until it has all the context it needs, returning a final response to the User.

**NOTE**: See [tools.md](./tools.md) for detailed Tool execution architecture, validation rules, and the implemented memory tools (Phase 4B).
