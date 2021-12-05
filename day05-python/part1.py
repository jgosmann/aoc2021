import re
from collections import defaultdict
from dataclasses import dataclass


@dataclass
class Point:
    x: int
    y: int


@dataclass
class Line:
    start: Point
    end: Point


input_line_pattern = re.compile(r"^(\d+),(\d+)\s*->\s*(\d+),(\d+)$")


def parse_input_line(line):
    m = input_line_pattern.match(line)
    return Line(
        start=Point(x=int(m.group(1)), y=int(m.group(2))),
        end=Point(x=int(m.group(3)), y=int(m.group(4))),
    )


class DangerousPointCounter:
    def __init__(self):
        self._counts = defaultdict(lambda: 0)

    def count_line(self, line: Line):
        if line.start.x == line.end.x:
            self._count_vertical_line(line.start.x, line.start.y, line.end.y)
        elif line.start.y == line.end.y:
            self._count_horizontal_line(line.start.x, line.end.x, line.end.y)
        else:
            pass

    def _count_horizontal_line(self, start_x, end_x, y):
        if start_x > end_x:
            start_x, end_x = end_x, start_x
        for x in range(start_x, end_x + 1):
            self._counts[(x, y)] += 1

    def _count_vertical_line(self, x, start_y, end_y):
        if start_y > end_y:
            start_y, end_y = end_y, start_y
        for y in range(start_y, end_y + 1):
            self._counts[(x, y)] += 1

    @property
    def num_of_dangerous_points(self):
        return sum(count > 1 for count in self._counts.values())


def count_dangerous_points(input_lines):
    counter = DangerousPointCounter()
    for line in input_lines:
        parsed_line = parse_input_line(line)
        if parsed_line:
            counter.count_line(parsed_line)
    return counter.num_of_dangerous_points


if __name__ == "__main__":
    import sys

    print(count_dangerous_points(sys.stdin))


def test_parse_input():
    assert parse_input_line("1,23 -> 45,6\n") == Line(
        start=Point(1, 23), end=Point(45, 6)
    )


def test_number_of_dangerous_points():
    # ....1.....
    # ....1.....
    # ....1.....
    # ....2.....
    # 1111311111
    # ....2.....
    # ....1.....
    # ....1.....
    # ....1.....
    # ....1.....
    counter = DangerousPointCounter()
    assert counter.num_of_dangerous_points == 0
    counter.count_line(Line(start=Point(5, 1), end=Point(5, 10)))
    assert counter.num_of_dangerous_points == 0
    counter.count_line(Line(start=Point(1, 5), end=Point(10, 5)))
    assert counter.num_of_dangerous_points == 1
    counter.count_line(Line(start=Point(5, 6), end=Point(5, 4)))
    assert counter.num_of_dangerous_points == 3
    counter.count_line(Line(start=Point(4, 2), end=Point(7, 5)))
    assert counter.num_of_dangerous_points == 3


def test_count_dangerous_points():
    with open("test.input", "r") as f:
        assert count_dangerous_points(f) == 5
