import sys
import time

for i in range(5):
    print(i)
    # flush the output buffer
    sys.stdout.flush()

    # sleep 1
    time.sleep(1)
