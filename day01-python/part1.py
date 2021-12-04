import numpy as np
import sys

sonar_readings = np.array([int(x) for x in sys.stdin.readlines()])
print(np.sum(np.diff(sonar_readings) > 0))
