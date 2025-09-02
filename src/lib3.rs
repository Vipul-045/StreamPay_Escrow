#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;
use alloc::{string::String, vec::Vec};

use stylus_sdk::{
    alloy_primitives::{Address, U256}, 
    prelude::*,
    storage::*,
    alloy_sol_types::sol,
    evm,
};

// Define simplified Solidity error types
sol! {
    error Unauthorized();
    error InvalidInput();
    error InsufficientFunds();
    error NotFound();
}

// Events for monitoring
sol! {
    event Initialized(address indexed admin);
    event ProviderRegistered(address indexed provider, string name);
    event PlanCreated(uint256 indexed planId, address indexed provider, uint256 price, uint256 interval);
    event SubscriptionCreated(uint256 indexed subscriptionId, address indexed user, uint256 indexed planId);
    event PaymentProcessed(address indexed from, address indexed to, uint256 amount);
    event BalanceDeposited(address indexed user, uint256 amount);
    event EmergencyWithdrawal(address indexed user, uint256 amount);
}

// Simplified error enum
#[derive(SolidityError)]
pub enum SubscriptionError {
    Unauthorized(Unauthorized),
    InvalidInput(InvalidInput),
    InsufficientFunds(InsufficientFunds),
    NotFound(NotFound),
}

// Main production contract storage
sol_storage! {
    #[entrypoint]
    pub struct SubscriptionEscrow {
        // Core admin controls
        address admin;
        uint256 protocol_fee_percentage;
        
        // Counter state
        uint256 next_plan_id;
        uint256 next_subscription_id;
        uint256 total_value_locked;
        uint256 total_protocol_fees;
        
        // Provider management
        mapping(address => bool) registered_providers;
        mapping(address => uint256) provider_earnings;
        
        // Plan management  
        mapping(uint256 => address) plan_provider;
        mapping(uint256 => uint256) plan_price;
        mapping(uint256 => uint256) plan_interval;
        mapping(uint256 => bool) plan_active;
        
        // Subscription management
        mapping(uint256 => uint256) subscription_plan_id;
        mapping(uint256 => address) subscription_subscriber;
        mapping(uint256 => uint256) subscription_created_at;
        mapping(uint256 => uint256) subscription_last_payment;
        mapping(uint256 => bool) subscription_active;
        
        // User financial management
        mapping(address => uint256) user_escrow_balance;
        
        // Gelato automation integration
        mapping(uint256 => bytes32) subscription_gelato_task_id;
        mapping(bytes32 => uint256) gelato_task_to_subscription;
        mapping(address => bool) authorized_gelato_executors;
    }
}

#[public]
impl SubscriptionEscrow {
    
    // ==================== INITIALIZATION ====================
    
    pub fn initialize(&mut self) -> Result<bool, SubscriptionError> {
        if self.admin.get() != Address::ZERO {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        let caller = self.vm().msg_sender();
        self.admin.set(caller);
        self.next_plan_id.set(U256::from(1));
        self.next_subscription_id.set(U256::from(1));
        self.protocol_fee_percentage.set(U256::from(250)); // 2.5%
        
        evm::log(Initialized { admin: caller });
        Ok(true)
    }
    
    // ==================== ADMIN FUNCTIONS ====================
    
    pub fn authorize_gelato_executor(&mut self, executor: Address, authorized: bool) -> Result<bool, SubscriptionError> {
        self.only_admin()?;
        self.authorized_gelato_executors.insert(executor, authorized);
        Ok(true)
    }
    
    // ==================== PROVIDER FUNCTIONS ====================
    
    pub fn register_provider(&mut self, name: String) -> Result<bool, SubscriptionError> {
        let caller = self.vm().msg_sender();
        
        if self.registered_providers.get(caller) {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        if name.len() > 100 {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        self.registered_providers.insert(caller, true);
        self.provider_earnings.insert(caller, U256::ZERO);
        
        evm::log(ProviderRegistered { 
            provider: caller, 
            name: name 
        });
        
        Ok(true)
    }
    
    pub fn create_plan(&mut self, price: U256, interval: U256) -> Result<U256, SubscriptionError> {
        let caller = self.vm().msg_sender();
        self.require_registered_provider(caller)?;
        
        if price.is_zero() || interval.is_zero() {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        let plan_id = self.next_plan_id.get();
        
        // Store plan data
        self.plan_provider.insert(plan_id, caller);
        self.plan_price.insert(plan_id, price);
        self.plan_interval.insert(plan_id, interval);
        self.plan_active.insert(plan_id, true);
        
        // Update counters
        self.next_plan_id.set(plan_id + U256::from(1));
        
        evm::log(PlanCreated {
            planId: plan_id,
            provider: caller,
            price: price,
            interval: interval
        });
        
        Ok(plan_id)
    }
    
    // ==================== SUBSCRIPTION FUNCTIONS ====================
    
    #[payable]
    pub fn subscribe(&mut self, plan_id: U256) -> Result<U256, SubscriptionError> {
        let caller = self.vm().msg_sender();
        let payment = self.vm().msg_value();
        
        // Validate plan exists and is active
        let plan_provider = self.plan_provider.get(plan_id);
        if plan_provider == Address::ZERO {
            return Err(SubscriptionError::NotFound(NotFound {}));
        }
        
        if !self.plan_active.get(plan_id) {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        let plan_price = self.plan_price.get(plan_id);
        
        // Handle payment deposit
        if payment > U256::ZERO {
            self.process_deposit(caller, payment)?;
        }
        
        // Check sufficient balance
        let user_balance = self.user_escrow_balance.get(caller);
        if user_balance < plan_price {
            return Err(SubscriptionError::InsufficientFunds(InsufficientFunds {}));
        }
        
        // Calculate fees
        let protocol_fee = (plan_price * self.protocol_fee_percentage.get()) / U256::from(10000);
        let provider_amount = plan_price - protocol_fee;
        
        // Create subscription
        let subscription_id = self.next_subscription_id.get();
        let current_time = U256::from(self.vm().block_timestamp());
        
        self.subscription_plan_id.insert(subscription_id, plan_id);
        self.subscription_subscriber.insert(subscription_id, caller);
        self.subscription_created_at.insert(subscription_id, current_time);
        self.subscription_last_payment.insert(subscription_id, current_time);
        self.subscription_active.insert(subscription_id, true);
        
        // Update financial state
        self.user_escrow_balance.insert(caller, user_balance - plan_price);
        
        let provider_earnings = self.provider_earnings.get(plan_provider);
        self.provider_earnings.insert(plan_provider, provider_earnings + provider_amount);
        
        let total_fees = self.total_protocol_fees.get();
        self.total_protocol_fees.set(total_fees + protocol_fee);
        
        // Update counters
        self.next_subscription_id.set(subscription_id + U256::from(1));
        
        evm::log(SubscriptionCreated {
            subscriptionId: subscription_id,
            user: caller,
            planId: plan_id
        });
        
        evm::log(PaymentProcessed {
            from: caller,
            to: plan_provider,
            amount: provider_amount
        });
        
        Ok(subscription_id)
    }
    
    // ==================== FINANCIAL FUNCTIONS ====================
    
    #[payable]
    pub fn deposit(&mut self) -> Result<bool, SubscriptionError> {
        let caller = self.vm().msg_sender();
        let amount = self.vm().msg_value();
        
        self.process_deposit(caller, amount)?;
        
        evm::log(BalanceDeposited {
            user: caller,
            amount: amount
        });
        
        Ok(true)
    }
    
    pub fn withdraw(&mut self, amount: U256) -> Result<bool, SubscriptionError> {
        let caller = self.vm().msg_sender();
        let user_balance = self.user_escrow_balance.get(caller);
        
        if amount > user_balance {
            return Err(SubscriptionError::InsufficientFunds(InsufficientFunds {}));
        }
        
        if amount.is_zero() {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        // Update balances
        self.user_escrow_balance.insert(caller, user_balance - amount);
        
        let total_locked = self.total_value_locked.get();
        self.total_value_locked.set(total_locked - amount);
        
        // Transfer ETH - handle error properly
        match self.vm().transfer_eth(caller, amount) {
            Ok(_) => {},
            Err(_) => {
                return Err(SubscriptionError::InvalidInput(InvalidInput {}));
            }
        }
        
        Ok(true)
    }
    
    // ==================== GELATO AUTOMATION ====================
    
    pub fn process_subscription_payment(&mut self, subscription_id: U256) -> Result<bool, SubscriptionError> {
        let caller = self.vm().msg_sender();
        
        // Check if caller is authorized Gelato executor
        if !self.authorized_gelato_executors.get(caller) && caller != self.admin.get() {
            return Err(SubscriptionError::Unauthorized(Unauthorized {}));
        }
        
        if !self.subscription_active.get(subscription_id) {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        let subscriber = self.subscription_subscriber.get(subscription_id);
        let plan_id = self.subscription_plan_id.get(subscription_id);
        let plan_provider = self.plan_provider.get(plan_id);
        let plan_price = self.plan_price.get(plan_id);
        let plan_interval = self.plan_interval.get(plan_id);
        
        // Check if payment is due
        let last_payment = self.subscription_last_payment.get(subscription_id);
        let current_time = U256::from(self.vm().block_timestamp());
        
        if current_time < last_payment + plan_interval {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        // Check user balance
        let user_balance = self.user_escrow_balance.get(subscriber);
        if user_balance < plan_price {
            // Auto-cancel subscription if insufficient funds
            self.subscription_active.insert(subscription_id, false);
            
            return Err(SubscriptionError::InsufficientFunds(InsufficientFunds {}));
        }
        
        // Process payment
        let protocol_fee = (plan_price * self.protocol_fee_percentage.get()) / U256::from(10000);
        let provider_amount = plan_price - protocol_fee;
        
        // Update balances
        self.user_escrow_balance.insert(subscriber, user_balance - plan_price);
        
        let provider_earnings = self.provider_earnings.get(plan_provider);
        self.provider_earnings.insert(plan_provider, provider_earnings + provider_amount);
        
        let total_fees = self.total_protocol_fees.get();
        self.total_protocol_fees.set(total_fees + protocol_fee);
        
        // Update subscription state
        self.subscription_last_payment.insert(subscription_id, current_time);
        
        evm::log(PaymentProcessed {
            from: subscriber,
            to: plan_provider,
            amount: provider_amount
        });
        
        Ok(true)
    }
    
    // ==================== VIEW FUNCTIONS ====================
    
    pub fn get_admin(&self) -> Address {
        self.admin.get()
    }
    
    pub fn get_protocol_fee(&self) -> U256 {
        self.protocol_fee_percentage.get()
    }
    
    pub fn get_user_balance(&self, user: Address) -> U256 {
        self.user_escrow_balance.get(user)
    }
    
    pub fn get_provider_earnings(&self, provider: Address) -> U256 {
        self.provider_earnings.get(provider)
    }
    
    pub fn is_provider_registered(&self, provider: Address) -> bool {
        self.registered_providers.get(provider)
    }
    
    // ==================== INTERNAL HELPER FUNCTIONS ====================
    
    fn only_admin(&self) -> Result<(), SubscriptionError> {
        if self.vm().msg_sender() != self.admin.get() {
            return Err(SubscriptionError::Unauthorized(Unauthorized {}));
        }
        Ok(())
    }
    
    fn require_registered_provider(&self, provider: Address) -> Result<(), SubscriptionError> {
        if !self.registered_providers.get(provider) {
            return Err(SubscriptionError::Unauthorized(Unauthorized {}));
        }
        Ok(())
    }
    
    fn process_deposit(&mut self, user: Address, amount: U256) -> Result<(), SubscriptionError> {
        if amount.is_zero() {
            return Err(SubscriptionError::InvalidInput(InvalidInput {}));
        }
        
        // Update user balances
        let current_balance = self.user_escrow_balance.get(user);
        self.user_escrow_balance.insert(user, current_balance + amount);
        
        // Update contract TVL
        let total_locked = self.total_value_locked.get();
        self.total_value_locked.set(total_locked + amount);
        
        Ok(())
    }
}
