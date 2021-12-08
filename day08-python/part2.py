from collections import Counter
from typing import Sequence, Set


def decode_mapping(signal_patterns: Sequence[Set]):
    segment_counts = Counter(
        segment for pattern in signal_patterns for segment in pattern
    )
    count_mapped_to_segment = {
        count: segment for segment, count in segment_counts.items()
    }
    mapping = {
        count_mapped_to_segment[4]: "e",
        count_mapped_to_segment[6]: "b",
        count_mapped_to_segment[9]: "f",
    }

    digit_one_pattern = [s for s in signal_patterns if len(s) == 2][0]
    mapping[next(iter(digit_one_pattern - set(count_mapped_to_segment[9])))] = "c"

    digit_seven_pattern = [s for s in signal_patterns if len(s) == 3][0]
    mapping[next(iter(digit_seven_pattern - digit_one_pattern))] = "a"

    digit_eight_pattern = [s for s in signal_patterns if len(s) == 7][0]
    digit_four_pattern = [s for s in signal_patterns if len(s) == 4][0]
    mapping[
        next(iter(digit_eight_pattern - digit_four_pattern - set(mapping.keys())))
    ] = "g"
    mapping[next(iter(set("abcdefg") - set(mapping.keys())))] = "d"

    return mapping


digit_decoder = {
    frozenset("abcefg"): 0,
    frozenset("cf"): 1,
    frozenset("acdeg"): 2,
    frozenset("acdfg"): 3,
    frozenset("bcdf"): 4,
    frozenset("abdfg"): 5,
    frozenset("abdefg"): 6,
    frozenset("acf"): 7,
    frozenset("abcdefg"): 8,
    frozenset("abcdfg"): 9,
}


def decode_digit(mapping, digit):
    return digit_decoder[frozenset(mapping[s] for s in digit)]


def decode_output(mapping, output):
    decoded_digits = [decode_digit(mapping, digit) for digit in output]
    acc = 0
    for decoded_digit in decoded_digits:
        acc *= 10
        acc += decoded_digit
    return acc


def decode_line(line):
    pattern, output = line.split("|")
    mapping = decode_mapping([frozenset(p.strip()) for p in pattern.split(" ")])
    return decode_output(mapping, [o.strip() for o in output.split(" ") if o.strip()])


def sum_of_four_digit_displays(lines):
    return sum(decode_line(line) for line in lines)


def test_sum_of_four_digit_displays():
    import os.path

    with open(os.path.join(os.path.dirname(__file__), "test.input"), "r") as f:
        assert sum_of_four_digit_displays(f.readlines()) == 61229


if __name__ == "__main__":
    import sys

    print(sum_of_four_digit_displays(sys.stdin.readlines()))
