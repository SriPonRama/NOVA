import cv2

class Camera:
    def __init__(self, index=0):
        self.index = index
        self.cap = None

    def start(self):
        if self.cap is None:
            self.cap = cv2.VideoCapture(self.index)
            # Try to lower resolution to reduce processing cost
            self.cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
            self.cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
            
            if not self.cap.isOpened():
                self.cap = None
                return False
        return True

    def read(self):
        if self.cap is not None and self.cap.isOpened():
            ret, frame = self.cap.read()
            if ret:
                return frame
        return None

    def stop(self):
        if self.cap is not None:
            self.cap.release()
            self.cap = None
