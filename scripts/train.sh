#!/bin/bash
if [ $# -ne 3 ]; then
    echo "Usage: $0 <model-type> <model> <domain>, e.g. $0 wl-ilg-gpr partial-space blocksworld"
    exit 1
fi

model_type=$1
model=$2
domain=$3

model_dir=experiments/models
data_dir=experiments/domains
log_dir=training_logs

if [[ $TRAINER_BIN ]]; then
    trainer_bin=$TRAINER_BIN
else
    trainer_bin="target/release/trainer"
fi

echo "This script will overwrite previous logs for the same training targets $log_dir and the previous trained models at trained_models/..."

mkdir -p $log_dir

source scripts/setup_dynamic_library.sh

cmd="./$trainer_bin --data $data_dir/$domain.toml --model $model_dir/$model_type/$model.toml --save trained_models/$model_type-$model-$domain.model --verbose"
err_log=$log_dir/$model_type-$model-$domain.err
out_log=$log_dir/$model_type-$model-$domain.out
echo "Training model for domain $domain with command: $cmd, saving logs to $err_log and $out_log"
$cmd 2> $err_log 1> $out_log
