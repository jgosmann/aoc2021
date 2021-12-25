# Day 24

I found this to be the most difficult problem this year.
I probably also over-engineered this a bit.
First,
I parse the instruction set.
Then I run a symbolic ALU to construct an AST.
While doing so,
I also apply some optimizations 
(e.g. cut off branches where a value is multiplied with 0).
This works pretty nicely with the pattern matching of Rust.
You can just add the special cases next to the more general cases.
Then I take the AST and construct an AST representation
where shared nodes are deduplicated
and have a topological sorting according to their interdependencies.
On this representation the search algorithm is then run.