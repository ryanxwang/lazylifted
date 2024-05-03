#!/bin/bash
if [ $# -ne 3 ]; then
    echo "Usage: $0 <model-type> <model> <domain>, e.g. $0 wl-ilg-gpr partial-space blocksworld"
    exit 1
fi

model_type=$1
model=$2
domain=$3

model_dir=experiments/models
data_dir=experiments/ipc23-learning
log_dir=training_logs

if [[ $TRAINER_BIN ]]; then
    trainer_bin=$TRAINER_BIN
else
    trainer_bin="target/release/trainer"
fi

# https://stackoverflow.com/questions/1885525/how-do-i-prompt-a-user-for-confirmation-in-bash-script
echo "This script will overwrite previous logs for the same training targets $log_dir and the previous trained models at trained_models/..."
read -p "Are you sure? (y/n) " -n 1 -r
echo    # move to a new line
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    [[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1 # handle exits from shell or function but don't exit interactive shell
fi

mkdir -p $log_dir

cmd="./$trainer_bin --data $data_dir/$domain.toml --model $model_dir/$model_type/$model.toml --save trained_models/$model_type-$model-$domain"
err_log=$log_dir/$model_type-$model-$domain.err
out_log=$log_dir/$model_type-$model-$domain.out
echo "Training model for domain $domain with command: $cmd, saving logs to $err_log and $out_log"
$cmd 2> $err_log 1> $out_log
