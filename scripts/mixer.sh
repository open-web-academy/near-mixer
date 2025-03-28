#!/bin/bash

set -e

NEAR_ACCOUNT=""
CONTRACT_ID=""
NETWORK="testnet"

# Helper function for generating hashes and proofs
generate_commitment() {
    local secret="$RANDOM-$RANDOM-$RANDOM"
    echo "$secret" > .mixer_secret.txt
    echo "Secret saved to .mixer_secret.txt"
    
    # Simple hash generation
    echo -n "$secret" | sha256sum | awk '{print $1}'
}

generate_nullifier() {
    if [ ! -f .mixer_secret.txt ]; then
        echo "Error: No secret found. Please deposit first."
        exit 1
    fi
    
    local secret=$(cat .mixer_secret.txt)
    echo -n "withdraw-$secret" | sha256sum | awk '{print $1}'
}

generate_proof() {
    if [ ! -f .mixer_secret.txt ]; then
        echo "Error: No secret found. Please deposit first."
        exit 1
    fi
    
    local secret=$(cat .mixer_secret.txt)
    local commitment=$(echo -n "$secret" | sha256sum | awk '{print $1}')
    local nullifier=$(echo -n "withdraw-$secret" | sha256sum | awk '{print $1}')
    
    # Simple proof generation (in a real implementation this would be cryptographic)
    local proof_base=$(echo -n "$nullifier$commitment" | sha256sum | awk '{print $1}')
    echo "${proof_base:0:1}${proof_base}"
}

print_help() {
    echo "NEAR Mixer CLI"
    echo ""
    echo "Usage:"
    echo "  $0 init <your-account> <contract-id> [network]  - Initialize CLI"
    echo "  $0 deploy                                       - Deploy the contract"
    echo "  $0 deposit <amount>                             - Deposit NEAR to the mixer"
    echo "  $0 withdraw <recipient>                         - Withdraw NEAR to an account"
    echo "  $0 stats                                        - Show mixer pool statistics"
    echo ""
    echo "Examples:"
    echo "  $0 init alice.testnet mixer.alice.testnet testnet"
    echo "  $0 deploy"
    echo "  $0 deposit 1"
    echo "  $0 withdraw bob.testnet"
}

init() {
    if [ -z "$1" ] || [ -z "$2" ]; then
        echo "Error: Missing parameters. Usage: $0 init <your-account> <contract-id> [network]"
        exit 1
    fi
    
    NEAR_ACCOUNT="$1"
    CONTRACT_ID="$2"
    
    if [ ! -z "$3" ]; then
        NETWORK="$3"
    fi
    
    echo "NEAR_ACCOUNT=$NEAR_ACCOUNT" > .mixer_config
    echo "CONTRACT_ID=$CONTRACT_ID" >> .mixer_config
    echo "NETWORK=$NETWORK" >> .mixer_config
    
    echo "Mixer CLI initialized:"
    echo "  Account: $NEAR_ACCOUNT"
    echo "  Contract: $CONTRACT_ID"
    echo "  Network: $NETWORK"
}

load_config() {
    if [ ! -f .mixer_config ]; then
        echo "Error: CLI not initialized. Run '$0 init <your-account> <contract-id> [network]' first."
        exit 1
    fi
    
    source .mixer_config
}

deploy_contract() {
    load_config
    
    echo "Building contract..."
    cargo build --target wasm32-unknown-unknown --release
    
    echo "Deploying contract to $CONTRACT_ID on $NETWORK..."
    near deploy --accountId $NEAR_ACCOUNT --wasmFile target/wasm32-unknown-unknown/release/near_mixer.wasm --networkId $NETWORK
    
    echo "Initializing contract..."
    near call $CONTRACT_ID new "{\"owner\": \"$NEAR_ACCOUNT\", \"fee_basis_points\": 100}" --accountId $NEAR_ACCOUNT --networkId $NETWORK
    
    echo "Contract deployed and initialized!"
}

deposit() {
    load_config
    
    if [ -z "$1" ]; then
        echo "Error: Missing amount parameter. Usage: $0 deposit <amount>"
        exit 1
    fi
    
    local amount="$1"
    # Convert to yoctoNEAR
    local yocto_amount=$(echo "$amount * 10^24" | bc)
    
    echo "Generating commitment for your deposit..."
    local commitment=$(generate_commitment)
    
    echo "Depositing $amount NEAR to the mixer..."
    near call $CONTRACT_ID deposit '{"commitment": "'"$commitment"'"}' \
        --accountId $NEAR_ACCOUNT --networkId $NETWORK --amount $amount --gas 300000000000000
    
    echo "Deposit complete! Your commitment: $commitment"
    echo ""
    echo "IMPORTANT: Keep your secret secure. You'll need it to withdraw funds."
    echo "Wait at least 24 hours before withdrawing for better privacy."
}

withdraw() {
    load_config
    
    if [ -z "$1" ]; then
        echo "Error: Missing recipient parameter. Usage: $0 withdraw <recipient>"
        exit 1
    fi
    
    local recipient="$1"
    
    echo "Generating withdrawal data..."
    local commitment=$(echo -n "$(cat .mixer_secret.txt)" | sha256sum | awk '{print $1}')
    local nullifier=$(generate_nullifier)
    local proof=$(generate_proof)
    
    echo "Withdrawing funds to $recipient..."
    near call $CONTRACT_ID withdraw "{\"recipient\": \"$recipient\", \"nullifier\": \"$nullifier\", \"commitment\": \"$commitment\", \"proof\": \"$proof\"}" \
        --accountId $NEAR_ACCOUNT --networkId $NETWORK --gas 300000000000000
    
    echo "Withdrawal complete!"
    rm -f .mixer_secret.txt
}

show_stats() {
    load_config
    
    echo "Fetching mixer pool statistics..."
    near view $CONTRACT_ID get_pool_stats '{}' --networkId $NETWORK
}

# Main script execution
case "$1" in
    init)
        init "$2" "$3" "$4"
        ;;
    deploy)
        deploy_contract
        ;;
    deposit)
        deposit "$2"
        ;;
    withdraw)
        withdraw "$2"
        ;;
    stats)
        show_stats
        ;;
    *)
        print_help
        ;;
esac