#!/bin/bash

if [ $# -eq 0 ]; then
    echo "Usage: $0 <tag>"
    exit 1
fi

tag=$1

if [ -d /tmp/$tag ]; then
    rm -r /tmp/$tag
fi

echo "Downloading release $tag to /tmp/$tag"
gh release download $tag -p "*" --clobber -D /tmp/$tag

gadi_dir="/home/444/rw8037/lazylifted"

echo "Removing existing $gadi_dir directory (if exists) on Gadi"
ssh rw8037@gadi.nci.org.au "if [ -d $gadi_dir/$tag ]; then rm -r $gadi_dir/$tag; fi"

echo "Uploading release $tag to Gadi at $gadi_dir"
ssh rw8037@gadi.nci.org.au "mkdir -p $gadi_dir"
scp -r /tmp/$tag rw8037@gadi.nci.org.au:$gadi_dir
ssh rw8037@gadi.nci.org.au "chmod +x $gadi_dir/$tag/*"