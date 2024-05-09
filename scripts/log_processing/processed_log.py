from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Dict, List


@dataclass
class LogValueEntry:
    value: Any
    timestamp: float


@dataclass
class LogTextEntry:
    text: str
    timestamp: float


@dataclass
class ProcessedLog:
    domain: str
    instance: str
    model_type: str
    model: str
    values: Dict[str, List[LogValueEntry]]
    texts: List[LogTextEntry]


def extract_values(content: str) -> Dict[str, Any]:
    pairs = content.split(" ")
    values = {}
    for pair in pairs:
        key, value = pair.split("=", 1)

        try:
            value = int(value)
        except ValueError:
            try:
                value = float(value)
            except ValueError:
                pass

        values[key] = value
    return values


def process_log(path: str) -> ProcessedLog:
    # get various attributes from the path
    domain = path.split("/")[-2]
    model_type = path.split("/")[-3].split("-")[0]
    model = path.split("/")[-3].split("-")[1]
    instance = path.split("/")[-1].split(".")[0]

    # read the log file
    with open(path, "r") as f:
        lines = f.readlines()

    # parse the log file
    values = {}
    texts = []
    start_time = None
    for line in lines:
        header, content = line.split(": ", 1)

        # header is in the format "timestamp  LEVEL source_location"
        timestamp, level, source = header.split(" ", 2)

        # process timestamp
        timestamp = datetime.fromisoformat(timestamp[:-1])  # remove the 'Z' at the end
        if start_time is None:
            start_time = timestamp
        timestamp = (timestamp - start_time).total_seconds()

        try:
            values_dict = extract_values(content)
            for key, value in values_dict.items():
                if key not in values:
                    values[key] = []
                values[key].append(LogValueEntry(value=value, timestamp=timestamp))
        except ValueError:
            texts.append(LogTextEntry(text=content, timestamp=timestamp))

    return ProcessedLog(
        domain=domain,
        instance=instance,
        model_type=model_type,
        model=model,
        values=values,
        texts=texts,
    )

if __name__ == "__main__":
    processed_log = process_log("planning_logs/partial_space-wl_sclg_gpr/blocksworld/testing_hard_p19.err")
    print(processed_log.values["expanded_nodes"])