#!/bin/bash

for lpc in lpc81x lpc11xx lpc13xx lpc15xx lpc17xx;
do
    pushd algos/$lpc
    cargo build --release
    if [ -f target/thumbv6m-none-eabi/release/$lpc ]; then
        target-gen elf target/thumbv6m-none-eabi/release/$lpc -u template.yaml
        cp template.yaml ../../$lpc.yaml
    elif [ -f target/thumbv7m-none-eabi/release/$lpc ]; then
        target-gen elf target/thumbv7m-none-eabi/release/$lpc -u template.yaml
        cp template.yaml ../../$lpc.yaml
    else
        echo "No output found for target $lpc"
    fi
    popd
done