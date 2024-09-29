#!/usr/bin/env python3
import argparse
import os
import subprocess

PLANNER_BIN = "target/release/plan"


def parse_arguments():
    parser = argparse.ArgumentParser(
        description="Run the LazyLifted planner on domains in benchmarks/"
    )
    parser.add_argument("search", type=str, choices=["partial-space", "state-space"])
    parser.add_argument(
        "heuristic",
        type=str,
        help="Either the name of the heuristic such as 'hadd', or a model name that starts with 'wl-', e.g. wl-rslg-lp",
    )
    parser.add_argument(
        "domain",
        type=str,
        help="Domain name, e.g. blocksworld, use the -hbf suffix to indicate a high-branching-factor domain",
    )
    parser.add_argument(
        "instance", type=str, help="Instance name, e.g. testing/easy/p30"
    )
    return parser.parse_args()


def main():
    args = parse_arguments()

    instance = args.instance
    domain = args.domain
    if args.domain.endswith("-hbf"):
        domain_dir = f"benchmarks/hbf/{domain[:-4]}"
    else:
        domain_dir = f"benchmarks/ipc23-learning/{domain}"

    if args.heuristic.startswith("wl-"):
        heuristic = "wl"
        model = args.heuristic
        architecture = "{}-{}".format(
            args.search.replace("-", "_"), model.replace("-", "_")
        )
    else:
        heuristic = args.heuristic
        model = None
        architecture = "{}-{}".format(args.search.replace("-", "_"), args.heuristic)

    log_dir = f"planning_logs/{architecture}/{domain}"
    plan_dir = f"plans/{architecture}/{domain}"

    if not os.path.exists(log_dir):
        os.makedirs(log_dir)
    if not os.path.exists(plan_dir):
        os.makedirs(plan_dir)

    instance_str = instance.replace("/", "_")
    plan_file = f"{plan_dir}/{instance_str}.plan"

    cmd = " ".join(
        [
            f"./{PLANNER_BIN}",
            f"{domain_dir}/domain.pddl",
            f"{domain_dir}/{instance}.pddl",
            f"-o {plan_file}",
            "--time-limit 30min",
            "--memory-limit 8000",
        ]
    )
    if model is not None:
        cmd += f" --model trained_models/{model.replace('-', '_')}-{domain.replace('-', '_')}.model"

    cmd += f" {args.search}-search"
    cmd += f" --heuristic {heuristic}"

    out_log = f"{log_dir}/{instance_str}.out"
    err_log = f"{log_dir}/{instance_str}.err"

    print("WARNING: This script will override previous model and log files")
    print("CMD: ", cmd)
    print("PLAN: ", plan_file)
    print("STDOUT: ", out_log)
    print("STDERR: ", err_log)
    subprocess.run(
        cmd, shell=True, stdout=open(out_log, "w"), stderr=open(err_log, "w")
    )


if __name__ == "__main__":
    main()
