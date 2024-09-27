# LazyLifted

![build](https://github.com/Thyroidr/lazylifted/actions/workflows/build.yml/badge.svg)

LazyLifted (LL) is a domain-independent lifted PDDL planner utilising graph
representations of PDDL structures. It performs state space search (traditional)
and *partial space search* (new).

- The parser based on [pddl-rs](https://github.com/sunsided/pddl-rs)
- The graph representations and learning component is based on
  [GOOSE](https://github.com/DillonZChen/goose)
- The successor generator and delete relaxation heuristics are based on
  [powerlifted](https://github.com/abcorrea/powerlifted)

(See [references](#references) for a list of all the papers Lazylifted is based
on)

## Installation

1. Install Rust and Cargo according to official documentations. See `cargo.toml`
   for the minimum supported rust version of this project.
2. Set up a Python virtual environment and install the dependencies listed in
   `requirements.txt`.
3. Run `source scripts/setup_dynamic_library.sh`. This will add the Python dynamic
   libraries to the dynamic library path environment variable. This might not be
   necessary if you are not using a virtual environment. See
    [pyo3](https://pyo3.rs/v0.15.0/building_and_distribution.html#dynamically-embedding-the-python-interpreter)
    documentation and the [cargo book](https://doc.rust-lang.org/cargo/reference/environment-variables.html#dynamic-library-paths)
    for more details.
4. Build with `cargo build --release`. Note that the built binary will only be
   compatible with the Python version (e.g. 3.11) of the environment that you
   build in.

## Usage

See usage with:

```bash
./target/release/trainer --help
```

and

```bash
./target/release/planner --help
```

For convenience, you can also use `scripts/plan.sh` and `scripts/train.sh`.

## Example

To train WL-ILG-GPR, run

```bash
./scripts/train.sh state-space wl-ilg-gpr ferry
```

Then run

```bash
./scripts/plan.sh state-space wl-ilg-gpr ferry testing/easy/p30
```

## Development

When developping in VS Code, it is useful to add the following to
`settings.json` to allow the VS Code extensions to work properly. The path
`/opt/homebrew/anaconda3/lib` should be replaced with the local folder
containing the Python shared library, and the environment variable
`DYLD_FALLBACK_LIBRARY_PATH` should also be replaced the appropriate one for the
OS.

```json
"rust-analyzer.cargo.extraEnv": {
    "DYLD_FALLBACK_LIBRARY_PATH": "/opt/homebrew/anaconda3/lib"
},
"rust-analyzer.runnables.extraEnv": {
    "DYLD_FALLBACK_LIBRARY_PATH": "/opt/homebrew/anaconda3/lib"
},
"lldb.launch.env": {
    "DYLD_FALLBACK_LIBRARY_PATH": "/opt/homebrew/anaconda3/lib"
},
```

## References

Below is a list of works that LazyLifted is based on, sorted in chronological
order.

- Kovacs, D. L. 2011. Complete BNF Description of PDDL 3.1 (Completely
  Corrected).
  [[pdf]](https://helios.hud.ac.uk/scommv/IPC-14/repository/kovacs-pddl-3.1-2011.pdf)
- Corrêa, A. B.; Pommerening, F.; Helmert, M.; and Francès, G. 2020. Lifted
  Successor Generation using Query Optimization Techniques. In Proc. ICAPS 2020,
  pp. 80-89. [[pdf]](https://ai.dmi.unibas.ch/papers/correa-et-al-icaps2020.pdf)
- Corrêa, A. B.; Francès, G.; Pommerening, F.; and Helmert, M. 2021.
  Delete-Relaxation Heuristics for Lifted Classical Planning. In Proc. ICAPS
  2021, pp. 94-102.
  [[pdf]](https://ai.dmi.unibas.ch/papers/correa-et-al-icaps2021.pdf)
- Corrêa, A. B.; Pommerening, F.; Helmert, M.; and Francès, G. 2022. The FF
  Heuristic for Lifted Classical Planning. In Proc. AAAI 2022.
  [[pdf]](https://ai.dmi.unibas.ch/papers/correa-et-al-aaai2022.pdf)
- Chen, D. Z.; Trevizan, F. W.; and Thiébaux, S. 2024. Return to Tradition:
  Learning Reliable Heuristics with Classical Machine Learning. In Proc. ICAPS
  2024. [[pdf]](https://openreview.net/pdf?id=zVO8ZRIg7Q)
- Hao, M.; Trevizan, F.W.; Thiébaux, S.; Ferber, P.; and Hoffmann, J. 2024.
  Guilding GBFS through Learned Pairwise Rankings. In Proc. IJCAI 2024.
  [[pdf]](https://felipe.trevizan.org/papers/hao24:ranking.pdf)
