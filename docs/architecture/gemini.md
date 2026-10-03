# Gemini AI Integration Architecture

## Overview
NOVA uses an abstracted AI Provider pattern (`AIProvider`) to decouple the application from any single AI service. The first implemented provider is Google's Gemini API via REST.

## Gemini Provider Architecture
The `GeminiProvider` implements the `AIProvider` trait, exposing a single asynchronous `generate_response` method. The provider constructs the JSON payload required by the `gemini-1.5-flash` model, handling system instructions, message history mapping, and API HTTP requests using the `reqwest` crate.

- **Isolation**: The Gemini API logic is confined entirely to `src-tauri/src/ai/gemini.rs`. No other part of the system (React, `main.rs`, or other Tauri commands) knows how Gemini structures its requests or responses.
- **Data Normalization**: Responses are mapped to the generic `AIResponse` struct. Errors are mapped to the generic `AIError` enum, allowing uniform error handling across the application.

## Request/Response Flow
1. **React UI**: Captures user input and updates the local chat log.
2. **Conversation Service (Tauri Command)**: The `send_message` Tauri command receives the raw text from the frontend. It fetches the conversation history from a managed `Mutex` state (`ConversationState`), appends the new message, and constructs an `AIRequest`.
3. **AI Provider**: The `generate_response` method is called. It parses the request, formats it into Gemini's expected JSON structure, and dispatches the HTTP POST request.
4. **Resolution**: The JSON response is parsed into an `AIResponse`, appended to the Rust backend history, and the text is returned to the React UI via IPC.

## Security Model
- **Secrets Management**: The `GEMINI_API_KEY` is loaded natively in Rust via the `dotenvy` crate. The key is never exposed to the React frontend, passed via IPC, or bundled in client-side code.
- **Git Safety**: The `.env` file is explicitly ignored in `.gitignore`, and a `.env.example` is provided for developer onboarding.
- **Logging**: The application avoids logging authentication headers or sensitive API keys during failures.

## Error Handling
The `AIError` enum categorizes failures to provide graceful recovery and clear UX:
- `ConfigurationError`: Fired when the `GEMINI_API_KEY` is missing or invalid.
- `NetworkError`: Fired on request timeouts or TCP failures.
- `ProviderError`: Fired when Gemini returns a non-200 status code, malformed JSON, or empty responses.

Errors are converted to strings when returned to the UI, allowing React to display a friendly fallback without needing complex error parsing logic.

## Abstraction Rationale
We isolate Gemini specifically to guarantee we can switch to Local LLMs (e.g., LLaMA, Mistral) or other cloud providers without changing the conversation logic or the React frontend. The `AIProvider` trait ensures that as long as a provider accepts an `AIRequest` and returns an `AIResponse`, NOVA remains oblivious to the underlying implementation.
