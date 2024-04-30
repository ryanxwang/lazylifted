pub const BLOCKSWORLD_DOMAIN_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/benchmarks/ipc23-learning/blocksworld/domain.pddl"
));

pub const BLOCKSWORLD_PROBLEM13_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/benchmarks/ipc23-learning/blocksworld/training/easy/p13.pddl"
));

pub const SPANNER_DOMAIN_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/benchmarks/ipc23-learning/spanner/domain.pddl"
));

pub const SPANNER_PROBLEM10_TEXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/benchmarks/ipc23-learning/spanner/testing/easy/p10.pddl"
));
