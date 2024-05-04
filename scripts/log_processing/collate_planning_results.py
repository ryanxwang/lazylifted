from argparse import ArgumentParser
import csv
from dataclasses import dataclass
import os
from typing import Optional, List

from processed_log import process_log


@dataclass
class PlanningResult:
    domain: str
    instance: str
    model_type: str
    model: str
    found_plan: bool
    plan_length: Optional[int]


def save_results(results: List[PlanningResult], output_file):
    field_names = [
        "domain",
        "instance",
        "model_type",
        "model",
        "found_plan",
        "plan_length",
    ]
    with open(output_file, "w") as f:
        writer = csv.DictWriter(f, fieldnames=field_names)
        writer.writeheader()
        for result in results:
            writer.writerow(result.__dict__)


def parse_args():
    parser = ArgumentParser(description="Collate planning results")
    parser.add_argument("log_dir", type=str, help="Directory containing log files")
    parser.add_argument("output_file", type=str, help="Output CSV file")
    return parser.parse_args()


def main():
    args = parse_args()

    results = []
    for log_file in os.listdir(args.log_dir):
        log_path = os.path.join(args.log_dir, log_file)

        try:
            processed_log = process_log(log_path)
        except Exception as e:
            continue

        print(f"Processing found log file {log_file}")

        found_plan = False
        plan_length = None
        for key, value_entries in processed_log.values.items():
            if key == "plan_length" and value_entries:
                found_plan = True
                plan_length = value_entries[-1].value
                break

        results.append(
            PlanningResult(
                domain=processed_log.domain,
                instance=processed_log.instance,
                model_type=processed_log.model_type,
                model=processed_log.model,
                found_plan=found_plan,
                plan_length=plan_length,
            )
        )

    save_results(results, args.output_file)


if __name__ == "__main__":
    main()
