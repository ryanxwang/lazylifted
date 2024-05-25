import numpy as np
from sklearn.svm import LinearSVC
from xgboost import XGBRanker


class RankingModel:
    def __init__(self, model_str):
        self.model_str = model_str
        if model_str == "ranksvm":
            # we create the model later
            pass
        elif model_str == "lambdamart":
            raise ValueError("LambdaMART is no longer supported")
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

    def _train_ranksvm(self, X, y):
        C_values = [1e-3, 1e-2, 1e-1, 1, 1e1, 1e2, 1e3]
        X_train, X_val = X[: int(0.8 * len(X))], X[int(0.8 * len(X)) :]
        y_train, y_val = y[: int(0.8 * len(y))], y[int(0.8 * len(y)) :]
        best_score = 0
        best_C = None
        survived_challenges = 0

        for C in C_values:
            print("Training RankSVM with C =", C)
            model = LinearSVC(
                C=C,
                loss="hinge",
                max_iter=20000,
                dual=True,
                fit_intercept=False,
                tol=1e-3,
            )
            model.fit(X_train, y_train)
            score = model.score(X_val, y_val)
            if score > best_score:
                best_score = score
                best_C = C
            else:
                survived_challenges += 1
                if survived_challenges == 2:
                    break

        print("Best C:", best_C)
        model = LinearSVC(
            C=best_C,
            loss="hinge",
            max_iter=20000,
            dual=True,
            fit_intercept=False,
            tol=1e-3,
        )
        model.fit(X, y)
        if model.coef_.shape[0] == 1:
            self.weights = model.coef_[0]
        else:
            self.weights = model.coef_

    def fit(self, X, y, group):
        if self.model_str == "ranksvm":
            X, y = self._to_classification(X, y, group)
            self._train_ranksvm(X, y)
        elif self.model_str == "lambdamart":
            raise ValueError("LambdaMART is no longer supported")
        else:
            raise ValueError("Unknown ranking model: " + self.model_str)

    def predict(self, X):
        if self.model_str == "ranksvm":
            return -np.dot(X, self.weights.T).astype(np.float64)

        elif self.model_str == "lambdamart":
            raise ValueError("LambdaMART is no longer supported")
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
