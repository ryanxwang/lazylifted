#!/bin/bash

if [ $# -ne 2 ]; then
    echo "Usage: $0 <domain> <instance>"
    exit 1
fi

planner_type=instrumented
log_dir=planning_logs

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

source scripts/setup_dynamic_library.sh
cargo build --release --bins

cmd="./target/release/planner -s instrumented-gbfs -e wl-ilg -m trained_models/wl-ilg-gpr-$domain -p trained_models/wl-palg-lambdamart-$domain benchmarks/ipc23-learning/$domain/domain.pddl benchmarks/ipc23-learning/$domain/$instance.pddl"
instance_str=$(sed 's/\//_/g' <<< $instance)
err_log=$log_dir/$planner_type-$domain-$instance_str.err
out_log=$log_dir/$planner_type-$domain-$instance_str.out
echo "Training model for domain $domain with command: $cmd, saving logs to $err_log and $out_log"
$cmd 2> $err_log 1> $out_log
