import numpy as np
from sklearn.svm import LinearSVC
from rank2plan import LpModel, Pair
from rank2plan.metrics import kendall_tau as r2p_kendall_tau
from pulp import PULP_CBC_CMD
import logging
import random
import sys
from scipy.sparse import issparse, dok_matrix


class RankingModel:
    def __init__(self, model_str, C, verbose):
        if verbose:
            logging.basicConfig(
                level=logging.INFO,
                format="%(asctime)s [%(levelname)8s] %(message)s (%(filename)s:%(lineno)s)",
                stream=sys.stdout,
            )

        self.model_str = model_str
        self.verbose = verbose
        self.C = C
        self.feature_dim = None
        if model_str == "ranksvm":
            # we create the model later
            pass
        elif model_str == "lp":
            SECS_PER_MINUTE = 60
            solver = PULP_CBC_CMD(
                msg=False,  # already logging, so no need to print solver messages
                options=[f"RandomS 2024"],
                timeLimit=10 * SECS_PER_MINUTE,
                mip=False,
            )
            random.seed(2024)
            self.model = LpModel(
                solver,
                C=C,
                use_constraint_generation=True,
                use_column_generation=True,
            )
        else:
            raise ValueError("Unknown regressor model: " + model_str)

    def set_feature_dim(self, feature_dim):
        """Set the feature dimension for sparse data."""
        self.feature_dim = feature_dim

    def _to_classification(self, X, pairs):
        X_new = []
        y_new = []
        sample_weight = []
        for i, j, relation, importance in pairs:
            if np.array_equal(X[i], X[j]):
                continue

            # go both ways to make sure the data is balanced
            X_new.append(X[j] - X[i])
            y_new.append(1)
            sample_weight.append(importance)

            X_new.append(X[i] - X[j])
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
            C=self.C,
            tol=1e-3,
        )
        model.fit(X, y, sample_weight=sample_weight)
        if model.coef_.shape[0] == 1:
            self.weights = model.coef_[0]
        else:
            self.weights = model.coef_

    def fit(self, X, pairs):
        """
        Fit the ranking model to the given data.

        Parameters
        ----------
        X : numpy array or a list of dictionaries mapping features to values
            (i.e. a sparse representation). The features of the data

        pairs : list of tuples (i, j, relation), where i and j are indices into
                X and relation is the relation between i and j. The relation is
                an integer describing how much better i is than j. For example,
                if the relation is 1, then i is strictly better than j, if the
                relation is 0, then i is better than or equal to j.
        """
        if isinstance(X, list):
            X = self.dicts_to_csr_matrix(X)

        if self.model_str == "ranksvm":
            data = self._to_classification(X, pairs)
            self._train_ranksvm(*data)
        elif self.model_str == "lp":
            pairs = self._to_rank2plan_pairs(pairs)
            self.model.fit(X, pairs)
            self.weights = self.model.weights()
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def tune(self, X_train, pairs_train, X_val, pairs_val):
        if isinstance(X_train, list):
            X_train = self.dicts_to_csr_matrix(X_train)
        if isinstance(X_val, list):
            X_val = self.dicts_to_csr_matrix(X_val)

        if self.model_str == "lp":
            pairs_train = self._to_rank2plan_pairs(pairs_train)
            pairs_val = self._to_rank2plan_pairs(pairs_val)
            _best_C = self.model.tune(X_train, pairs_train, X_val, pairs_val)
        else:
            raise ValueError("Tuning is not supported for model: " + self.model_str)

    def predict(self, X):
        if isinstance(X, list):
            X = self.dicts_to_csr_matrix(X)

        # We require that all the features come from the same group. In
        # practice, this is okay because as of 2024/07/09, we call predict
        # with a single feature at a time.
        if self.model_str == "ranksvm" or self.model_str == "lp":
            if issparse(X):
                return (X.dot(self.weights.T)).astype(np.float64)
            else:
                return np.dot(X, self.weights.T).astype(np.float64)
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def get_weights(self):
        return self.weights

    def _to_rank2plan_pairs(self, pairs):
        return [
            Pair(i, j, gap=relation, sample_weight=importance)
            for i, j, relation, importance in pairs
        ]

    def kendall_tau(self, X, pairs):
        scores = self.predict(X)
        pairs = self._to_rank2plan_pairs(pairs)
        return r2p_kendall_tau(pairs, scores)

    def dicts_to_csr_matrix(self, dictionaries):
        """
        Convert a list of dictionaries to a CSR matrix.

        Parameters
        ----------
        dictionaries : list of dictionaries
            Each dictionary maps feature indices to values.

        Returns
        -------
        csr_matrix
            A sparse matrix where each row corresponds to a dictionary.
        """
        # First build a DOK matrix, then convert it to CSR format.
        assert self.feature_dim is not None, "Feature dimension must be set."
        dok = dok_matrix((len(dictionaries), self.feature_dim), dtype=np.float64)

        for i, dictionary in enumerate(dictionaries):
            for key, value in dictionary.items():
                dok[i, key] = value

        return dok.tocsr()
