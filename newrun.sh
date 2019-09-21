#!/bin/bash
./target/release/bool-node purge-chain --dev
./target/release/bool-node --dev --other-execution=Native --syncing-execution=Native --block-construction-execution=Native --importing-execution=Native