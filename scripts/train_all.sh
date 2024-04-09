#!/bin/bash
model_dir=experiments/models
data_dir=experiments/ipc23-learning
log_dir=training_logs

model=wl-aslg-lambdamart
domains=("blocksworld" "floortile" "miconic" "rovers" "sokoban" "spanner" "transport")

# https://stackoverflow.com/questions/1885525/how-do-i-prompt-a-user-for-confirmation-in-bash-script
echo "This script will overwrite some files (i.e. previous logs) at $log_dir and the previous trained models at trained_models/..."
read -p "Are you sure? (y/n) " -n 1 -r
echo    # move to a new line
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    [[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1 # handle exits from shell or function but don't exit interactive shell
fi

mkdir -p $log_dir

source scripts/setup_dynamic_library.sh
cargo build --release --bins

for domain in ${domains[@]}; do
    cmd="./target/release/trainer --data $data_dir/$domain.toml --model $model_dir/$model.toml --save trained_models/$model-$domain"
    err_log=$log_dir/$model-$domain.err
    out_log=$log_dir/$model-$domain.out
    echo "Training model for domain $domain with command: $cmd, saving logs to $err_log and $out_log"
    $cmd 2> $err_log 1> $out_log
done
