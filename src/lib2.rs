// ENHANCED MINIMAL SUBSCRIPTION ESCROW FOR ARBITRUM STYLUS
// Core functionality optimized for 24KB compressed size limit
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

extern crate alloc;
use alloc::{string::String, vec, vec::Vec};

use stylus_sdk::{
    alloy_primitives::{Address, U256}, 
    prelude::*,
    storage::*,
    msg, block, call
};

// =============================================================================
// STORAGE STRUCTURE
// =============================================================================

#[storage]
#[entrypoint]
pub struct SubscriptionEscrow {
    // Core admin
    admin: StorageAddress,
    paused: StorageBool,
    
    // Counters
    next_plan_id: StorageU256,
    next_subscription_id: StorageU256,
    
    // Providers
    providers: StorageMap<Address, StorageBool>,
    provider_names: StorageMap<Address, StorageString>,
    
    // Plans
    plan_provider: StorageMap<U256, StorageAddress>,
    plan_price: StorageMap<U256, StorageU256>,
    plan_interval: StorageMap<U256, StorageU256>,
    plan_active: StorageMap<U256, StorageBool>,
    
    // Subscriptions
    subscription_plan_id: StorageMap<U256, StorageU256>,
    subscription_subscriber: StorageMap<U256, StorageAddress>,
    subscription_last_payment: StorageMap<U256, StorageU256>,
    subscription_active: StorageMap<U256, StorageBool>,
    subscription_suspended: StorageMap<U256, StorageBool>,
    user_subscriptions: StorageMap<Address, StorageVec<StorageU256>>,
    
    // Balances
    user_escrow_balance: StorageMap<Address, StorageU256>,
    provider_earnings: StorageMap<Address, StorageU256>,
    
    // Stats
    total_value_locked: StorageU256,
}

#[public]
impl SubscriptionEscrow {
    
    /// Initialize contract
    pub fn initialize(&mut self) -> bool {
        if self.admin.get() != Address::ZERO {
            return false;
        }
        
        let caller = msg::sender();
        self.admin.set(caller);
        self.next_plan_id.set(U256::from(1));
        self.next_subscription_id.set(U256::from(1));
        true
    }
    
    /// Register as provider (enhanced with validation)
    pub fn register_provider(&mut self, name: String) -> bool {
        if self.paused.get() || name.is_empty() || name.len() > 100 {
            return false;
        }
        
        let caller = msg::sender();
        
        // Check if already registered
        if self.providers.get(caller) {
            return false; // Already registered
        }
        
        self.providers.insert(caller, true);
        let mut provider_name = self.provider_names.setter(caller);
        provider_name.set_str(&name);
        
        true
    }
    
    /// Unregister as provider (prevents new plans, existing ones continue)
    pub fn unregister_provider(&mut self) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = msg::sender();
        
        if !self.providers.get(caller) {
            return false; // Not registered
        }
        
        self.providers.insert(caller, false);
        
        true
    }
    
    /// Create subscription plan
    pub fn create_plan(&mut self, price: U256, interval: U256) -> U256 {
        if self.paused.get() || price == U256::ZERO || interval == U256::ZERO {
            return U256::ZERO;
        }
        
        let caller = msg::sender();
        if !self.providers.get(caller) {
            return U256::ZERO;
        }
        
        let plan_id = self.next_plan_id.get();
        self.next_plan_id.set(plan_id + U256::from(1));
        
        self.plan_provider.insert(plan_id, caller);
        self.plan_price.insert(plan_id, price);
        self.plan_interval.insert(plan_id, interval);
        self.plan_active.insert(plan_id, true);
        
        plan_id
    }
    
    /// Update plan status (activate/deactivate)
    pub fn update_plan_status(&mut self, plan_id: U256, active: bool) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = msg::sender();
        let plan_provider = self.plan_provider.get(plan_id);
        
        // Only the plan's provider can update it
        if plan_provider != caller || plan_provider == Address::ZERO {
            return false;
        }
        
        self.plan_active.insert(plan_id, active);
        true
    }
    
    /// Subscribe to plan
    #[payable]
    pub fn subscribe(&mut self, plan_id: U256) -> U256 {
        if self.paused.get() {
            return U256::ZERO;
        }
        
        let caller = msg::sender();
        let payment = msg::value();
        
        // Check plan exists and is active
        let plan_provider = self.plan_provider.get(plan_id);
        if plan_provider == Address::ZERO || !self.plan_active.get(plan_id) {
            return U256::ZERO;
        }
        
        let plan_price = self.plan_price.get(plan_id);
        
        // Add payment to escrow
        if payment > U256::ZERO {
            let current_balance = self.user_escrow_balance.get(caller);
            self.user_escrow_balance.insert(caller, current_balance + payment);
            
            let total_locked = self.total_value_locked.get();
            self.total_value_locked.set(total_locked + payment);
        }
        
        // Check sufficient balance
        let user_balance = self.user_escrow_balance.get(caller);
        if user_balance < plan_price {
            return U256::ZERO;
        }
        
        let subscription_id = self.next_subscription_id.get();
        self.next_subscription_id.set(subscription_id + U256::from(1));
        
        // Create subscription
        self.subscription_plan_id.insert(subscription_id, plan_id);
        self.subscription_subscriber.insert(subscription_id, caller);
        self.subscription_last_payment.insert(subscription_id, U256::from(block::timestamp()));
        self.subscription_active.insert(subscription_id, true);
        self.subscription_suspended.insert(subscription_id, false);
        
        // Add to user's subscription list
        let mut user_subs = self.user_subscriptions.setter(caller);
        user_subs.push(subscription_id);
        
        // Process payment
        self.user_escrow_balance.insert(caller, user_balance - plan_price);
        
        let provider_earnings = self.provider_earnings.get(plan_provider);
        self.provider_earnings.insert(plan_provider, provider_earnings + plan_price);
        
        subscription_id
    }
    
    /// Deposit funds
    #[payable]
    pub fn deposit_funds(&mut self) {
        let caller = msg::sender();
        let amount = msg::value();
        
        if amount > U256::ZERO {
            let current_balance = self.user_escrow_balance.get(caller);
            self.user_escrow_balance.insert(caller, current_balance + amount);
            
            let total_locked = self.total_value_locked.get();
            self.total_value_locked.set(total_locked + amount);
        }
    }
    
    /// Withdraw funds
    pub fn withdraw_funds(&mut self, amount: U256) -> bool {
        if self.paused.get() || amount == U256::ZERO {
            return false;
        }
        
        let caller = msg::sender();
        let current_balance = self.user_escrow_balance.get(caller);
        
        if amount > current_balance {
            return false;
        }
        
        self.user_escrow_balance.insert(caller, current_balance - amount);
        
        let total_locked = self.total_value_locked.get();
        if total_locked >= amount {
            self.total_value_locked.set(total_locked - amount);
        }
        
        match call::transfer_eth(caller, amount) {
            Ok(_) => true,
            Err(_) => {
                // Revert balance change
                self.user_escrow_balance.insert(caller, current_balance);
                let total_locked = self.total_value_locked.get();
                self.total_value_locked.set(total_locked + amount);
                false
            }
        }
    }
    
    /// Withdraw earnings (providers)
    pub fn withdraw_earnings(&mut self, amount: U256) -> bool {
        if self.paused.get() || amount == U256::ZERO {
            return false;
        }
        
        let caller = msg::sender();
        let available_earnings = self.provider_earnings.get(caller);
        
        if amount > available_earnings {
            return false;
        }
        
        self.provider_earnings.insert(caller, available_earnings - amount);
        
        match call::transfer_eth(caller, amount) {
            Ok(_) => true,
            Err(_) => {
                self.provider_earnings.insert(caller, available_earnings);
                false
            }
        }
    }
    
    /// Cancel subscription (manual cancellation by subscriber)
    pub fn cancel_subscription(&mut self, subscription_id: U256) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = msg::sender();
        
        // Check subscription exists and belongs to caller
        let subscriber = self.subscription_subscriber.get(subscription_id);
        if subscriber != caller || subscriber == Address::ZERO {
            return false;
        }
        
        let is_active = self.subscription_active.get(subscription_id);
        if !is_active {
            return false;
        }
        
        // Deactivate subscription
        self.subscription_active.insert(subscription_id, false);
        
        true
    }
    
    /// Process subscription payment (for automation/manual triggering)
    pub fn process_subscription_payment(&mut self, subscription_id: U256) -> bool {
        if self.paused.get() {
            return false;
        }
        
        // Get subscription details
        let subscriber = self.subscription_subscriber.get(subscription_id);
        let plan_id = self.subscription_plan_id.get(subscription_id);
        let is_active = self.subscription_active.get(subscription_id);
        let is_suspended = self.subscription_suspended.get(subscription_id);
        
        if subscriber == Address::ZERO || !is_active {
            return false;
        }
        
        // Get plan details
        let plan_provider = self.plan_provider.get(plan_id);
        let plan_price = self.plan_price.get(plan_id);
        let plan_interval = self.plan_interval.get(plan_id);
        
        if plan_provider == Address::ZERO {
            return false;
        }
        
        // Check if payment is due (time-based)
        let last_payment = self.subscription_last_payment.get(subscription_id);
        let current_time = U256::from(block::timestamp());
        
        if current_time < last_payment + plan_interval {
            return false; // Payment not due yet
        }
        
        // Check user balance
        let user_balance = self.user_escrow_balance.get(subscriber);
        
        if user_balance < plan_price {
            // Insufficient balance - suspend subscription
            if !is_suspended {
                self.subscription_suspended.insert(subscription_id, true);
            }
            return false;
        }
        
        // Process payment
        self.user_escrow_balance.insert(subscriber, user_balance - plan_price);
        
        let provider_earnings = self.provider_earnings.get(plan_provider);
        self.provider_earnings.insert(plan_provider, provider_earnings + plan_price);
        
        // Update last payment time
        self.subscription_last_payment.insert(subscription_id, current_time);
        
        // Reactivate if was suspended
        if is_suspended {
            self.subscription_suspended.insert(subscription_id, false);
        }
        
        true
    }
    
    /// Check if payment is due for a subscription
    pub fn is_payment_due(&self, subscription_id: U256) -> bool {
        let is_active = self.subscription_active.get(subscription_id);
        if !is_active {
            return false;
        }
        
        let plan_id = self.subscription_plan_id.get(subscription_id);
        let plan_interval = self.plan_interval.get(plan_id);
        let last_payment = self.subscription_last_payment.get(subscription_id);
        let current_time = U256::from(block::timestamp());
        
        current_time >= last_payment + plan_interval
    }
    
    /// Reactivate suspended subscription when user adds funds
    pub fn reactivate_subscription(&mut self, subscription_id: U256) -> bool {
        if self.paused.get() {
            return false;
        }
        
        let caller = msg::sender();
        let subscriber = self.subscription_subscriber.get(subscription_id);
        
        if subscriber != caller {
            return false;
        }
        
        let is_active = self.subscription_active.get(subscription_id);
        let is_suspended = self.subscription_suspended.get(subscription_id);
        
        if !is_active || !is_suspended {
            return false;
        }
        
        let plan_id = self.subscription_plan_id.get(subscription_id);
        let plan_price = self.plan_price.get(plan_id);
        let user_balance = self.user_escrow_balance.get(caller);
        
        if user_balance >= plan_price {
            self.subscription_suspended.insert(subscription_id, false);
            return true;
        }
        
        false
    }
    
    /// Emergency pause (admin only)
    pub fn emergency_pause(&mut self) -> bool {
        let caller = msg::sender();
        if caller != self.admin.get() {
            return false;
        }
        
        let current_state = self.paused.get();
        self.paused.set(!current_state);
        true
    }
    
    // =============================================================================
    // VIEW FUNCTIONS
    // =============================================================================
    
    /// Get plan info
    pub fn get_plan_info(&self, plan_id: U256) -> (Address, U256, U256, bool) {
        let provider = self.plan_provider.get(plan_id);
        let price = self.plan_price.get(plan_id);
        let interval = self.plan_interval.get(plan_id);
        let active = self.plan_active.get(plan_id);
        
        (provider, price, interval, active)
    }
    
    /// Get subscription info
    pub fn get_subscription_info(&self, subscription_id: U256) -> (U256, Address, U256, bool, bool) {
        let plan_id = self.subscription_plan_id.get(subscription_id);
        let subscriber = self.subscription_subscriber.get(subscription_id);
        let last_payment = self.subscription_last_payment.get(subscription_id);
        let active = self.subscription_active.get(subscription_id);
        let suspended = self.subscription_suspended.get(subscription_id);
        
        (plan_id, subscriber, last_payment, active, suspended)
    }
    
    /// Get user's subscription count
    pub fn get_user_subscription_count(&self, user: Address) -> U256 {
        let user_subs = self.user_subscriptions.get(user);
        let mut active_count = 0u32;
        
        for i in 0..user_subs.len() {
            if let Some(sub_guard) = user_subs.get(i) {
                let sub_id = sub_guard.get();
                if self.subscription_active.get(sub_id) {
                    active_count += 1;
                }
            }
        }
        
        U256::from(active_count)
    }
    
    /// Get user balance
    pub fn get_user_balance(&self, user: Address) -> U256 {
        self.user_escrow_balance.get(user)
    }
    
    /// Get provider earnings
    pub fn get_provider_earnings(&self, provider: Address) -> U256 {
        self.provider_earnings.get(provider)
    }
    
    /// Get provider info
    pub fn get_provider_info(&self, provider: Address) -> (bool, String) {
        let is_provider = self.providers.get(provider);
        let name = self.provider_names.get(provider).get_string();
        
        (is_provider, name)
    }
    
    /// Get contract stats
    pub fn get_stats(&self) -> (U256, U256, U256) {
        let total_locked = self.total_value_locked.get();
        let next_plan = self.next_plan_id.get();
        let next_sub = self.next_subscription_id.get();
        
        (total_locked, next_plan, next_sub)
    }
    
    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused.get()
    }
    
    /// Get admin
    pub fn get_admin(&self) -> Address {
        self.admin.get()
    }
    
    /// Check if address is registered provider
    pub fn is_provider(&self, provider: Address) -> bool {
        self.providers.get(provider)
    }
    
    /// Get total number of plans
    pub fn get_total_plans(&self) -> U256 {
        let next_plan = self.next_plan_id.get();
        if next_plan > U256::ZERO {
            next_plan - U256::from(1)
        } else {
            U256::ZERO
        }
    }
    
    /// Get total number of subscriptions
    pub fn get_total_subscriptions(&self) -> U256 {
        let next_sub = self.next_subscription_id.get();
        if next_sub > U256::ZERO {
            next_sub - U256::from(1)
        } else {
            U256::ZERO
        }
    }
}
