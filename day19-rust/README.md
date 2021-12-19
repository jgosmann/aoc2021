# Day 19

This was not quite as bad as it seemed at first.
Implementing methods to rotate and translate a a scanner
with its set of beacons
was not really complicated.
An iterator over all possible orientations was also not too complicated
once you sorted out the correct sequence of rotations needed.
After that,
all there is left to do is to try to align each set of beacons with the map.

My solution could probably be optimized.
For each rotation or translation,
I generate a new set of beacons.
This makes the data structure immutable which eases the implementation a bit,
but might also need to reallocate memory more often.
However,
I am storing the coordinates in hash sets which (I think) cannot be manipulated
in place that easily.
Each coordinate needs to be removed, changed, and reinserted anyways.

## Edit: optimization

I improved the runtime a little bit
by first checking the square of Euclidian distances.
These are invariant under the rotations.
Thus,
I only if at least 12 distances match,
the rotations need to be performed.
This introduces some more overhead to keep track of the distances,
but it is less than the time won.