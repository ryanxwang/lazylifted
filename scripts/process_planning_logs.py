import os
import csv

LOG_DIR = "planning_logs"

def process_stats(s):
    stats = {}
    for field in s.split(' '):
        name, value = field.split('=')
        stats[name] = float(value)
    
    return stats


def process_logs(stderr_lines, stdout_lines):
    def get_content(line):
        return line.split(": ")[1].strip()

    for line_num, line in enumerate(stderr_lines):
        if get_content(line) == "finalising search":
            final_line_num = line_num

    final_stats = process_stats(get_content(stderr_lines[final_line_num + 1]))
    search_duration = process_stats(get_content(stderr_lines[final_line_num + 2]))
    return final_stats | search_duration


def main():
    with open("planning_logs.csv", "w") as csvfile:
        base_fieldnames = ["planner_type", "domain", "instance"]
        data_fieldnames = ["search_duration", "expanded_nodes", "evaluated_nodes", "generated_nodes", "reopened_nodes", "generated_actions", "expanded_preferred_nodes"]
        fieldnames = base_fieldnames + data_fieldnames
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()

        for log_file in os.listdir(LOG_DIR):
            if not os.path.isfile(os.path.join(LOG_DIR, log_file)) or not log_file.endswith(".err"):
                continue

            basename = log_file.replace(".err", "")
            [planner_type, domain, instance] = basename.split("-")
            row = {"planner_type": planner_type, "domain": domain, "instance": instance}
            
            stderr_file = os.path.join(LOG_DIR, log_file)
            stdout_file = stderr_file.replace(".err", ".out")

            with open(stderr_file, "r") as err, open(stdout_file, "r") as out:
                stderr_lines = err.readlines()
                stdout_lines = out.readlines()
                data = process_logs(stderr_lines, stdout_lines)
                
                for key, value in data.items():
                    if key in data_fieldnames:
                        row[key] = value
            
            writer.writerow(row)

if __name__ == "__main__":
    main()