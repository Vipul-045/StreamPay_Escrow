# Hybrid Escrow Contract - Testing Guide

## Contract Overview

This is a simplified hybrid escrow recurring payments contract without Gelato automation. Users can:

1. **Deposit ETH to escrow** for future subscription payments
2. **Create subscriptions** with custom amounts and block intervals
3. **Process payments** manually (either from escrow or by sending ETH)
4. **Withdraw remaining escrow** balance
5. **Cancel subscriptions**

## Key Features Removed from Gelato Version

- ❌ Gelato automation functions (`checker`, `process_payment_auto`, etc.)
- ❌ Gelato task management
- ❌ Gelato automate address constants
- ✅ Simple manual payment processing
- ✅ Escrow balance management
- ✅ Subscription lifecycle management

## Contract Functions

### Core Functions

1. **`initialize(provider_address)`** - Initialize contract with provider who receives payments
2. **`deposit_to_escrow()`** - Deposit ETH to your escrow balance (payable)
3. **`create_subscription(amount, interval_blocks)`** - Create a subscription 
4. **`process_payment(subscriber)`** - Process payment by sending ETH (payable)
5. **`process_payment_from_escrow(subscriber)`** - Process payment from escrow balance
6. **`cancel_subscription()`** - Cancel your subscription
7. **`withdraw_escrow()`** - Withdraw remaining escrow balance

### View Functions

1. **`get_escrow_balance(user)`** - Check escrow balance
2. **`is_payment_due(subscriber)`** - Check if payment is due
3. **`get_subscription(subscriber)`** - Get subscription details
4. **`get_all_subscribers()`** - Get list of active subscribers
5. **`get_active_subscription_count()`** - Count active subscriptions
6. **`owner()`** - Get contract owner
7. **`provider()`** - Get provider address
8. **`total_payments()`** - Get total payments processed
9. **`current_block_number()`** - Get current block number

## Deployment Commands

```bash
# Build the contract
cargo build

# Check for deployment
cargo stylus check

# Deploy to Arbitrum Sepolia (replace with your private key)
cargo stylus deploy --private-key=0x... --estimate-gas

# Verify deployment
cargo stylus verify --deployment-tx=0x...
```

## Testing Workflow with Cast

### 1. Initialize Contract
```bash
# Initialize with provider address
cast send CONTRACT_ADDRESS "initialize(address)" "PROVIDER_ADDRESS" \
    --private-key=PRIVATE_KEY --rpc-url=ARBITRUM_SEPOLIA_RPC
```

### 2. Deposit to Escrow
```bash
# Deposit 0.1 ETH to escrow
cast send CONTRACT_ADDRESS "depositToEscrow()" \
    --value 0.1ether --private-key=PRIVATE_KEY --rpc-url=ARBITRUM_SEPOLIA_RPC
```

### 3. Create Subscription
```bash
# Create subscription: 0.01 ETH every 10 blocks
cast send CONTRACT_ADDRESS "createSubscription(uint256,uint256)" \
    "10000000000000000" "10" \
    --private-key=PRIVATE_KEY --rpc-url=ARBITRUM_SEPOLIA_RPC
```

### 4. Check Payment Status
```bash
# Check if payment is due
cast call CONTRACT_ADDRESS "isPaymentDue(address)" "USER_ADDRESS" \
    --rpc-url=ARBITRUM_SEPOLIA_RPC

# Get subscription details
cast call CONTRACT_ADDRESS "getSubscription(address)" "USER_ADDRESS" \
    --rpc-url=ARBITRUM_SEPOLIA_RPC
```

### 5. Process Payment from Escrow
```bash
# Process payment from escrow (anyone can call this)
cast send CONTRACT_ADDRESS "processPaymentFromEscrow(address)" "USER_ADDRESS" \
    --private-key=PRIVATE_KEY --rpc-url=ARBITRUM_SEPOLIA_RPC
```

### 6. Check Balances
```bash
# Check escrow balance
cast call CONTRACT_ADDRESS "getEscrowBalance(address)" "USER_ADDRESS" \
    --rpc-url=ARBITRUM_SEPOLIA_RPC

# Check total payments
cast call CONTRACT_ADDRESS "totalPayments()" \
    --rpc-url=ARBITRUM_SEPOLIA_RPC
```

### 7. Cancel and Withdraw
```bash
# Cancel subscription
cast send CONTRACT_ADDRESS "cancelSubscription()" \
    --private-key=PRIVATE_KEY --rpc-url=ARBITRUM_SEPOLIA_RPC

# Withdraw remaining escrow
cast send CONTRACT_ADDRESS "withdrawEscrow()" \
    --private-key=PRIVATE_KEY --rpc-url=ARBITRUM_SEPOLIA_RPC
```

## Example Test Sequence

```bash
# 1. Deploy and initialize
CONTRACT_ADDRESS="0x..."
PROVIDER_ADDRESS="0x..."
USER_ADDRESS="0x..."

# 2. Deposit escrow
cast send $CONTRACT_ADDRESS "depositToEscrow()" --value 0.1ether --private-key=$PRIVATE_KEY

# 3. Create subscription (0.01 ETH every 5 blocks)
cast send $CONTRACT_ADDRESS "createSubscription(uint256,uint256)" "10000000000000000" "5" --private-key=$PRIVATE_KEY

# 4. Wait 5+ blocks, then process payment
cast send $CONTRACT_ADDRESS "processPaymentFromEscrow(address)" $USER_ADDRESS --private-key=$PRIVATE_KEY

# 5. Check balances
cast call $CONTRACT_ADDRESS "getEscrowBalance(address)" $USER_ADDRESS
cast call $CONTRACT_ADDRESS "totalPayments()"
```

## Key Differences from Multi-Tenant Version

This simplified version:
- ✅ Single provider model (simpler)
- ✅ Block-based intervals (no time-based)
- ✅ Manual payment processing (no automation)
- ✅ Direct escrow model (user deposits, contract holds)
- ✅ Simple subscription lifecycle

The previous multi-tenant version had:
- ❌ Multiple providers with different fees
- ❌ Time-based intervals with grace periods
- ❌ Complex fee calculations
- ❌ Hybrid transferFrom + escrow model
- ❌ Advanced subscription states (PastDue, etc.)

This version is perfect for learning and testing the core concepts!
