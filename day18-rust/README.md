# Day 18

Represented the snailfish numbers as trees
with the leaves connected in a double linked list.
This makes the various operations more or less straightforward to implement.
However,
implementing the data structure in Rust was not so straigtforward.
First,
the ownership rules do not allow multiple references
without using `Rc` for explicit reference counting.
Second,
I had some problems performing mutations
until I wrapped almost everything into `RefCell`.
Third,
as I am mutating the data structure in place,
I had to clone it,
but deeply which required a custom `Clone` implementation
to circumvent the `Rc` clone behavior in that case.