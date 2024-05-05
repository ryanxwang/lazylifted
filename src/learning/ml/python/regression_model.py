from sklearn.linear_model import LinearRegression
from sklearn.gaussian_process import GaussianProcessRegressor
from sklearn.gaussian_process.kernels import DotProduct


class RegressionModel:
    def __init__(self, model_str, **kwargs):
        self.model_str = model_str
        if model_str == "lr":
            self.model = LinearRegression()
        elif model_str == "gpr":
            alpha = kwargs["alpha"]
            self.model = GaussianProcessRegressor(kernel=DotProduct(), alpha=alpha)
        else:
            raise ValueError("Unknown regressor model: " + model_str)

    def fit(self, X, y):
        self.model.fit(X, y)
        self.weights = self.get_weights()

    def predict(self, X):
        # for efficiency, we don't use the predict method of the model but
        # compute the prediction directly
        assert hasattr(self, "weights") and self.weights is not None
        return X @ self.weights.T

    def get_weights(self):
        if self.model_str == "lr":
            return self.model.coef_
        elif self.model_str == "gpr":
            # this only works for GPR with DotProduct kernel
            return self.model.alpha_ @ self.model.X_train_

    def __getstate__(self):
        return {
            "model_str": self.model_str,
            "weights": self.weights,
        }

    def __setstate__(self, state):
        self.model_str = state["model_str"]
        self.model = None
        self.weights = state["weights"]
