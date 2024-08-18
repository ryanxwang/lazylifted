#!/bin/bash

if [ $# -ne 2 ]; then
    echo "Usage: $0 <model_type> <model>"
    exit 1
fi

model_type=$1
model=$2

model_str=$(sed 's/-/_/g' <<< $model_type)-$(sed 's/-/_/g' <<< $model)

domains=("blocksworld" "childsnack" "ferry" "floortile" "miconic" "rovers" "satellite" "sokoban" "spanner" "transport" "blocksworld-htg")
csvs=()

mkdir -p planning_results/$model_str
for domain in "${domains[@]}"; do
    log_dir=planning_logs/$model_str/$domain

    if [ ! -d $log_dir ]; then
        echo "Log directory $log_dir does not exist. Skipping..."
        continue
    fi

    result_file=planning_results/$model_str/$domain.csv
    echo "Collecting log for $domain in $log_dir"

    python3 scripts/log_processing/collate_planning_results.py $log_dir $result_file
    csvs+=($result_file)
done

echo "Creating merged csv file"
to_merge=$(IFS=" "; echo "${csvs[*]}")
python3 scripts/log_processing/merge_csv.py --output planning_results/$model_str/merged.csv $to_merge


echo "All results collected and saved to planning_results/$model_str"