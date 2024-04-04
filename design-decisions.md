# Design Decisions

There is a lot of room to explore in this project. This file documents and
discusses some of these design decisions as well as results from trialing out
some of these options so that we can backtrack if something doesn't work.

## Ranking algorithm

This is the actual ML algorithm we use to rank things. So far we've explored
RankSVM and LambdaMart. RankSVM is faster to train but produces worse results
than LambdaMart. LamdbdaMart however suffers from a lot of overfitting.

Currently going with: LambdaMart

## Grounding before or after ranking

This refers to whether we instantiate the applicable actions at a state before
or after calling the ranktree model. The benefit of instantiating before is that
the model has access to this applicability information, which avoids it having
to compare between things that would otherwise be found inapplicable only in the
end. The tradeoff is that if we ground after, we only need to ground lazily and
only ground the partial action that the model deems useful. In otherwords, we
are trading between (in the ground before case) fewer calls to the ML model and
(in the ground after case) less grounding.

There is an middle option here as well. At each layer of the ranktree, we just
need to know which edges to the next layer are inapplicable. For example, at the
root layer, we only care which of the action schemas are applicable (i.e. have
an applicable instantiation). If we can efficiently approximate or compute this,
then we can get the best of both worlds. However, in an approximation approach,
this sort of also means we have to ground twice, once when approximating, and
once in the end.

The ultimate answer to this decision is probably to just implement the easiet
option then profile the code to see if there is a need to improve on this. Also
worth noting that none of this discussion takes into account the option of
batching multiple edges into one, i.e. ranking sets of objects.

Currently going with: grounding before, it is the easiest thing to implement.
