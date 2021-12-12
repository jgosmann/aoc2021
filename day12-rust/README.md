# Day 12

Pretty basic depth first search,
using a recursive implementation.
The graph representation could probably be made a bit more memory efficient.
Currently, I am storing each string multiple times in memory.
But to use references,
something else with a sufficient lifetime needs to be given ownership of the strings.
