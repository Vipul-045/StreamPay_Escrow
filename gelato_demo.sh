#!/bin/bash

# Gelato Automation Demo Script
# This simulates how Gelato keepers would interact with our contract

CONTRACT_ADDRESS="0xbd667dec9a2bbae30609caac894f1ca81379df51"
PRIVATE_KEY="0x9897ba4c9e055bcf476562ccbcd819abf16a1b3a1f7225ed23651a48f0fe2f1b"
SUBSCRIBER="0xd27e6Bf5bF3AdbBE1d85c1c0537F1Dd5302A7E76"
RPC_URL="https://sepolia-rollup.arbitrum.io/rpc"

echo "=== GELATO AUTOMATION DEMO ==="
echo "Contract: $CONTRACT_ADDRESS"
echo "Subscriber: $SUBSCRIBER"
echo ""

# Function to check if payments are due
check_due_payments() {
    echo "🔍 Checking if payments are due..."
    RESULT=$(cast call $CONTRACT_ADDRESS "hasDuePayments()(bool)" --rpc-url $RPC_URL)
    echo "Result: $RESULT"
    if [ "$RESULT" = "true" ]; then
        echo "💰 Payment is due!"
        return 0
    else
        echo "⏰ No payment due yet"
        return 1
    fi
}

# Function to process auto payment
process_auto_payment() {
    echo "🤖 Processing automatic payment..."
    cast send $CONTRACT_ADDRESS "processPaymentAuto(address)" $SUBSCRIBER \
        --private-key $PRIVATE_KEY --rpc-url $RPC_URL
    echo "✅ Auto payment processed!"
}

# Function to check total payments
check_total_payments() {
    echo "💳 Checking total payments..."
    TOTAL=$(cast call $CONTRACT_ADDRESS "totalPayments()(uint256)" --rpc-url $RPC_URL)
    echo "Total payments: $TOTAL wei ($(echo "scale=6; $TOTAL / 1000000000000000000" | bc) ETH)"
}

# Function to check subscription details
check_subscription() {
    echo "📋 Checking subscription details..."
    DETAILS=$(cast call $CONTRACT_ADDRESS "getSubscription(address)(uint256,uint256,uint256,bool)" $SUBSCRIBER --rpc-url $RPC_URL)
    echo "Subscription details: $DETAILS"
}

# Main automation loop (simulating Gelato keeper)
echo "🚀 Starting Gelato automation simulation..."
echo ""

for i in {1..3}; do
    echo "--- Round $i ---"
    
    check_due_payments
    if [ $? -eq 0 ]; then
        process_auto_payment
        echo ""
        check_total_payments
        echo ""
        check_subscription
    fi
    
    echo ""
    echo "⏳ Waiting for next check..."
    sleep 5
done

echo "🏁 Demo completed!"
