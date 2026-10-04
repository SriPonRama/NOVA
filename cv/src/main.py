import sys
import time
import threading
from camera import Camera
from face import FaceDetector
from protocol import send_message, parse_message
import drowsiness

def main():
    cam = Camera()
    detector = FaceDetector()
    running = False

    def listen_commands():
        nonlocal running
        for line in sys.stdin:
            msg = parse_message(line)
            if not msg:
                continue
            
            t = msg.get("type")
            if t == "START":
                if not running:
                    if cam.start():
                        running = True
                        send_message("status", status="RUNNING")
                    else:
                        send_message("status", status="CAMERA_ERROR")
            elif t == "STOP":
                running = False
                cam.stop()
                send_message("status", status="STOPPED")
            elif t == "PING":
                send_message("pong")

    # Start command listener thread
    t = threading.Thread(target=listen_commands, daemon=True)
    t.start()

    send_message("status", status="READY")

    while True:
        if not running:
            time.sleep(0.1)
            continue

        frame = cam.read()
        if frame is None:
            send_message("drowsiness_signal", 
                face_detected=False,
                eye_measurement=0.0,
                confidence=0.0,
                timestamp=int(time.time()),
                status="CAMERA_ERROR"
            )
            time.sleep(0.5)
            continue

        results = detector.process(frame)
        
        if results and results.multi_face_landmarks:
            face_landmarks = results.multi_face_landmarks[0]
            left_ear, right_ear, combined_ear = drowsiness.process_landmarks(face_landmarks)
            
            send_message("drowsiness_signal",
                face_detected=True,
                eye_measurement=combined_ear,
                confidence=1.0,
                timestamp=int(time.time()),
                status="FACE_DETECTED"
            )
        else:
            send_message("drowsiness_signal",
                face_detected=False,
                eye_measurement=0.0,
                confidence=0.0,
                timestamp=int(time.time()),
                status="NO_FACE"
            )

        # Rate limit to ~10 fps to save CPU
        time.sleep(0.1)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        send_message("error", message=str(e))
