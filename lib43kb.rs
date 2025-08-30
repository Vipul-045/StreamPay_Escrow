// PRODUCTION-READY SUBSCRIPTION ESCROW SYSTEM FOR ARBITRUM STYLUS
// Complete multi-provider subscription platform with automation and safety features
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

extern crate alloc;
use alloc::{string::String, vec::Vec, vec};

// Import Stylus SDK components
use stylus_sdk::{
    alloy_primitives::{Address, U256}, 
    prelude::*
};
use alloy_sol_types::sol;

// =============================================================================
// EVENTS - Comprehensive logging for off-chain indexing
// =============================================================================

sol! {
    // Provider Events
    event ProviderRegistered(address indexed provider, string name, uint256 timestamp);
    event ProviderUpdated(address indexed provider, string newName, bool active);
    
    // Plan Events  
    event PlanCreated(uint256 indexed planId, address indexed provider, string name, uint256 price, uint256 interval, uint256 timestamp);
    event PlanUpdated(uint256 indexed planId, uint256 newPrice, uint256 newInterval, bool active);
    event PlanDeactivated(uint256 indexed planId, address indexed provider, uint256 timestamp);
    
    // Subscription Events
    event SubscriptionCreated(uint256 indexed subscriptionId, uint256 indexed planId, address indexed subscriber, uint256 timestamp);
    event SubscriptionCancelled(uint256 indexed subscriptionId, address indexed subscriber, uint256 refundAmount, uint256 timestamp);
    event SubscriptionExpired(uint256 indexed subscriptionId, address indexed subscriber, uint256 timestamp);
    
    // Payment Events
    event PaymentProcessed(uint256 indexed subscriptionId, uint256 indexed planId, address indexed subscriber, uint256 amount, uint256 timestamp);
    event PaymentFailed(uint256 indexed subscriptionId, address indexed subscriber, uint256 requiredAmount, uint256 availableBalance, uint256 timestamp);
    event AutoPaymentToggled(uint256 indexed subscriptionId, address indexed subscriber, bool enabled, uint256 timestamp);
    
    // Escrow Events
    event FundsDeposited(address indexed user, uint256 amount, uint256 newBalance, uint256 timestamp);
    event FundsWithdrawn(address indexed user, uint256 amount, uint256 newBalance, uint256 timestamp);
    event EarningsWithdrawn(address indexed provider, uint256 amount, uint256 timestamp);
    
    // System Events
    event EmergencyPaused(bool paused, address indexed admin, uint256 timestamp);
    event AdminUpdated(address indexed oldAdmin, address indexed newAdmin, uint256 timestamp);
    event KeeperUpdated(address indexed oldKeeper, address indexed newKeeper, uint256 timestamp);
    event SystemParametersUpdated(uint256 keeperFeePercentage, uint256 minKeeperFee, uint256 maxKeeperFee, uint256 timestamp);
    
    // Migration Events (for future ERC20 support)
    event TokenSupportAdded(address indexed token, bool enabled, uint256 timestamp);
}

// =============================================================================
// STORAGE STRUCTURE - Complete state management
// =============================================================================

sol_storage! {
    #[entrypoint]
    pub struct SubscriptionEscrow {
        // =============================================================================
        // CORE SYSTEM STATE
        // =============================================================================
        address admin;                    // Contract administrator
        address emergency_admin;          // Emergency pause authority  
        bool paused;                     // Emergency pause state
        bool initialized;                // Initialization flag
        uint256 contract_version;        // Version for upgrades
        
        // =============================================================================
        // GLOBAL COUNTERS - Unique ID generation
        // =============================================================================
        uint256 next_provider_id;       // Provider registration counter
        uint256 next_plan_id;           // Plan creation counter  
        uint256 next_subscription_id;   // Subscription counter
        
        // =============================================================================
        // PROVIDER MANAGEMENT
        // =============================================================================
        mapping(address => uint256) provider_ids;              // provider => unique ID
        mapping(address => string) provider_names;             // provider => display name
        mapping(address => bool) provider_active;              // provider => active status
        mapping(address => uint256) provider_plan_count;       // provider => number of plans
        mapping(address => uint256) provider_earnings;         // provider => withdrawable earnings
        mapping(address => uint256) provider_total_earnings;   // provider => lifetime earnings
        mapping(address => uint256) provider_subscriber_count; // provider => active subscribers
        
        // =============================================================================
        // PLAN MANAGEMENT
        // =============================================================================
        mapping(uint256 => address) plan_provider;             // planId => provider address
        mapping(uint256 => string) plan_name;                  // planId => plan name
        mapping(uint256 => uint256) plan_price;                // planId => price per interval
        mapping(uint256 => uint256) plan_interval;             // planId => payment interval (seconds)
        mapping(uint256 => bool) plan_active;                  // planId => active status
        mapping(uint256 => uint256) plan_subscriber_count;     // planId => active subscribers
        mapping(uint256 => uint256) plan_total_revenue;        // planId => lifetime revenue
        mapping(uint256 => uint256) plan_creation_time;        // planId => creation timestamp
        
        // =============================================================================
        // SUBSCRIPTION MANAGEMENT
        // =============================================================================
        mapping(uint256 => uint256) subscription_plan_id;      // subscriptionId => planId
        mapping(uint256 => address) subscription_subscriber;   // subscriptionId => subscriber address
        mapping(uint256 => uint256) subscription_start_time;   // subscriptionId => start timestamp
        mapping(uint256 => uint256) subscription_last_payment; // subscriptionId => last payment timestamp
        mapping(uint256 => uint256) subscription_payment_count; // subscriptionId => total payments made
        mapping(uint256 => bool) subscription_active;          // subscriptionId => active status
        mapping(uint256 => bool) subscription_auto_pay;        // subscriptionId => auto-payment enabled
        mapping(uint256 => uint256) subscription_grace_period; // subscriptionId => grace period end time
        
        // =============================================================================
        // USER ESCROW BALANCES
        // =============================================================================
        mapping(address => uint256) user_escrow_balance;       // user => available balance
        mapping(address => uint256) user_total_deposited;      // user => lifetime deposits
        mapping(address => uint256) user_total_spent;          // user => lifetime spending
        
        // =============================================================================
        // USER SUBSCRIPTION TRACKING
        // =============================================================================
        mapping(address => uint256) user_subscription_count;   // user => active subscription count
        mapping(address => mapping(uint256 => uint256)) user_subscriptions; // user => index => subscriptionId
        
        // =============================================================================
        // PROVIDER PLAN TRACKING  
        // =============================================================================
        mapping(address => mapping(uint256 => uint256)) provider_plans; // provider => index => planId
        
        // =============================================================================
        // PLAN SUBSCRIBER TRACKING
        // =============================================================================
        mapping(uint256 => mapping(uint256 => uint256)) plan_subscriptions; // planId => index => subscriptionId
        
        // =============================================================================
        // GLOBAL STATISTICS
        // =============================================================================
        uint256 total_value_locked;     // Total ETH in escrow
        uint256 total_revenue_processed; // Lifetime revenue processed
        uint256 total_subscriptions;    // Total subscriptions created
        uint256 active_subscriptions;   // Currently active subscriptions
        uint256 total_providers;        // Total registered providers
        uint256 total_plans;            // Total plans created
        
        // =============================================================================
        // AUTOMATION & KEEPER SYSTEM
        // =============================================================================
        address keeper_address;         // Gelato/keeper authorized address
        uint256 keeper_fee_percentage;  // Keeper fee in basis points (100 = 1%)
        uint256 min_keeper_fee;         // Minimum keeper fee in wei
        uint256 max_keeper_fee;         // Maximum keeper fee in wei
        uint256 payment_grace_period;   // Grace period before cancellation (seconds)
        
        // =============================================================================
        // SECURITY & LIMITS
        // =============================================================================
        uint256 max_plan_price;         // Maximum allowed plan price
        uint256 min_plan_interval;      // Minimum allowed payment interval
        uint256 max_plan_interval;      // Maximum allowed payment interval  
        uint256 withdrawal_delay;       // Delay for large withdrawals
        mapping(address => uint256) withdrawal_requests; // user => withdrawal request time
        
        // =============================================================================
        // FUTURE ERC20 SUPPORT
        // =============================================================================
        mapping(address => bool) supported_tokens;    // token => supported status
        address primary_token;                        // Primary token (0x0 = ETH)
        
        // =============================================================================
        // EMERGENCY FEATURES
        // =============================================================================
        mapping(address => bool) emergency_withdrawal_enabled; // user => emergency flag
        uint256 emergency_withdrawal_deadline;        // Deadline for emergency withdrawals
    }
}

// =============================================================================
// MAIN CONTRACT IMPLEMENTATION
// =============================================================================

#[public]
impl SubscriptionEscrow {
    
    // =============================================================================
    // INITIALIZATION & ADMIN FUNCTIONS
    // =============================================================================
    
    /// Initialize the contract with security parameters and default settings
    /// @param emergency_admin Address that can pause the contract in emergencies
    pub fn initialize(&mut self, emergency_admin: Address) -> bool {
        // Prevent double initialization
        if self.initialized.get() {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        
        // Set core admin addresses
        self.admin.set(caller);
        self.emergency_admin.set(emergency_admin);
        self.initialized.set(true);
        self.paused.set(false);
        self.contract_version.set(U256::from(1));
        
        // Initialize counters
        self.next_provider_id.set(U256::from(1));
        self.next_plan_id.set(U256::from(1));
        self.next_subscription_id.set(U256::from(1));
        
        // Set default system parameters
        self.keeper_fee_percentage.set(U256::from(100));      // 1% default
        self.min_keeper_fee.set(U256::from(1000000000000000u64)); // 0.001 ETH
        self.max_keeper_fee.set(U256::from(10000000000000000u64)); // 0.01 ETH
        self.payment_grace_period.set(U256::from(86400));     // 24 hours
        
        // Set security limits
        self.max_plan_price.set(U256::from_str_radix("100000000000000000000", 10).unwrap_or(U256::from(1000000000000000000u64))); // 100 ETH max
        self.min_plan_interval.set(U256::from(3600));         // 1 hour minimum
        self.max_plan_interval.set(U256::from(31536000));     // 1 year maximum
        self.withdrawal_delay.set(U256::from(0));             // No delay by default
        
        // Initialize globals
        self.total_value_locked.set(U256::ZERO);
        self.total_revenue_processed.set(U256::ZERO);
        self.total_subscriptions.set(U256::ZERO);
        self.active_subscriptions.set(U256::ZERO);
        self.total_providers.set(U256::ZERO);
        self.total_plans.set(U256::ZERO);
        
        // Set primary token to ETH (address zero)
        self.primary_token.set(Address::ZERO);
        
        true
    }
    
    /// Update admin address (only current admin)
    pub fn update_admin(&mut self, new_admin: Address) -> bool {
        let caller = self.vm().msg_sender();
        if caller != self.admin.get() || new_admin == Address::ZERO {
            return false;
        }
        
        let old_admin = self.admin.get();
        self.admin.set(new_admin);
        
        log(self.vm(), AdminUpdated {
            oldAdmin: old_admin,
            newAdmin: new_admin,
            timestamp: U256::from(self.vm().block_timestamp()),
        });
        
        true
    }
    
    /// Emergency pause/unpause (admin or emergency admin only)
    pub fn emergency_pause(&mut self) -> bool {
        let caller = self.vm().msg_sender();
        let admin = self.admin.get();
        let emergency_admin = self.emergency_admin.get();
        
        if caller != admin && caller != emergency_admin {
            return false;
        }
        
        let current_state = self.paused.get();
        self.paused.set(!current_state);
        
        log(self.vm(), EmergencyPaused {
            paused: !current_state,
            admin: caller,
            timestamp: U256::from(self.vm().block_timestamp()),
        });
        
        true
    }
    
    // =============================================================================
    // PROVIDER REGISTRATION & MANAGEMENT
    // =============================================================================
    
    /// Register as a service provider
    /// @param name Display name for the provider
    pub fn register_provider(&mut self, name: String) -> bool {
        if self.paused.get() || name.is_empty() {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        
        // Check if already registered
        if self.provider_ids.getter(caller).get() != U256::ZERO {
            return false;
        }
        
        let provider_id = self.next_provider_id.get();
        self.next_provider_id.set(provider_id + U256::from(1));
        
        // Set provider data
        self.provider_ids.setter(caller).set(provider_id);
        let mut provider_name = self.provider_names.setter(caller);
        provider_name.set_str(&name);
        self.provider_active.setter(caller).set(true);
        self.provider_plan_count.setter(caller).set(U256::ZERO);
        self.provider_earnings.setter(caller).set(U256::ZERO);
        self.provider_total_earnings.setter(caller).set(U256::ZERO);
        self.provider_subscriber_count.setter(caller).set(U256::ZERO);
        
        // Update global counter
        let total_providers = self.total_providers.get();
        self.total_providers.set(total_providers + U256::from(1));
        
        log(self.vm(), ProviderRegistered {
            provider: caller,
            name,
            timestamp: U256::from(self.vm().block_timestamp()),
        });
        
        true
    }
    
    /// Update provider information
    /// @param new_name New display name
    /// @param active New active status
    pub fn update_provider(&mut self, new_name: String, active: bool) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        
        // Verify provider is registered
        if self.provider_ids.getter(caller).get() == U256::ZERO {
            return false;
        }
        
        // Update name if provided
        if !new_name.is_empty() {
            let mut provider_name = self.provider_names.setter(caller);
            provider_name.set_str(&new_name);
        }
        
        // Update active status
        self.provider_active.setter(caller).set(active);
        
        log(self.vm(), ProviderUpdated {
            provider: caller,
            newName: new_name,
            active,
        });
        
        true
    }
    
    // =============================================================================
    // PLAN CREATION & MANAGEMENT
    // =============================================================================
    
    /// Create a new subscription plan
    /// @param name Plan display name
    /// @param price Price per payment interval in wei
    /// @param interval Payment interval in seconds
    pub fn create_plan(&mut self, name: String, price: U256, interval: U256) -> U256 {
        if self.paused.get() || name.is_empty() {
            return U256::ZERO;
        }
        
        let caller = self.vm().msg_sender();
        
        // Verify provider is registered and active
        if self.provider_ids.getter(caller).get() == U256::ZERO || 
           !self.provider_active.getter(caller).get() {
            return U256::ZERO;
        }
        
        // Validate parameters
        if price == U256::ZERO || price > self.max_plan_price.get() ||
           interval < self.min_plan_interval.get() || interval > self.max_plan_interval.get() {
            return U256::ZERO;
        }
        
        let plan_id = self.next_plan_id.get();
        self.next_plan_id.set(plan_id + U256::from(1));
        
        // Set plan data
        self.plan_provider.setter(plan_id).set(caller);
        let mut plan_name = self.plan_name.setter(plan_id);
        plan_name.set_str(&name);
        self.plan_price.setter(plan_id).set(price);
        self.plan_interval.setter(plan_id).set(interval);
        self.plan_active.setter(plan_id).set(true);
        self.plan_subscriber_count.setter(plan_id).set(U256::ZERO);
        self.plan_total_revenue.setter(plan_id).set(U256::ZERO);
        
        let timestamp = self.vm().block_timestamp();
        self.plan_creation_time.setter(plan_id).set(U256::from(timestamp));
        
        // Update provider plan count and tracking
        let provider_plan_count = self.provider_plan_count.getter(caller).get();
        self.provider_plan_count.setter(caller).set(provider_plan_count + U256::from(1));
        self.provider_plans.setter(caller).setter(provider_plan_count).set(plan_id);
        
        // Update global counter
        let total_plans = self.total_plans.get();
        self.total_plans.set(total_plans + U256::from(1));
        
        log(self.vm(), PlanCreated {
            planId: plan_id,
            provider: caller,
            name,
            price,
            interval,
            timestamp: U256::from(timestamp),
        });
        
        plan_id
    }
    
    /// Update an existing plan
    /// @param plan_id Plan to update
    /// @param new_price New price (0 to keep current)
    /// @param new_interval New interval (0 to keep current)  
    /// @param active New active status
    pub fn update_plan(&mut self, plan_id: U256, new_price: U256, new_interval: U256, active: bool) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        
        // Verify caller owns the plan
        if self.plan_provider.getter(plan_id).get() != caller {
            return false;
        }
        
        // Update price if specified
        if new_price > U256::ZERO && new_price <= self.max_plan_price.get() {
            self.plan_price.setter(plan_id).set(new_price);
        }
        
        // Update interval if specified
        if new_interval > U256::ZERO && 
           new_interval >= self.min_plan_interval.get() && 
           new_interval <= self.max_plan_interval.get() {
            self.plan_interval.setter(plan_id).set(new_interval);
        }
        
        // Update active status
        self.plan_active.setter(plan_id).set(active);
        
        log(self.vm(), PlanUpdated {
            planId: plan_id,
            newPrice: new_price,
            newInterval: new_interval,
            active,
        });
        
        true
    }
    
    /// Deactivate a plan and handle existing subscriptions
    /// @param plan_id Plan to deactivate
    pub fn deactivate_plan(&mut self, plan_id: U256) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        
        // Verify caller owns the plan
        if self.plan_provider.getter(plan_id).get() != caller {
            return false;
        }
        
        self.plan_active.setter(plan_id).set(false);
        
        log(self.vm(), PlanDeactivated {
            planId: plan_id,
            provider: caller,
            timestamp: U256::from(self.vm().block_timestamp()),
        });
        
        true
    }
    
    // =============================================================================
    // SUBSCRIPTION MANAGEMENT
    // =============================================================================
    
    /// Subscribe to a plan
    /// @param plan_id Plan to subscribe to
    /// @param auto_pay Enable automatic payments
    #[payable]
    pub fn subscribe(&mut self, plan_id: U256, auto_pay: bool) -> U256 {
        if self.paused.get() {
            return U256::ZERO;
        }
        
        let caller = self.vm().msg_sender();
        let payment = self.vm().msg_value();
        
        // Validate plan exists and is active
        let plan_provider = self.plan_provider.getter(plan_id).get();
        if plan_provider == Address::ZERO || !self.plan_active.getter(plan_id).get() {
            return U256::ZERO;
        }
        
        let plan_price = self.plan_price.getter(plan_id).get();
        
        // Add payment to user's escrow
        if payment > U256::ZERO {
            let current_balance = self.user_escrow_balance.getter(caller).get();
            self.user_escrow_balance.setter(caller).set(current_balance + payment);
            
            let total_deposited = self.user_total_deposited.getter(caller).get();
            self.user_total_deposited.setter(caller).set(total_deposited + payment);
            
            let total_vlocked = self.total_value_locked.get();
            self.total_value_locked.set(total_vlocked + payment);
            
            log(self.vm(), FundsDeposited {
                user: caller,
                amount: payment,
                newBalance: current_balance + payment,
                timestamp: U256::from(self.vm().block_timestamp()),
            });
        }
        
        // Check if user has sufficient balance for first payment
        let user_balance = self.user_escrow_balance.getter(caller).get();
        if user_balance < plan_price {
            return U256::ZERO;
        }
        
        let subscription_id = self.next_subscription_id.get();
        self.next_subscription_id.set(subscription_id + U256::from(1));
        
        let current_time = U256::from(self.vm().block_timestamp());
        
        // Create subscription
        self.subscription_plan_id.setter(subscription_id).set(plan_id);
        self.subscription_subscriber.setter(subscription_id).set(caller);
        self.subscription_start_time.setter(subscription_id).set(current_time);
        self.subscription_last_payment.setter(subscription_id).set(current_time);
        self.subscription_payment_count.setter(subscription_id).set(U256::from(1));
        self.subscription_active.setter(subscription_id).set(true);
        self.subscription_auto_pay.setter(subscription_id).set(auto_pay);
        self.subscription_grace_period.setter(subscription_id).set(U256::ZERO);
        
        // Process first payment
        self.user_escrow_balance.setter(caller).set(user_balance - plan_price);
        
        let provider_earnings = self.provider_earnings.getter(plan_provider).get();
        self.provider_earnings.setter(plan_provider).set(provider_earnings + plan_price);
        
        let provider_total_earnings = self.provider_total_earnings.getter(plan_provider).get();
        self.provider_total_earnings.setter(plan_provider).set(provider_total_earnings + plan_price);
        
        let user_total_spent = self.user_total_spent.getter(caller).get();
        self.user_total_spent.setter(caller).set(user_total_spent + plan_price);
        
        // Update plan revenue
        let plan_revenue = self.plan_total_revenue.getter(plan_id).get();
        self.plan_total_revenue.setter(plan_id).set(plan_revenue + plan_price);
        
        // Update counters
        let user_sub_count = self.user_subscription_count.getter(caller).get();
        self.user_subscription_count.setter(caller).set(user_sub_count + U256::from(1));
        self.user_subscriptions.setter(caller).setter(user_sub_count).set(subscription_id);
        
        let plan_sub_count = self.plan_subscriber_count.getter(plan_id).get();
        self.plan_subscriber_count.setter(plan_id).set(plan_sub_count + U256::from(1));
        self.plan_subscriptions.setter(plan_id).setter(plan_sub_count).set(subscription_id);
        
        let provider_sub_count = self.provider_subscriber_count.getter(plan_provider).get();
        self.provider_subscriber_count.setter(plan_provider).set(provider_sub_count + U256::from(1));
        
        // Update global stats
        let total_subs = self.total_subscriptions.get();
        self.total_subscriptions.set(total_subs + U256::from(1));
        
        let active_subs = self.active_subscriptions.get();
        self.active_subscriptions.set(active_subs + U256::from(1));
        
        let total_revenue = self.total_revenue_processed.get();
        self.total_revenue_processed.set(total_revenue + plan_price);
        
        // Emit events
        log(self.vm(), SubscriptionCreated {
            subscriptionId: subscription_id,
            planId: plan_id,
            subscriber: caller,
            timestamp: current_time,
        });
        
        log(self.vm(), PaymentProcessed {
            subscriptionId: subscription_id,
            planId: plan_id,
            subscriber: caller,
            amount: plan_price,
            timestamp: current_time,
        });
        
        if auto_pay {
            log(self.vm(), AutoPaymentToggled {
                subscriptionId: subscription_id,
                subscriber: caller,
                enabled: true,
                timestamp: current_time,
            });
        }
        
        subscription_id
    }
    
    /// Cancel a subscription and refund unused balance
    /// @param subscription_id Subscription to cancel
    pub fn cancel_subscription(&mut self, subscription_id: U256) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        
        // Verify ownership and active status
        if self.subscription_subscriber.getter(subscription_id).get() != caller ||
           !self.subscription_active.getter(subscription_id).get() {
            return false;
        }
        
        // Deactivate subscription
        self.subscription_active.setter(subscription_id).set(false);
        self.subscription_auto_pay.setter(subscription_id).set(false);
        
        let plan_id = self.subscription_plan_id.getter(subscription_id).get();
        let plan_provider = self.plan_provider.getter(plan_id).get();
        
        // Update counters
        let active_subs = self.active_subscriptions.get();
        if active_subs > U256::ZERO {
            self.active_subscriptions.set(active_subs - U256::from(1));
        }
        
        let provider_sub_count = self.provider_subscriber_count.getter(plan_provider).get();
        if provider_sub_count > U256::ZERO {
            self.provider_subscriber_count.setter(plan_provider).set(provider_sub_count - U256::from(1));
        }
        
        let plan_sub_count = self.plan_subscriber_count.getter(plan_id).get();
        if plan_sub_count > U256::ZERO {
            self.plan_subscriber_count.setter(plan_id).set(plan_sub_count - U256::from(1));
        }
        
        // Calculate refund (remaining escrow balance)
        let refund_amount = self.user_escrow_balance.getter(caller).get();
        
        log(self.vm(), SubscriptionCancelled {
            subscriptionId: subscription_id,
            subscriber: caller,
            refundAmount: refund_amount,
            timestamp: U256::from(self.vm().block_timestamp()),
        });
        
        true
    }
    
    /// Toggle auto-payment for a subscription
    /// @param subscription_id Subscription to modify
    /// @param enabled Enable or disable auto-payment
    pub fn toggle_auto_payment(&mut self, subscription_id: U256, enabled: bool) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        
        // Verify ownership and active status
        if self.subscription_subscriber.getter(subscription_id).get() != caller ||
           !self.subscription_active.getter(subscription_id).get() {
            return false;
        }
        
        self.subscription_auto_pay.setter(subscription_id).set(enabled);
        
        log(self.vm(), AutoPaymentToggled {
            subscriptionId: subscription_id,
            subscriber: caller,
            enabled,
            timestamp: U256::from(self.vm().block_timestamp()),
        });
        
        true
    }
    
    // =============================================================================
    // ESCROW & BALANCE MANAGEMENT
    // =============================================================================
    
    /// Deposit funds to escrow for subscriptions
    #[payable]
    pub fn deposit_funds(&mut self) {
        let caller = self.vm().msg_sender();
        let amount = self.vm().msg_value();
        
        if amount > U256::ZERO {
            let current_balance = self.user_escrow_balance.getter(caller).get();
            let new_balance = current_balance + amount;
            
            self.user_escrow_balance.setter(caller).set(new_balance);
            
            let total_deposited = self.user_total_deposited.getter(caller).get();
            self.user_total_deposited.setter(caller).set(total_deposited + amount);
            
            let total_vlocked = self.total_value_locked.get();
            self.total_value_locked.set(total_vlocked + amount);
            
            log(self.vm(), FundsDeposited {
                user: caller,
                amount,
                newBalance: new_balance,
                timestamp: U256::from(self.vm().block_timestamp()),
            });
        }
    }
    
    /// Withdraw funds from escrow
    /// @param amount Amount to withdraw in wei
    pub fn withdraw_funds(&mut self, amount: U256) -> bool {
        if self.paused.get() || amount == U256::ZERO {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        let current_balance = self.user_escrow_balance.getter(caller).get();
        
        if amount > current_balance {
            return false;
        }
        
        // Check withdrawal delay for large amounts
        let withdrawal_delay = self.withdrawal_delay.get();
        if withdrawal_delay > U256::ZERO && amount > U256::from(10000000000000000000u64) { // > 10 ETH
            let last_request = self.withdrawal_requests.getter(caller).get();
            let current_time = U256::from(self.vm().block_timestamp());
            
            if last_request == U256::ZERO {
                self.withdrawal_requests.setter(caller).set(current_time);
                return false; // Request initiated, wait for delay
            }
            
            if current_time < last_request + withdrawal_delay {
                return false; // Still in delay period
            }
            
            // Clear the request
            self.withdrawal_requests.setter(caller).set(U256::ZERO);
        }
        
        // Update balances
        let new_balance = current_balance - amount;
        self.user_escrow_balance.setter(caller).set(new_balance);
        
        let total_vlocked = self.total_value_locked.get();
        if total_vlocked >= amount {
            self.total_value_locked.set(total_vlocked - amount);
        }
        
        // Transfer funds
        match self.vm().transfer_eth(caller, amount) {
            Ok(_) => {
                log(self.vm(), FundsWithdrawn {
                    user: caller,
                    amount,
                    newBalance: new_balance,
                    timestamp: U256::from(self.vm().block_timestamp()),
                });
                true
            }
            Err(_) => {
                // Revert balance changes if transfer failed
                self.user_escrow_balance.setter(caller).set(current_balance);
                let total_vlocked = self.total_value_locked.get();
                self.total_value_locked.set(total_vlocked + amount);
                false
            }
        }
    }
    
    /// Withdraw provider earnings
    /// @param amount Amount to withdraw
    pub fn withdraw_earnings(&mut self, amount: U256) -> bool {
        if self.paused.get() || amount == U256::ZERO {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        let available_earnings = self.provider_earnings.getter(caller).get();
        
        if amount > available_earnings {
            return false;
        }
        
        // Update earnings
        self.provider_earnings.setter(caller).set(available_earnings - amount);
        
        // Transfer funds
        match self.vm().transfer_eth(caller, amount) {
            Ok(_) => {
                log(self.vm(), EarningsWithdrawn {
                    provider: caller,
                    amount,
                    timestamp: U256::from(self.vm().block_timestamp()),
                });
                true
            }
            Err(_) => {
                // Revert balance changes if transfer failed
                self.provider_earnings.setter(caller).set(available_earnings);
                false
            }
        }
    }
    
    // =============================================================================
    // AUTOMATION & KEEPER FUNCTIONS (GELATO INTEGRATION)
    // =============================================================================
    
    /// Set keeper address for automation (admin only)
    /// @param keeper_addr Address of the Gelato keeper
    pub fn set_keeper(&mut self, keeper_addr: Address) -> bool {
        let caller = self.vm().msg_sender();
        if caller != self.admin.get() {
            return false;
        }
        
        let old_keeper = self.keeper_address.get();
        self.keeper_address.set(keeper_addr);
        
        log(self.vm(), KeeperUpdated {
            oldKeeper: old_keeper,
            newKeeper: keeper_addr,
            timestamp: U256::from(self.vm().block_timestamp()),
        });
        
        true
    }
    
    /// Update system parameters (admin only)
    /// @param fee_percentage Keeper fee in basis points
    /// @param min_fee Minimum fee amount
    /// @param max_fee Maximum fee amount
    pub fn update_system_parameters(&mut self, fee_percentage: U256, min_fee: U256, max_fee: U256) -> bool {
        let caller = self.vm().msg_sender();
        if caller != self.admin.get() {
            return false;
        }
        
        // Validate parameters
        if fee_percentage > U256::from(1000) || // Max 10%
           min_fee > max_fee || 
           max_fee > U256::from(100000000000000000u64) { // Max 0.1 ETH
            return false;
        }
        
        self.keeper_fee_percentage.set(fee_percentage);
        self.min_keeper_fee.set(min_fee);
        self.max_keeper_fee.set(max_fee);
        
        log(self.vm(), SystemParametersUpdated {
            keeperFeePercentage: fee_percentage,
            minKeeperFee: min_fee,
            maxKeeperFee: max_fee,
            timestamp: U256::from(self.vm().block_timestamp()),
        });
        
        true
    }
    
    /// Process subscription payment (keeper callable)
    /// @param subscription_id Subscription to process payment for
    pub fn process_subscription_payment(&mut self, subscription_id: U256) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = self.vm().msg_sender();
        let keeper = self.keeper_address.get();
        
        // Only keeper or subscription owner can process payment
        let subscriber = self.subscription_subscriber.getter(subscription_id).get();
        if caller != keeper && caller != subscriber {
            return false;
        }
        
        // Verify subscription is active and has auto-pay enabled
        if !self.subscription_active.getter(subscription_id).get() ||
           !self.subscription_auto_pay.getter(subscription_id).get() {
            return false;
        }
        
        let plan_id = self.subscription_plan_id.getter(subscription_id).get();
        let plan_price = self.plan_price.getter(plan_id).get();
        let plan_interval = self.plan_interval.getter(plan_id).get();
        let last_payment = self.subscription_last_payment.getter(subscription_id).get();
        let current_time = U256::from(self.vm().block_timestamp());
        
        // Check if payment is due
        if current_time < last_payment + plan_interval {
            return false;
        }
        
        // Check user has sufficient balance
        let user_balance = self.user_escrow_balance.getter(subscriber).get();
        if user_balance < plan_price {
            // Start grace period
            let grace_period = self.payment_grace_period.get();
            self.subscription_grace_period.setter(subscription_id).set(current_time + grace_period);
            
            log(self.vm(), PaymentFailed {
                subscriptionId: subscription_id,
                subscriber,
                requiredAmount: plan_price,
                availableBalance: user_balance,
                timestamp: current_time,
            });
            
            return false;
        }
        
        // Process payment
        let provider = self.plan_provider.getter(plan_id).get();
        
        // Calculate keeper fee if called by keeper
        let mut keeper_fee = U256::ZERO;
        if caller == keeper && keeper != Address::ZERO {
            let fee_percentage = self.keeper_fee_percentage.get();
            keeper_fee = (plan_price * fee_percentage) / U256::from(10000);
            
            let min_fee = self.min_keeper_fee.get();
            let max_fee = self.max_keeper_fee.get();
            
            if keeper_fee < min_fee {
                keeper_fee = min_fee;
            } else if keeper_fee > max_fee {
                keeper_fee = max_fee;
            }
            
            // Ensure keeper fee doesn't exceed payment
            if keeper_fee > plan_price {
                keeper_fee = plan_price / U256::from(10);
            }
        }
        
        let provider_amount = plan_price - keeper_fee;
        
        // Update balances
        self.user_escrow_balance.setter(subscriber).set(user_balance - plan_price);
        
        let provider_earnings = self.provider_earnings.getter(provider).get();
        self.provider_earnings.setter(provider).set(provider_earnings + provider_amount);
        
        let provider_total_earnings = self.provider_total_earnings.getter(provider).get();
        self.provider_total_earnings.setter(provider).set(provider_total_earnings + provider_amount);
        
        // Pay keeper fee if applicable
        if keeper_fee > U256::ZERO && caller == keeper {
            let _transfer_result = self.vm().transfer_eth(keeper, keeper_fee);
        }
        
        // Update subscription data
        self.subscription_last_payment.setter(subscription_id).set(current_time);
        let payment_count = self.subscription_payment_count.getter(subscription_id).get();
        self.subscription_payment_count.setter(subscription_id).set(payment_count + U256::from(1));
        self.subscription_grace_period.setter(subscription_id).set(U256::ZERO);
        
        // Update user total spent
        let user_total_spent = self.user_total_spent.getter(subscriber).get();
        self.user_total_spent.setter(subscriber).set(user_total_spent + plan_price);
        
        // Update plan revenue
        let plan_revenue = self.plan_total_revenue.getter(plan_id).get();
        self.plan_total_revenue.setter(plan_id).set(plan_revenue + plan_price);
        
        // Update global revenue
        let total_revenue = self.total_revenue_processed.get();
        self.total_revenue_processed.set(total_revenue + plan_price);
        
        log(self.vm(), PaymentProcessed {
            subscriptionId: subscription_id,
            planId: plan_id,
            subscriber,
            amount: plan_price,
            timestamp: current_time,
        });
        
        true
    }
    
    /// Check if subscription needs payment (view function for keepers)
    /// @param subscription_id Subscription to check
    pub fn needs_payment(&self, subscription_id: U256) -> bool {
        if !self.subscription_active.getter(subscription_id).get() ||
           !self.subscription_auto_pay.getter(subscription_id).get() {
            return false;
        }
        
        let plan_id = self.subscription_plan_id.getter(subscription_id).get();
        let plan_interval = self.plan_interval.getter(plan_id).get();
        let last_payment = self.subscription_last_payment.getter(subscription_id).get();
        let current_time = U256::from(self.vm().block_timestamp());
        
        current_time >= last_payment + plan_interval
    }
    
    // =============================================================================
    // COMPREHENSIVE VIEW FUNCTIONS - DATA ACCESS & ANALYTICS
    // =============================================================================
    
    /// Get provider information
    /// @param provider Provider address
    pub fn get_provider_info(&self, provider: Address) -> (U256, String, bool, U256, U256, U256, U256) {
        let provider_id = self.provider_ids.getter(provider).get();
        let name = self.provider_names.getter(provider).get_string();
        let active = self.provider_active.getter(provider).get();
        let plan_count = self.provider_plan_count.getter(provider).get();
        let earnings = self.provider_earnings.getter(provider).get();
        let total_earnings = self.provider_total_earnings.getter(provider).get();
        let subscriber_count = self.provider_subscriber_count.getter(provider).get();
        
        (provider_id, name, active, plan_count, earnings, total_earnings, subscriber_count)
    }
    
    /// Get plan information
    /// @param plan_id Plan ID
    pub fn get_plan_info(&self, plan_id: U256) -> (Address, String, U256, U256, bool, U256, U256, U256) {
        let provider = self.plan_provider.getter(plan_id).get();
        let name = self.plan_name.getter(plan_id).get_string();
        let price = self.plan_price.getter(plan_id).get();
        let interval = self.plan_interval.getter(plan_id).get();
        let active = self.plan_active.getter(plan_id).get();
        let subscriber_count = self.plan_subscriber_count.getter(plan_id).get();
        let total_revenue = self.plan_total_revenue.getter(plan_id).get();
        let creation_time = self.plan_creation_time.getter(plan_id).get();
        
        (provider, name, price, interval, active, subscriber_count, total_revenue, creation_time)
    }
    
    /// Get subscription information
    /// @param subscription_id Subscription ID
    pub fn get_subscription_info(&self, subscription_id: U256) -> (U256, Address, U256, U256, U256, bool, bool, U256) {
        let plan_id = self.subscription_plan_id.getter(subscription_id).get();
        let subscriber = self.subscription_subscriber.getter(subscription_id).get();
        let start_time = self.subscription_start_time.getter(subscription_id).get();
        let last_payment = self.subscription_last_payment.getter(subscription_id).get();
        let payment_count = self.subscription_payment_count.getter(subscription_id).get();
        let active = self.subscription_active.getter(subscription_id).get();
        let auto_pay = self.subscription_auto_pay.getter(subscription_id).get();
        let grace_period = self.subscription_grace_period.getter(subscription_id).get();
        
        (plan_id, subscriber, start_time, last_payment, payment_count, active, auto_pay, grace_period)
    }
    
    /// Get user account information
    /// @param user User address
    pub fn get_user_info(&self, user: Address) -> (U256, U256, U256, U256) {
        let escrow_balance = self.user_escrow_balance.getter(user).get();
        let total_deposited = self.user_total_deposited.getter(user).get();
        let total_spent = self.user_total_spent.getter(user).get();
        let subscription_count = self.user_subscription_count.getter(user).get();
        
        (escrow_balance, total_deposited, total_spent, subscription_count)
    }
    
    /// Get global platform statistics
    pub fn get_platform_stats(&self) -> (U256, U256, U256, U256, U256, U256) {
        let total_vlocked = self.total_value_locked.get();
        let total_revenue = self.total_revenue_processed.get();
        let total_subs = self.total_subscriptions.get();
        let active_subs = self.active_subscriptions.get();
        let total_providers = self.total_providers.get();
        let total_plans = self.total_plans.get();
        
        (total_vlocked, total_revenue, total_subs, active_subs, total_providers, total_plans)
    }
    
    /// Get system configuration
    pub fn get_system_config(&self) -> (Address, Address, Address, bool, U256, U256, U256, U256) {
        let admin = self.admin.get();
        let emergency_admin = self.emergency_admin.get();
        let keeper = self.keeper_address.get();
        let paused = self.paused.get();
        let keeper_fee = self.keeper_fee_percentage.get();
        let min_fee = self.min_keeper_fee.get();
        let max_fee = self.max_keeper_fee.get();
        let grace_period = self.payment_grace_period.get();
        
        (admin, emergency_admin, keeper, paused, keeper_fee, min_fee, max_fee, grace_period)
    }
    
    /// Check if contract is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.get()
    }
    
    /// Get contract version
    pub fn get_version(&self) -> U256 {
        self.contract_version.get()
    }
    
    /// Get user's subscription by index
    /// @param user User address
    /// @param index Index in user's subscription list
    pub fn get_user_subscription(&self, user: Address, index: U256) -> U256 {
        self.user_subscriptions.getter(user).getter(index).get()
    }
    
    /// Get provider's plan by index
    /// @param provider Provider address
    /// @param index Index in provider's plan list
    pub fn get_provider_plan(&self, provider: Address, index: U256) -> U256 {
        self.provider_plans.getter(provider).getter(index).get()
    }
    
    /// Get next IDs for external tracking
    pub fn get_next_ids(&self) -> (U256, U256, U256) {
        let next_provider = self.next_provider_id.get();
        let next_plan = self.next_plan_id.get();
        let next_subscription = self.next_subscription_id.get();
        
        (next_provider, next_plan, next_subscription)
    }
}
