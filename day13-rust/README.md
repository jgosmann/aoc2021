# Day 13

Finally a reason to use a `BTreeMap`
to somewhat efficiently iterate over the folded range.
Not sure whether it is actually more efficient than using a simple set.
That would require iterating over all marks for each fold.
However,
the update
(remove and insert of the folded mark)
is in constant time.
With the `BTreeMap` that will become logarithmic,
but one has to iterate over less marks.