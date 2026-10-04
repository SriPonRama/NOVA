# NOVA Computer Vision Subsystem

This subsystem provides completely local, offline computer vision capabilities for NOVA, specifically for drowsiness detection. 
It is isolated from the main application via an IPC protocol and is invoked by the Rust backend.

## Privacy Guarantee
**This subsystem never saves, logs, transmits, or uploads webcam frames.**
Frames exist only temporarily in memory for processing and are discarded immediately.
No facial recognition or identity tracking is performed.

## Installation
Ensure you have Python 3.9+ installed.
Run the following to install dependencies:
```bash
pip install -r requirements.txt
```

## Running standalone
You can test the CV loop via stdin:
```bash
python src/main.py
```
And provide structured IPC commands.
