# Desktop Monitoring & Computer Vision

## Distraction Awareness
The Rust backend utilizes native Windows APIs to monitor the currently focused window.

### Classification Logic
- **Detection**: Rust periodically polls the active window title/executable.
- **Classification**: Apps are categorized (e.g., VS Code = Productive, Netflix = Distracting). Users can configure and override these categories.
- **Thresholds**: Distractions are ignored for short periods (e.g., 30 seconds to reply to a message). Prolonged distraction during a "Focus Session" triggers a reminder.
- **Privacy**: Window titles are evaluated locally and discarded. Contents are never recorded.

## Drowsiness Detection (Computer Vision)
A separate Python process handles webcam processing using OpenCV and MediaPipe.

### Architecture
- **Isolation**: Runs as a separate `cv_engine.exe` or python script, communicating with Rust via IPC/Sockets.
- **Detection**: Evaluates eye aspect ratio (EAR) and head pose to detect prolonged drowsiness.
- **Thresholds & Cooldowns**: Requires consistent drowsiness over N seconds to avoid false positives from blinking. Includes cooldowns to prevent notification spam.
- **Privacy**: **Webcam frames are NEVER stored to disk or sent to the cloud.** Only the binary state ("drowsy" / "alert") is transmitted to Rust. The camera is explicitly disabled when not in use.
