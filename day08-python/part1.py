def count_easy_digits(lines):
    segment_counts = [
        len(output.strip())
        for line in lines
        for output in line.split("|")[1].split(" ")
    ]
    return sum(1 for s in segment_counts if s in set((2, 3, 4, 7)))


def test_count_easy_digits():
    import os.path

    with open(os.path.join(os.path.dirname(__file__), "test.input"), "r") as f:
        assert count_easy_digits(f.readlines()) == 26


if __name__ == "__main__":
    import sys

    print(count_easy_digits(sys.stdin.readlines()))
