# Design Decisions

There is a lot of room to explore in this project. This file documents and
discusses some of these design decisions as well as results from trialing out
some of these options so that we can backtrack if something doesn't work.

## Ranking algorithm

This is the actual ML algorithm we use to rank things. So far we've explored
RankSVM and LambdaMart. RankSVM is faster to train but produces worse results
than LambdaMart. LamdbdaMart however suffers from a lot of overfitting.

Currently going with: LambdaMart

## Model training granularity

This refers to how we should partition the data when training the ranking models
making up the tree. The options are

1. Train a single model with all the data. This is the simplest and allows the
   rank produced to be global. The downside is that the model may not be able to
   specialise.
2. Train specialised models by partition data using domain structure. This
   mainly refers to that we can train a single model for the action schema
   schema ranking, then specialise lower level nodes with nodels trained
   specifically on data belonging to that action schema. Models trained this way
   do not produce global rankings and care has to be take for that.
  
Ultimately, this comes down to which one can perform better. I am going with the
single model approach first, as this allows that single model to see more data
and potentially generalise better. It's also much, much nicer to have a global
ranking.

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

Update: We found out that grounding takes less than 1% of total runtime when
running WL-ILG with GBFS. In this case the right choice here is clearly
grounding before.
