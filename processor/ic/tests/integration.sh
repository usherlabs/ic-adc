#!/bin/bash
# This requires DFX[https://internetcomputer.org/docs/current/developer-docs/getting-started/install/] to be installed on the system

set -x
set -e

# validate that thte dfx version os 0.22.0
export current_dfx_version=$(dfx --version | cut -d' ' -f2)
if [ "$current_dfx_version" != "0.22.0" ]; then
    echo "Error: Expected DFX version 0.22.0 but got $current_dfx_version"
    exit 1
fi

# export global variables
export CANISTER_NAME="adc"
export REQUESTED_CURRENCY_PAIRS="ETH"

# restart the local chain
dfx stop
nohup dfx start --clean &

# wait some arbitrary time for the dfx local chain to start
sleep 5

dfx identity use default
export CALLER_PRINCIPAL=$(dfx identity get-principal)

# deploy the canister
dfx deploy $CANISTER_NAME

# whitelist the caller
dfx canister call $CANISTER_NAME add_to_whitelist '(principal '\"$CALLER_PRINCIPAL\"')'

# call the function to request data and get a timestamp back
export TIMESTAMP=$(dfx canister call $CANISTER_NAME request_data '('\"$REQUESTED_CURRENCY_PAIRS\"', record {price = true})')

# get the last line of the logs and make sure it contains an ID
export VALID_LOG=$(dfx canister logs $CANISTER_NAME | tail -n 1 | grep "id")

if [ -z "$VALID_LOG" ] || [ "$VALID_LOG" = "" ]; then
    echo "Test Failed: Price Request not logged"
    exit 1
fi

if [ -z "$TIMESTAMP" ] || [ "$TIMESTAMP" = "" ]; then
    echo "Test Failed: Timestamp not returned as id deom data request"
    exit 1
fi