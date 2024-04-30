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
            raise ValueError("Unknown regressor model: " + self.model_str)

    def predict(self, X):
        if self.model_str == "ranksvm":
            raise NotImplementedError("RankSVM not implemented yet")
        elif self.model_str == "lambdamart":
            return self.model.predict(X).astype(np.float64)
        else:
            raise ValueError("Unknown regressor model: " + self.model_str)
