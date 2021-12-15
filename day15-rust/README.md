# Day 15

Dijkstra!

Reused my neighbor iterator implementation from a few days ago,
and improved the API a little bit with the `Borrow` trait.

Using a `HashSet` to store touched vertices might not be quite optimal;
maybe a grid of booleans or a bitmap would be more efficient,
but it is still fast enough
(and it isn't a difference in the asymptotic complexity).
A similar point applies to the `HashMap` for `prev_node`.
However, using `HashSet` and `HashMap` was easier to implement API-wise.