# lazylifted


![build](https://github.com/Thyroidr/lazylifted/actions/workflows/rust.yml/badge.svg)

## Installation

Lazylifted is written in rust, and can be built with any recent version of
cargo. The produced binaries (and any tests) will [dynamically link to a Python
shared
library](https://pyo3.rs/v0.15.0/building_and_distribution.html#dynamically-embedding-the-python-interpreter),
and the location it looks for can be configured by adding to the environment
variables described in the [cargo
book](https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths).
Running the following command should set this up for you on Unix systems.

```bash
source ./setup_dynamic_library.sh
```

When developping in VS Code, it is useful to add the following to
`settings.json` to allow the VS Code extensions to work properly (replacing
`/opt/homebrew/anaconda3/lib` with the local folder containing the Python shared
library).

```json
"rust-analyzer.cargo.extraEnv": {
    "RUSTFLAGS": "-C link-args=-Wl,-rpath,/opt/homebrew/anaconda3/lib"
},
"rust-analyzer.runnables.extraEnv": {
    "RUSTFLAGS": "-C link-args=-Wl,-rpath,/opt/homebrew/anaconda3/lib"
},
```
