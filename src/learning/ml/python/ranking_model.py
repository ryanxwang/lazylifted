import numpy as np
import sys
from sklearn.exceptions import ConvergenceWarning
from sklearn.svm import LinearSVC
from sklearn.experimental import enable_halving_search_cv  # noqa
from sklearn.model_selection import HalvingGridSearchCV
import warnings


class RankingModel:
    def __init__(self, model_str):
        self.model_str = model_str
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
        for i, j, relation in pairs:
            if np.array_equal(X[i], X[j]):
                continue

            # go both ways to make sure the data is balanced
            X_new.append(X[i] - X[j])
            y_new.append(np.sign(1))

            X_new.append(X[j] - X[i])
            y_new.append(np.sign(-1))

        return np.array(X_new), np.array(y_new)

    def _train_ranksvm(self, X, y):
        model = LinearSVC(
            loss="hinge",
            max_iter=30000,
            dual=True,
            fit_intercept=False,
            C=1,
            tol=1e-3,
        )
        model.fit(X, y)
        if model.coef_.shape[0] == 1:
            self.weights = model.coef_[0]
        else:
            self.weights = model.coef_

    def fit(self, X, pairs):
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
        """
        if self.model_str == "ranksvm":
            X, y = self._to_classification(X, pairs)
            self._train_ranksvm(X, y)
        elif self.model_str == "lp":
            lp = LP()
            lp.fit(X, pairs)
            self.weights = lp.weights
        elif self.model_str == "lambdamart":
            raise ValueError("LambdaMART is no longer supported")
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def predict(self, X):
        if self.model_str == "ranksvm" or self.model_str == "lp":
            return -np.dot(X, self.weights.T).astype(np.float64)

        elif self.model_str == "lambdamart":
            raise ValueError("LambdaMART is no longer supported")
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def kendall_tau(self, X, pairs):
        concordant_pairs = 0
        discordant_pairs = 0
        total_pairs = 0

        for i, j, relation in pairs:
            if np.array_equal(X[i], X[j]):
                continue

            total_pairs += 1

            diff = self.predict(X[i]) - self.predict(X[j])
            assert relation >= 0
            if (relation > 0 and diff < 0) or (relation == 0 and diff <= 0):
                concordant_pairs += 1
            else:
                discordant_pairs += 1

        return (concordant_pairs - discordant_pairs) / total_pairs


from pulp import *


class LP:
    def __init__(self):
        pass

    def fit(self, X, pairs, C=1.0):
        prob = LpProblem("Ranking", LpMinimize)

        weights = []
        for i in range(X.shape[1]):
            # Dillon constrains the weights to be in {-1, 0, 1}, but it seems
            # for us that takes too long to solve, so we make this an LP
            weights.append(LpVariable("w" + str(i), cat=LpContinuous))

        abs_weights = [
            LpVariable("abs_w" + str(i), lowBound=0) for i in range(X.shape[1])
        ]
        for i in range(X.shape[1]):
            prob += abs_weights[i] >= weights[i]
            prob += abs_weights[i] >= -weights[i]

        slacks = []
        for i, j, relation in pairs:
            if np.array_equal(X[i], X[j]):
                continue

            slack = LpVariable("z" + str(i) + "_" + str(j), lowBound=0)
            slacks.append(slack)

            prob += (
                lpSum(weights[k] * (X[i][k] - X[j][k]) for k in range(X.shape[1]))
                >= relation - slack
            )

        prob += C * lpSum(slacks) + lpSum(abs_weights)

        solver_list = listSolvers(onlyAvailable=True)
        if "CPLEX_PY" not in solver_list:
            solver = PULP_CBC_CMD(msg=False, gapRel=0.01)
        else:
            solver = CPLEX_PY(msg=False, gapRel=0.01)

        prob.solve(solver)

        self.weights = np.array([w.varValue for w in weights])
