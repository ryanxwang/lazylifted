import numpy as np
from sklearn.svm import LinearSVC
from collections import defaultdict
from uuid import uuid4


class RankingModel:
    def __init__(self, model_str, verbose):
        self.model_str = model_str
        self.verbose = verbose
        if model_str == "ranksvm":
            # we create the model later
            pass
        elif model_str == "lp":
            # we create the model later
            pass
        elif model_str == "lambdamart":
            raise ValueError("LambdaMART is no longer supported")
        else:
            raise ValueError("Unknown regressor model: " + model_str)

    def _to_classification(self, X, pairs):
        X_new = []
        y_new = []
        sample_weight = []
        for i, j, relation, importance in pairs:
            if np.array_equal(X[i], X[j]):
                continue

            # go both ways to make sure the data is balanced
            X_new.append(X[i] - X[j])
            y_new.append(1)
            sample_weight.append(importance)

            X_new.append(X[j] - X[i])
            y_new.append(-1)
            sample_weight.append(importance)

        return np.array(X_new), np.array(y_new), np.array(sample_weight)

    def _train_ranksvm(self, X, y, sample_weight):
        model = LinearSVC(
            penalty="l1",
            loss="squared_hinge",
            max_iter=30000,
            dual=False,
            fit_intercept=False,
            C=1,
            tol=1e-3,
        )
        model.fit(X, y, sample_weight=sample_weight)
        if model.coef_.shape[0] == 1:
            self.weights = model.coef_[0]
        else:
            self.weights = model.coef_

    def fit(self, X, pairs, group_ids):
        """
        Fit the ranking model to the given data.

        Parameters
        ----------
        X : numpy array
            The features of the data
        pairs : list of tuples (i, j, relation), where i and j are indices into
            X and relation is the relation between i and j. The relation is
            an integer describing how much better i is than j. For example, if
            the relation is 1, then i is strictly better than j, if the relation
            is 0, then i is better than or equal to j.
        group_ids : list of integers indicating the group of each feature. Some
            models may use this information to specialise the weights for each
            group.
        """
        if self.model_str == "ranksvm":
            data = self._to_classification(X, pairs)
            self._train_ranksvm(*data)
        elif self.model_str == "lp":
            lp = LP()
            lp.fit(X, pairs, group_ids, verbose=self.verbose)
            self.weights = lp.weights
        elif self.model_str == "lambdamart":
            raise ValueError("LambdaMART is no longer supported")
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def predict(self, X, group_id):
        # We require that all the features come from the same group. In
        # practice, this is okay because as of 2024/07/09, we call predict
        # with a single feature at a time.
        if self.model_str == "ranksvm" or self.model_str == "lp":
            if type(self.weights) == dict:
                # this means we never saw this group during training, we can
                # only assume it's the worst
                if group_id not in self.weights:
                    return np.array([1000000.0])

                # Can only predict for a single group on batch
                return -np.dot(X, self.weights[group_id].T).astype(np.float64)
            else:
                return -np.dot(X, self.weights.T).astype(np.float64)
        elif self.model_str == "lambdamart":
            raise ValueError("LambdaMART is no longer supported")
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def get_weights(self):
        return self.weights

    def kendall_tau(self, X, pairs, group_ids):
        concordant_pairs = 0
        discordant_pairs = 0
        total_pairs = 0

        for i, j, relation, importance in pairs:
            if np.array_equal(X[i], X[j]):
                continue

            total_pairs += importance

            i_heuristic = self.predict(
                X[i], group_ids[i] if group_ids is not None else None
            )
            j_heuristic = self.predict(
                X[j], group_ids[j] if group_ids is not None else None
            )
            diff = i_heuristic - j_heuristic
            assert relation >= 0
            if (relation > 0 and diff < 0) or (relation == 0 and diff <= 0):
                concordant_pairs += importance
            else:
                discordant_pairs += importance

        return (concordant_pairs - discordant_pairs) / total_pairs


from pulp import *


class LP:
    def __init__(self):
        pass

    def fit(self, X, pairs, group_ids, C=10, verbose=False):
        prob = LpProblem("Ranking", LpMinimize)

        weights = defaultdict(list)
        abs_weights = defaultdict(list)

        is_using_groups = group_ids is not None
        if group_ids is None:
            MOCK_GROUP_ID = 0
            group_ids = [MOCK_GROUP_ID] * X.shape[0]

        for group_id in set(group_ids):
            for i in range(X.shape[1]):
                # Dillon constrains the weights to be in {-1, 0, 1}, but it seems
                # for us that takes too long to solve, so we make this an LP
                weights[group_id].append(
                    LpVariable(f"w({group_id})({i})", cat=LpContinuous)
                )

            abs_weights[group_id] = [
                LpVariable(f"abs_w({group_id})({i})", lowBound=0)
                for i in range(X.shape[1])
            ]
            for i in range(X.shape[1]):
                prob += abs_weights[group_id][i] >= weights[group_id][i]
                prob += abs_weights[group_id][i] >= -weights[group_id][i]

        slacks = []
        seen_pairs = set()
        for i, j, relation, importance in pairs:
            if np.array_equal(X[i], X[j]):
                continue

            # generally, we try and make variable names concise and meaningful,
            # but in the rare case of duplicate relation between variables, we
            # just add a UUID to the end
            if (i, j) in seen_pairs:
                slack = LpVariable(f"z{i}_{j}_{uuid4()}", lowBound=0)
            else:
                slack = LpVariable(f"z{i}_{j}", lowBound=0)
            slacks.append((slack, importance))

            prob += (
                lpSum(
                    weights[group_ids[i]][k] * X[i][k]
                    - weights[group_ids[j]][k] * X[j][k]
                    for k in range(X.shape[1])
                )
                >= relation - slack
            )
            seen_pairs.add((i, j))

        prob += C * lpSum(importance * slack for (slack, importance) in slacks) + lpSum(
            abs_weights[group_id][i]
            for i in range(X.shape[1])
            for group_id in set(group_ids)
        )

        # solver_list = listSolvers(onlyAvailable=True) if "CPLEX_PY" not in

        # fix seed for deterministic results, note that 0 results in using time
        # of day
        solver = PULP_CBC_CMD(msg=verbose, options=[f"RandomS 2024"])
        # else:
        # solver = CPLEX_PY(msg=verbose)
        prob.solve(solver)

        if is_using_groups:
            self.weights = {
                group_id: np.array([w.varValue for w in weights[group_id]])
                for group_id in set(group_ids)
            }
        else:
            self.weights = np.array([w.varValue for w in weights[MOCK_GROUP_ID]])
