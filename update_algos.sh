#!/usr/bin/env bash

for lpc in lpc8xx lpc11xx lpc13xx lpc15xx lpc177x_8x;
do
    pushd "lpc-probers-target"
    cargo run -- ../docs/${lpc}_targets.json
    cp ${lpc}_generated.yaml ../algos/${lpc}/template.yaml
    popd 

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
