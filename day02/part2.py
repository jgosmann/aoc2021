import sys

class Submarine:
    def __init__(self):
        self.horizontal_pos = 0
        self.depth = 0
        self.aim = 0

    def execute(self, command, arg):
        getattr(self, command)(arg)

    def forward(self, distance):
        self.horizontal_pos += distance
        self.depth += self.aim * distance

    def up(self, rise):
        self.aim -= rise

    def down(self, fall):
        self.aim += fall

submarine = Submarine()

for line in sys.stdin.readlines():
    command, arg = line.split()
    submarine.execute(command, int(arg))

print(submarine.horizontal_pos * submarine.depth)
