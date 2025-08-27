# Hybrid Escrow Recurring Payments Contract

A production-ready smart contract for **automated recurring payments** built with **Arbitrum Stylus** (Rust). This contract implements a **hybrid escrow + pull-allowance model** optimized for gas efficiency and user experience.

## 🏗️ Architecture Overview

### Hybrid Payment Model
1. **Initial Escrow**: Users deposit funds upfront (1+ intervals)
2. **Automated Releases**: Contract releases payments when due
3. **Fallback Mechanism**: Auto-cancellation with grace period if insufficient funds
4. **Provider Batching**: Providers can batch-withdraw accumulated payments

### Key Features
- ✅ **Gas Optimized**: Stylus Rust contract (~10x cheaper than Solidity)
- ✅ **Automation Friendly**: Anyone can trigger releases (decentralized keepers)
- ✅ **Flexible Durations**: Support for both fixed-term and indefinite subscriptions
- ✅ **Emergency Controls**: Admin pause/emergency functions for security
- ✅ **Comprehensive Events**: Full transparency for frontend integration
- ✅ **Re-entrancy Protection**: Built-in security guards

## 🚀 Quick Start

### Prerequisites
```bash
# Install Rust and Stylus toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install cargo-stylus

# Install Foundry for contract interaction
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Set up environment
export PRIVATE_KEY="your_private_key_here"
export PROVIDER_ADDRESS="0x742d35Cc6634C0532925a3b8D096A5A5b6f8E637"
```

### Deploy Contract
```bash
# 1. Build the contract
cargo build --release

# 2. Deploy to Arbitrum Sepolia
node scripts/deploy.js

# 3. Contract address saved to deployment.json
```

### Create a Subscription
```bash
# Create subscription with 0.005 ETH every 5 minutes for 12 intervals
AMOUNT_PER_INTERVAL="5000000000000000" \
INTERVAL_SECONDS="300" \
DURATION_INTERVALS="12" \
DEPOSIT_AMOUNT="15000000000000000" \
node scripts/create-subscription.js
```

### Start Automated Payment Releases
```bash
# Monitor and auto-release payments (runs continuously)
CHECK_INTERVAL="30" \
MAX_ITERATIONS="100" \
node scripts/release-loop.js
```

### Manage Subscriptions
```bash
# Top up a subscription
node scripts/topup.js <subscription_id> [amount_in_wei]

# Cancel a subscription (refunds remaining balance)
node scripts/cancel.js <subscription_id>
```

## 📋 Contract Interface

### Core Functions

#### `createSubscription(provider, amountPerInterval, intervalSeconds, durationIntervals)`
Creates a new subscription with ETH deposit.
- **Parameters**:
  - `provider`: Address of service provider
  - `amountPerInterval`: Payment amount in wei per interval
  - `intervalSeconds`: Time between payments (minimum 60 seconds)
  - `durationIntervals`: Total intervals (0 = indefinite)
- **Payable**: Must send initial deposit (≥ 1 interval amount)
- **Returns**: Subscription ID

#### `releasePayment(subscriptionId)`
Releases payment for a due subscription (callable by anyone).
- **Gas Optimized**: Batch operations for providers
- **Auto-management**: Handles insufficient funds and expiration

#### `topUp(subscriptionId)`
Add funds to extend subscription duration.
- **Reactivation**: Automatically reactivates past-due subscriptions
- **Flexible**: Any amount accepted

#### `cancelSubscription(subscriptionId)`
Cancel subscription and refund remaining balance.
- **Authorization**: Callable by user or provider
- **Instant Refund**: Remaining balance returned to user

### View Functions

#### `getSubscription(subscriptionId)`
Returns complete subscription details.
- **Returns**: `(user, provider, amountPerInterval, intervalSeconds, durationIntervals, balance, paidIntervals, nextPaymentTs, status, createdAt)`

#### `isPaymentDue(subscriptionId)`
Check if payment is ready for release.

#### `getProviderBalance(provider)`
Get accumulated payments for provider withdrawal.

#### `getUserSubscriptions(user)` / `getProviderSubscriptions(provider)`
Get all subscription IDs for an address.

## 🔧 Configuration

### Environment Variables
```bash
# Required
PRIVATE_KEY="0x..."                    # Deployer/user private key
PROVIDER_ADDRESS="0x..."               # Service provider address

# Subscription Parameters
AMOUNT_PER_INTERVAL="5000000000000000"  # 0.005 ETH in wei
INTERVAL_SECONDS="300"                  # 5 minutes for testing
DURATION_INTERVALS="12"                 # 12 intervals total
DEPOSIT_AMOUNT="15000000000000000"      # 0.015 ETH (3 intervals)

# Automation Parameters
CHECK_INTERVAL="30"                     # Check every 30 seconds
MAX_ITERATIONS="100"                    # Limit for demo loops
TARGET_SUB_ID="1"                      # Monitor specific subscription
```

### Network Configuration
```javascript
// Arbitrum Sepolia (Testnet)
const CONFIG = {
    rpcUrl: 'https://sepolia-rollup.arbitrum.io/rpc',
    chainId: 421614,
    explorerUrl: 'https://sepolia.arbiscan.io',
};
```

## 📊 Testing & Demo

### Unit Tests
```bash
# Run Rust unit tests
cargo test

# Test specific module
cargo test tests::subscription_creation_validation
```

### Integration Demo
```bash
# Full end-to-end demo
npm run demo

# Or step by step:
npm run deploy          # Deploy contract
npm run create-sub      # Create subscription
npm run release-loop    # Start automation
```

### Manual Testing Scenarios

1. **Basic Subscription Flow**:
   ```bash
   # Deploy and create subscription
   node scripts/deploy.js
   node scripts/create-subscription.js
   
   # Wait 5+ minutes, then release payment
   node scripts/release-loop.js
   ```

2. **Top-up Scenario**:
   ```bash
   # Create subscription with minimal deposit
   DEPOSIT_AMOUNT="5000000000000000" node scripts/create-subscription.js
   
   # Top up before it runs out
   node scripts/topup.js 1 10000000000000000
   ```

3. **Cancellation Testing**:
   ```bash
   # Create and immediately cancel
   node scripts/create-subscription.js
   node scripts/cancel.js 1
   ```

## 🔒 Security Features

### Access Controls
- **Owner-only functions**: Emergency controls, pausing
- **Subscription ownership**: Only users/providers can cancel their subscriptions
- **Authorization checks**: Comprehensive permission validation

### Re-entrancy Protection
```rust
fn nonreentrant_start(&mut self) -> Result<(), HybridEscrowError> {
    if self.locked.get() {
        return Err(HybridEscrowError::ReentrancyDetected("Reentrant call".to_string()));
    }
    self.locked.set(true);
    Ok(())
}
```

### Emergency Mechanisms
- **Pause functionality**: Admin can pause all operations
- **Emergency withdrawal**: Owner can withdraw funds in critical situations
- **Grace periods**: Users get time to top up before auto-cancellation

### Input Validation
- Minimum interval duration (60 seconds)
- Non-zero amounts and valid addresses
- Sufficient deposits for subscription creation
- Status checks before state changes

## 📈 Gas Optimization

### Stylus Efficiency
- **~10x cheaper** than equivalent Solidity contracts
- **Native Rust performance** with EVM compatibility
- **Optimized storage patterns** for reduced gas costs

### Batching Strategies
```rust
// Providers accumulate funds for batch withdrawal
mapping(address => uint256) provider_balances;

pub fn withdraw_provider_funds(&mut self) -> Result<(), HybridEscrowError> {
    let amount = self.provider_balances.get(provider);
    self.provider_balances.setter(provider).set(U256::ZERO);
    // Single withdrawal for all accumulated payments
}
```

## 🚦 Status Management

### Subscription States
```rust
pub enum SubscriptionStatus {
    Active = 0,           // Normal operation
    PastDue = 1,          // Payment failed, grace period
    CancelledByUser = 2,  // User cancelled
    CancelledByProvider = 3, // Provider cancelled
    Expired = 4,          // Reached duration limit
}
```

### State Transitions
- `Active` → `PastDue`: Insufficient balance
- `PastDue` → `Active`: Top-up received
- `PastDue` → `CancelledByProvider`: Grace period expired
- `Active/PastDue` → `Expired`: Duration limit reached
- `Any` → `CancelledByUser/Provider`: Manual cancellation

## 📝 Event Logging

### Complete Transparency
```solidity
event SubscriptionCreated(uint256 indexed subscription_id, address indexed user, address indexed provider, uint256 amount_per_interval, uint64 interval_seconds, uint64 duration_intervals);

event PaymentReleased(uint256 indexed subscription_id, uint256 paid_amount, address paid_to_provider, uint256 remaining_balance);

event TopUp(uint256 indexed subscription_id, uint256 amount, uint256 new_balance);

event SubscriptionCancelled(uint256 indexed subscription_id, address by, uint256 refunded_amount, uint8 reason);
```

## 🔄 Automation Integration

### Keeper Compatibility
The contract is designed for decentralized automation:

```javascript
// Example keeper logic
async function keeperLoop() {
    const subscriptions = await getActiveSubscriptions();
    
    for (const subId of subscriptions) {
        if (await contract.isPaymentDue(subId)) {
            await contract.releasePayment(subId);
        }
    }
}
```

### Chainlink Automation
```solidity
// Compatible with Chainlink Keepers
function checkUpkeep(bytes calldata) external view returns (bool upkeepNeeded, bytes memory performData) {
    // Check if any subscriptions need payment release
    for (uint256 i = 1; i <= subscriptionCount; i++) {
        if (isPaymentDue(i)) {
            return (true, abi.encode(i));
        }
    }
    return (false, "");
}

function performUpkeep(bytes calldata performData) external {
    uint256 subscriptionId = abi.decode(performData, (uint256));
    releasePayment(subscriptionId);
}
```

## 📁 File Structure

```
hybrid-escrow-contract/
├── src/
│   ├── lib.rs              # Main contract implementation
│   ├── main.rs             # Contract entry point
│   └── utils.rs            # Helper functions
├── tests/
│   └── contract_tests.rs   # Unit tests
├── scripts/
│   ├── deploy.js           # Deployment script
│   ├── create-subscription.js # Create subscription demo
│   ├── release-loop.js     # Automated payment releases
│   ├── topup.js           # Top-up subscription
│   └── cancel.js          # Cancel subscription
├── Cargo.toml             # Rust dependencies
├── package.json           # Node.js scripts
└── README.md              # This file
```

## 🤝 Contributing

### Development Setup
```bash
# Clone repository
git clone <repo-url>
cd hybrid-escrow-contract

# Install dependencies
rustup target add wasm32-unknown-unknown
cargo install cargo-stylus

# Build and test
cargo build --release
cargo test
```

### Code Standards
- **Rust best practices**: Use `clippy` and `rustfmt`
- **Security first**: All functions have proper access controls
- **Gas optimization**: Minimize storage operations
- **Comprehensive testing**: Unit tests for all logic paths

## 📜 License

MIT License - see LICENSE file for details.

## 🆘 Support

### Common Issues

1. **"Transaction failed"**: Check gas limits and network connectivity
2. **"Insufficient funds"**: Ensure adequate ETH balance for deposits
3. **"Subscription not found"**: Verify subscription ID exists
4. **"Not authorized"**: Check caller is user or provider

### Debug Mode
```bash
# Enable detailed logging
DEBUG=true node scripts/release-loop.js

# Check transaction details
cast tx <transaction_hash> --rpc-url https://sepolia-rollup.arbitrum.io/rpc
```

### Getting Help
- 📧 Email: support@mvp-team.com
- 💬 Discord: [MVP Community](https://discord.gg/mvp)
- 🐛 Issues: [GitHub Issues](https://github.com/mvp-team/hybrid-escrow-contract/issues)

---

**Built with ❤️ using Arbitrum Stylus & Rust**
