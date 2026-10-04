import json
import sys

def send_message(msg_type, **kwargs):
    message = {"type": msg_type}
    message.update(kwargs)
    try:
        print(json.dumps(message), flush=True)
    except Exception:
        pass

def parse_message(line):
    try:
        return json.loads(line.strip())
    except Exception:
        return None
