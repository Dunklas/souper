#!/bin/sh

cargo clean
RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="souper-%m.profraw" cargo test --tests
llvm-profdata merge -sparse souper-*.profraw -o souper.profdata

BINARY=$(ls target/debug/deps/ | grep souper | grep -v ".d")
llvm-cov report --use-color --ignore-filename-regex='/.cargo/registry' --instr-profile souper.profdata --object target/debug/deps/$BINARY

