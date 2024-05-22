import numpy as np
from sklearn.svm import LinearSVC
from xgboost import XGBRanker


class RankingModel:
    def __init__(self, model_str):
        self.model_str = model_str
        if model_str == "ranksvm":
            self.model = LinearSVC(
                C=1e-6, loss="hinge", max_iter=9999999, dual=True, fit_intercept=False
            )
        elif model_str == "lambdamart":
            self.model = XGBRanker(
                tree_method="hist", objective="rank:ndcg", lambdarank_pair_method="mean"
            )
        else:
            raise ValueError("Unknown regressor model: " + model_str)

    def _to_classification(self, X, y, group):
        X_new = []
        y_new = []
        start = 0
        for group_size in group:
            end = int(start + group_size)

            for i in range(start, end):
                for j in range(start, end):
                    if y[i] == y[j] or np.array_equal(X[i], X[j]):
                        continue

                    X_new.append(X[i] - X[j])
                    y_new.append(np.sign(y[i] - y[j]))

            start = end

        return np.array(X_new), np.array(y_new)

    def fit(self, X, y, group):
        if self.model_str == "ranksvm":
            X, y = self._to_classification(X, y, group)
            self.model.fit(X, y)
        elif self.model_str == "lambdamart":
            self.model.fit(X, y, group=group)
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def predict(self, X):
        if self.model_str == "ranksvm":
            if self.model.coef_.shape[0] == 1:
                coef = self.model.coef_[0]
            else:
                coef = self.model.coef_

            return -np.dot(X, coef.T).astype(np.float64)

        elif self.model_str == "lambdamart":
            return -self.model.predict(X).astype(np.float64)
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def score(self, X, y, group):
        start = 0
        correct_count = 0

        for group_size in group:
            end = int(start + group_size)
            group_y = y[start:end]
            group_pred = self.predict(X[start:end])

            # we only care if the model picks the correct best item
            if np.argmax(group_y) == np.argmin(group_pred):
                correct_count += 1

            start = end

        return correct_count / len(group)

    def kendall_tau(self, X, y, group):
        start = 0
        concordant_pairs = 0
        discordant_pairs = 0
        total_pairs = 0

        for group_size in group:
            end = int(start + group_size)

            prediction = self.predict(X[start:end])

            for i in range(start, end):
                for j in range(start, end):
                    if y[i] == y[j] or np.array_equal(X[i], X[j]):
                        continue

                    total_pairs += 1

                    if (y[i] - y[j]) * (prediction[i] - prediction[j]) > 0:
                        concordant_pairs += 1
                    elif (y[i] - y[j]) * (prediction[i] - prediction[j]) < 0:
                        discordant_pairs += 1

            start = end

        return (concordant_pairs - discordant_pairs) / total_pairs
