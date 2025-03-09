#!/bin/bash
set -e

if [ $# -ne 1 ]; then
    echo "Specify binary name"
    exit 1
fi

#cp bin.rs ../bin/$1.rs
cp bin_data.txt ../bin/$1_data.txt
sed "s/bin_data.txt/$1_data.txt/g" bin.rs > ../bin/$1.rs
