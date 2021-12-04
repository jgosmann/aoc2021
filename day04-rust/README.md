# Day 4

Today,
I had some more time (weekend!):
Finally a fully unit-tested solution.
With Rust! :)

I decided to not represent the full bingo boards in memory.
Essentially I store a hash map of unmarked numbers to their positions.
This allows to mark numbers efficiently
without searching the board.
To determine,
if a board won,
we only need to keep a count of numbers marked in each column and row.
