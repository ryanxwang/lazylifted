from sklearn.linear_model import LinearRegression
from sklearn.gaussian_process import GaussianProcessRegressor
from sklearn.gaussian_process.kernels import DotProduct


class RegressionModel:
    def __init__(self, model_str):
        self.model_str = model_str
        if model_str == "lr":
            self.model = LinearRegression()
        elif model_str == "gpr":
            self.model = GaussianProcessRegressor(kernel=DotProduct(), alpha=1e-7)
        else:
            raise ValueError("Unknown regressor model: " + model_str)

    def fit(self, X, y):
        self.model.fit(X, y)

    def predict(self, X):
        return self.model.predict(X)

    def get_weights(self):
        if self.model_str == "lr":
            return self.model.coef_
        elif self.model_str == "gpr":
            return self.model.alpha_ @ self.model.X_train_
