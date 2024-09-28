#!/usr/bin/env python3
import argparse
import os
import subprocess

TRAINER_BIN = "target/release/train"


def parse_arguments():
    parser = argparse.ArgumentParser(
        description="Train a model using a experiment configuration under experiments/"
    )
    parser.add_argument(
        "model",
        type=str,
        help="Subpath to the experiment model configuration file, e.g. partial-space/wl-rslg-lp",
    )
    parser.add_argument("domain", type=str, help="Domain name")
    return parser.parse_args()


def main():
    args = parse_arguments()

    model_name = args.model.split("/")[-1]

    model_dir = "trained_models"
    if not os.path.exists(model_dir):
        os.makedirs(model_dir)
    model_str = "{}-{}".format(
        model_name.replace("-", "_"), args.domain.replace("-", "_")
    )
    model_path = f"{model_dir}/{model_str}.model"

    cmd = " ".join(
        [
            f"./{TRAINER_BIN}",
            f"--data experiments/domains/{args.domain}.toml",
            f"--model experiments/models/{args.model}.toml",
            f"--save {model_path}",
            "--verbose",
        ]
    )

    log_dir = f"training_logs"
    if not os.path.exists(log_dir):
        os.makedirs(log_dir)
    err_log = log_dir + f"/{model_str}.err"
    out_log = log_dir + f"/{model_str}.out"

    print("WARNING: This script will override previous model and log files")
    print("CMD: ", cmd)
    print("MODEL: ", model_path)
    print("STDOUT: ", out_log)
    print("STDERR: ", err_log)
    subprocess.run(
        cmd, shell=True, stdout=open(out_log, "w"), stderr=open(err_log, "w")
    )


if __name__ == "__main__":
    main()
