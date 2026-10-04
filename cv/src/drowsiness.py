import math

# Eye landmark indices based on MediaPipe FaceMesh
LEFT_EYE = [33, 160, 158, 133, 153, 144]
RIGHT_EYE = [362, 385, 387, 263, 373, 380]

def euclidean_distance(p1, p2):
    return math.sqrt((p1.x - p2.x)**2 + (p1.y - p2.y)**2)

def calculate_ear(eye_landmarks, all_landmarks):
    # p1, p2, p3, p4, p5, p6
    # p1 to p4 is the horizontal distance
    # p2 to p6 and p3 to p5 are the vertical distances
    try:
        p1 = all_landmarks.landmark[eye_landmarks[0]]
        p2 = all_landmarks.landmark[eye_landmarks[1]]
        p3 = all_landmarks.landmark[eye_landmarks[2]]
        p4 = all_landmarks.landmark[eye_landmarks[3]]
        p5 = all_landmarks.landmark[eye_landmarks[4]]
        p6 = all_landmarks.landmark[eye_landmarks[5]]

        horiz = euclidean_distance(p1, p4)
        vert1 = euclidean_distance(p2, p6)
        vert2 = euclidean_distance(p3, p5)

        if horiz == 0:
            return 0.0

        ear = (vert1 + vert2) / (2.0 * horiz)
        return ear
    except Exception:
        return 0.0

def process_landmarks(face_landmarks):
    left_ear = calculate_ear(LEFT_EYE, face_landmarks)
    right_ear = calculate_ear(RIGHT_EYE, face_landmarks)
    combined = (left_ear + right_ear) / 2.0
    return left_ear, right_ear, combined
