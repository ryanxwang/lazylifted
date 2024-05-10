from argparse import ArgumentParser
import csv
from dataclasses import dataclass
import os
from typing import Optional, List

from processed_log import process_log


SEARCH_TIME_LIMIT = 1800 # 30 minutes


@dataclass
class PlanningResult:
    domain: str
    instance: str
    model_type: str
    model: str
    found_plan: bool
    search_duration: float
    expanded_nodes: int
    evaluated_nodes: int
    generated_nodes: int
    reopened_nodes: int
    generated_actions: int
    improving_expansions: int
    plan_length: Optional[int]


def save_results(results: List[PlanningResult], output_file):
    field_names = list(results[0].__dict__.keys())
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
        if not log_file.endswith(".err"):
            continue
        log_path = os.path.join(args.log_dir, log_file)

        processed_log = process_log(log_path)

        print(f"Processing found log file {log_file}")

        found_plan = False
        plan_length = None
        search_duration = None
        expanded_nodes = None
        evaluated_nodes = None
        generated_nodes = None
        reopened_nodes = None
        generated_actions = None
        improving_expansions = None
        for key, value_entries in processed_log.values.items():
            if key == "plan_length" and value_entries:
                found_plan = True
                plan_length = value_entries[-1].value
            if key == "search_duration" and value_entries:
                search_duration = value_entries[-1].value
            if key == "expanded_nodes" and value_entries:
                expanded_nodes = value_entries[-1].value
            if key == "evaluated_nodes" and value_entries:
                evaluated_nodes = value_entries[-1].value
            if key == "generated_nodes" and value_entries:
                generated_nodes = value_entries[-1].value
            if key == "reopened_nodes" and value_entries:
                reopened_nodes = value_entries[-1].value
            if key == "generated_actions" and value_entries:
                generated_actions = value_entries[-1].value
            if key == "improving_expansions" and value_entries:
                improving_expansions = value_entries[-1].value
        
        if search_duration is None:
            search_duration = SEARCH_TIME_LIMIT
        assert(search_duration <= SEARCH_TIME_LIMIT)
        
        results.append(
            PlanningResult(
                domain=processed_log.domain,
                instance=processed_log.instance,
                model_type=processed_log.model_type,
                model=processed_log.model,
                found_plan=found_plan,
                search_duration=search_duration,
                expanded_nodes=expanded_nodes,
                evaluated_nodes=evaluated_nodes,
                generated_nodes=generated_nodes,
                reopened_nodes=reopened_nodes,
                generated_actions=generated_actions,
                improving_expansions=improving_expansions,
                plan_length=plan_length,
            )
        )

    def instance_value(instance):
        difficulty = instance.split('_')[1]
        instance_value = int(instance.split('_')[2].strip('p'))
        if difficulty == "easy":
            return instance_value
        elif difficulty == "medium":
            return instance_value + 30
        elif difficulty == "hard":
            return instance_value + 60
            
    results.sort(key=lambda x: instance_value(x.instance))

    save_results(results, args.output_file)


if __name__ == "__main__":
    main()
