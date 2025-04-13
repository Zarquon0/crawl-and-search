from datetime import datetime

def log(msg):
    now = datetime.now()
    with open('log.txt', 'a') as fl:
        fl.write(f"{{{now}}}: {msg}\n")