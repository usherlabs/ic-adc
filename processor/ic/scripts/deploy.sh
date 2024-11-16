#!/bin/bash

export CANISTER_PRINCIPAL="bw4dl-smaaa-aaaaa-qaacq-cai"
dfx deploy adc --argument "(opt principal \"$CANISTER_PRINCIPAL\")"

