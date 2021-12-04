import numpy as np
import sys

sonar_readings = np.array([int(x) for x in sys.stdin.readlines()])
windowed = sonar_readings[0:-2] + sonar_readings[1:-1] + sonar_readings[2:]
print(np.sum(np.diff(windowed) > 0))
