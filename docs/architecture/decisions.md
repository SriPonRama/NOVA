# Architectural Decisions (ADRs)

## 1. Tauri Over Electron
We chose Tauri over Electron to minimize memory footprint and binary size, as NOVA is intended to be a persistent, always-running background companion.

## 2. Separate Python Process for Computer Vision
Instead of embedding OpenCV or heavy ML libraries directly into Rust or using JS wrappers, computer vision (drowsiness detection) will run as a separate local Python process. This ensures the UI remains extremely fast, prevents bloated binaries, and isolates potential crashes in ML processing.

## 3. Tailwind CSS
Used for rapid, maintainable styling without writing massive custom CSS files.

## 4. Assessment Mode
Architecturally, Assessment Mode must act as a master kill-switch in the Rust backend, halting all active listeners, tray loops, and child Python processes to guarantee privacy compliance.
