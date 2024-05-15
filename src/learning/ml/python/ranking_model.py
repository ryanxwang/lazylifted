import numpy as np
from sklearn.svm import LinearSVC
from xgboost import XGBRanker


class RankingModel:
    def __init__(self, model_str):
        self.model_str = model_str
        if model_str == "ranksvm":
            self.model = LinearSVC(
                C=1e-6, loss="hinge", max_iter=9999999, dual="auto", fit_intercept=False
            )
        elif model_str == "lambdamart":
            self.model = XGBRanker(
                tree_method="hist", objective="rank:ndcg", lambdarank_pair_method="mean"
            )
        else:
            raise ValueError("Unknown regressor model: " + model_str)

    def fit(self, X, y, group):
        if self.model_str == "ranksvm":
            raise NotImplementedError("RankSVM not implemented yet")
        elif self.model_str == "lambdamart":
            self.model.fit(X, y, group=group)
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def predict(self, X):
        if self.model_str == "ranksvm":
            raise NotImplementedError("RankSVM not implemented yet")
        elif self.model_str == "lambdamart":
            return self.model.predict(X).astype(np.float64)
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
            if np.argmax(group_y) == np.argmax(group_pred):
                correct_count += 1

            start = end

        return correct_count / len(group)
