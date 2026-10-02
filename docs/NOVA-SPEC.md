# NOVA Specification

## 1. Product Vision
NOVA is designed to be a deeply integrated, unobtrusive, and highly privacy-focused desktop AI companion. It aims to assist users in productivity, context awareness, and health without compromising local control.

## 2. Target Experience
A persistent, always-available desktop presence (system tray/desktop overlay). It should feel like a native Windows application that is fast, reliable, and intelligent.

## 3. Core Features
- Persistent desktop UI
- Context-aware reminders
- Conversational assistant
- Extensible logic

## 4. Desktop Behavior
Lives in the system tray. Can spawn overlay windows, focus timers, or brief news alerts. Native OS notifications integration.

## 5. Memory Philosophy
The AI will remember context across sessions. All memories are stored locally (SQLite), strictly user-controllable, and completely editable/deletable.

## 6. Productivity System
Includes built-in focus sessions, study/task management, and break management to keep the user healthy and on track.

## 7. Distraction Awareness
Monitors active applications locally to provide nudges when the user is off-task during focus sessions.

## 8. Drowsiness Detection
Optional feature using local webcam feeds (via OpenCV/MediaPipe in a separate Python process) to detect if the user is falling asleep during critical work/study hours. Frames are NEVER stored.

## 9. AI News
Provides a brief daily or on-demand summary of current AI news.

## 10. Mini Puzzle System
Offers small logic games or puzzles during breaks to keep the brain engaged but relaxed.

## 11. Assessment Mode
**MANDATORY REQUIREMENT**: An explicit mode that completely disables NOVA. No UI, no notifications, no reminders, no AI interaction, no webcam processing, no active-application monitoring, no voice interaction, and no background AI processing. Existing data is retained.

## 12. Privacy/Security Requirements
- Secrets (API keys, tokens) must never enter Git.
- Local-first inference or explicitly opt-in cloud AI.
- No personal data collection.

## 13. Future Extensibility
Architecture must allow snapping in new modular features (e.g., Python services) without coupling them directly to the React frontend.

## 14. MVP Boundary
The MVP will solely consist of the desktop UI shell, settings, and basic local memory storage structure.

## 15. Future Roadmap
1. Desktop Scaffold
2. Settings & Memory
3. Focus & Productivity Engine
4. Advanced AI & Computer Vision Integration
