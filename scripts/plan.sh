#!/bin/bash

if [ $# -ne 2 ]; then
    echo "Usage: $0 <domain> <instance>"
    exit 1
fi

planner_type=wl_palg
log_dir=planning_logs
plan_dir=plans

domain=$1
instance=$2

# https://stackoverflow.com/questions/1885525/how-do-i-prompt-a-user-for-confirmation-in-bash-script
echo "This script will overwrite previous logs for the same experiment at $log_dir"
read -p "Are you sure? (y/n) " -n 1 -r
echo    # move to a new line
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    [[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1 # handle exits from shell or function but don't exit interactive shell
fi

mkdir -p $log_dir
mkdir -p $plan_dir/$domain

source scripts/setup_dynamic_library.sh

bin_location=""
if [[ $(uname) == "Darwin" ]]; then
    bin_location="./target/release/planner"
elif [[ $(uname) == "Linux" ]]; then
    bin_location="./planner"
else
    echo "Unsupported operating system"
    exit 1
fi

instance_str=$(sed 's/\//_/g' <<< $instance)
plan_file=$plan_dir/$domain/$instance_str.plan
cmd="$bin_location benchmarks/ipc23-learning/$domain/domain.pddl benchmarks/ipc23-learning/$domain/$instance.pddl -o $plan_file partial-action-search --heuristic wl-palg --model trained_models/wl-palg-lambdamart-$domain "
err_log=$log_dir/$planner_type-$domain-$instance_str.err
out_log=$log_dir/$planner_type-$domain-$instance_str.out
echo "Planning for domain $domain with command: $cmd, saving logs to $err_log and $out_log and plan to $plan_file"
$cmd 2> $err_log 1> $out_log
