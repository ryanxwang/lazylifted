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
        if (
            self.model_str == "gpr"
            and hasattr(self, "weights")
            and self.weights is not None
        ):
            # for efficiency (especially for GPR), use the weights if available
            return X @ self.weights.T
        return self.model.predict(X)

    def get_weights(self):
        if self.model_str == "lr":
            return self.model.coef_
        elif self.model_str == "gpr":
            # this only works for GPR with DotProduct kernel
            return self.model.alpha_ @ self.model.X_train_
