# Day 6

Easily solved by realizing the cyclic nature.
For each day in the week,
the number of fish spawning needs to be stored.
In addition,
we have a double-ended queue
for fish that cannot spawn yet.
Each day we pop from the queue,
and add the count to current day of the week.