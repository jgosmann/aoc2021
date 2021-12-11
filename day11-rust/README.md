# Day 11

Not quite production quality Rust code today.
The number 10 is hard coded in several places.
However,
this allowed me to use a u128 as bitmap for locations that already flashed
without having to deal with multiple variables or some external library.

I again used a neighbors iterator.
This time included the diagonal positions.
It probably could be written using less code.