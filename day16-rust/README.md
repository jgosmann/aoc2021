# Day 16

Parsing on the bit level was a bit annoying.
I could have converted everything into a string of 0s and 1s,
but I wanted to try implementing something
that dos not blow up the memory consumption by a factor of 8.
Thus, I had to some bit fiddeling.
At least it is nicely encapsulated in the `BitReader`.
The rest was then a breeze.
Just putting the given rules into code,
reading the correct number of bits
and converting them into the appropriate data structures.
