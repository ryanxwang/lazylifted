#!/bin/bash

if [ $# -ne 4 ]; then
    echo "Usage: $0 <model_type> <heuristic> <domain> <instance>"
    exit 1
fi

model_type=$1
heuristic=$2
domain=$3
instance=$4

model_str=$(sed 's/-/_/g' <<< $model_type)-$(sed 's/-/_/g' <<< $heuristic)
log_dir=planning_logs/$model_str/$domain
plan_dir=plans/$model_str/$domain


echo "This script will overwrite previous logs for the same experiment at $log_dir"

mkdir -p $log_dir
mkdir -p $plan_dir

source scripts/setup_dynamic_library.sh

planner_bin=""
if [[ $PLANNER_BIN ]]; then
    planner_bin=$PLANNER_BIN
else 
    planner_bin="target/release/planner"
fi

instance_str=$(sed 's/\//_/g' <<< $instance)
plan_file=$plan_dir/$instance_str.plan

if [ "$model_type" == "partial-space" ]; then
    subcommand="partial-action-search"
elif [ "$model_type" == "state-space" ]; then
    subcommand="state-space-search"
else
    echo "Unsupported model type"
    exit 1
fi

cmd="./$planner_bin benchmarks/ipc23-learning/$domain/domain.pddl benchmarks/ipc23-learning/$domain/$instance.pddl -o $plan_file $subcommand --heuristic $heuristic"
err_log=$log_dir/$instance_str.err
out_log=$log_dir/$instance_str.out
echo "Planning for domain $domain with command: $cmd, saving logs to $err_log and $out_log and plan to $plan_file"
$cmd 2> $err_log 1> $out_log
