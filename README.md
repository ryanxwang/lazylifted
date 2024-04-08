# Lazylifted

![build](https://github.com/Thyroidr/lazylifted/actions/workflows/build.yml/badge.svg)

Lazylifted is a domain-independent lifted PDDL planner using utilising graph representations of PDDL structures.

- The parser based on [pddl-rs](https://github.com/sunsided/pddl-rs)
- The successor generation algorithm and planner is based on [powerlifted](https://github.com/abcorrea/powerlifted)
- The graph representations and learning component is based on [GOOSE](https://github.com/DillonZChen/goose)

(See references for more detail)

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
source ./scripts/setup_dynamic_library.sh
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

## Usage

See usage with:

```bash
./trainer --help
```

and

```bash
./planner --help
```

## References

- Corrêa, A. B.; Pommerening, F.; Helmert, M.; and Francès, G. 2020. Lifted Successor Generation using Query Optimization Techniques. In Proc. ICAPS 2020, pp. 80-89. [[pdf]](https://ai.dmi.unibas.ch/papers/correa-et-al-icaps2020.pdf)
- Chen, D. Z.; Trevizan, F. W.; and Thiébaux, S. 2024. Return to Tradition: Learning Reliable Heuristics with Classical Machine Learning. In Proc. ICAPS 2024. [[pdl]](https://openreview.net/pdf?id=zVO8ZRIg7Q)
