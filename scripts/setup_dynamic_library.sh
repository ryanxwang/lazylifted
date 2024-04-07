#!/bin/bash

python_path=$(which python3)
library_path=$(dirname $python_path)/../lib
library_path=$(realpath $library_path)

echo "Adding path to dynamic library path:"
echo "    $library_path"

# Detect the operating system
if [[ $(uname) == "Darwin" ]]; then
    # Code for macOS
    echo "Running on macOS, adding path to DYLD_FALLBACK_LIBRARY_PATH"
    export DYLD_FALLBACK_LIBRARY_PATH=$library_path:$DYLD_FALLBACK_LIBRARY_PATH
elif [[ $(uname) == "Linux" ]]; then
    # Code for Linux
    echo "Running on Linux, adding path to LD_LIBRARY_PATH"
    export LD_LIBRARY_PATH=$library_path:$LD_LIBRARY_PATH
else
    echo "Unsupported operating system"
    exit 1
fi

echo "Dynamic library path set up successfully"