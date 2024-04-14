import os
import csv

LOG_DIR = "training_logs"

def process_stats(s):
    stats = {}
    for field in s.split(' '):
        name, value = field.split('=')
        stats[name] = float(value)
    
    return stats


def process_logs(stderr_lines, stdout_lines):
    def get_content(line):
        return line.split(": ")[1].strip()
    
    stats = {}
    useful_prefixes = [
        "fitting_time", "total_colours",
        "val_score", "val_graphs", "val_score_time",
        "train_score", "train_graphs", "train_score_time"]
    for line in stderr_lines:
        if "INFO" not in line:
            continue
        content = get_content(line)
        if any([content.startswith(prefix) for prefix in useful_prefixes]):
            stats.update(process_stats(content))

    return stats


def main():
    with open("training_logs.csv", "w") as csvfile:
        base_fieldnames = ["domain"]
        data_fieldnames = [
            "fitting_time",
            "train_graphs", "val_graphs",
            "total_colours", 
            "val_score", "val_baseline", "val_improvement", "val_score_time",
            "train_score", "train_baseline", "train_improvement", "train_score_time",
        ]
        fieldnames = base_fieldnames + data_fieldnames
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()

        for log_file in os.listdir(LOG_DIR):
            if not os.path.isfile(os.path.join(LOG_DIR, log_file)) or not log_file.endswith(".err"):
                continue

            basename = log_file.replace(".err", "")
            if not basename.startswith("wl-palg-lambdamart"):
                continue
            domain = basename.split("-")[-1]
            row = {"domain": domain}
            
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