#!/bin/bash
set -e

if [ $# -ne 1 ]; then
    echo "Specify binary name"
    exit 1
fi

script_dir=$(dirname "${BASH_SOURCE[0]}")

#cp bin.rs ../bin/$1.rs
cp $script_dir/bin_data.txt $script_dir/../bin/$1_data.txt
sed "s/bin_data.txt/$1_data.txt/g" $script_dir/bin.rs > $script_dir/../bin/$1.rs
