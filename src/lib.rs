//! Hybrid Escrow Recurring Payments Contract
//! 
//! Features:
//! - Subscription management
//! - Block-based payment intervals
//! - ETH escrow handling

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

extern crate alloc;
use alloc::{vec::Vec, vec};

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

sol_storage! {
    #[entrypoint]
    pub struct HybridEscrowContract {
        address owner;
        bool initialized;
        
        // Provider who receives payments
        address provider;
        
        // Subscription data
        mapping(address => uint256) subscription_amounts;
        mapping(address => uint256) subscription_intervals;
        mapping(address => uint256) last_payment_blocks;
        mapping(address => bool) active_subscriptions;
        
        // Escrow balances for each user
        mapping(address => uint256) escrow_balances;
        
        // Contract stats
        uint256 total_payments;
        uint256 last_processed_block;
        
        // Subscriber list
        address[] subscribers;
        mapping(address => uint256) subscriber_indices;
    }
}

#[public]
impl HybridEscrowContract {
    pub fn initialize(&mut self, provider_address: Address) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(b"Already initialized".to_vec());
        }
        
        self.owner.set(self.vm().msg_sender());
        self.provider.set(provider_address);
        self.initialized.set(true);
        self.total_payments.set(U256::from(0));
        self.last_processed_block.set(U256::from(self.vm().block_number()));
        
        Ok(())
    }
    
    pub fn owner(&self) -> Address {
        self.owner.get()
    }
    
    pub fn provider(&self) -> Address {
        self.provider.get()
    }
    
    pub fn current_block_number(&self) -> u64 {
        self.vm().block_number()
    }
    
    pub fn total_payments(&self) -> U256 {
        self.total_payments.get()
    }
    
    // Deposit funds to escrow for future subscription payments
    #[payable]
    pub fn deposit_to_escrow(&mut self) -> Result<(), Vec<u8>> {
        let user = self.vm().msg_sender();
        let amount = self.vm().msg_value();
        
        if amount == U256::ZERO {
            return Err(b"Must deposit some ETH".to_vec());
        }
        
        // Add to user's escrow balance
        let current_balance = self.escrow_balances.get(user);
        self.escrow_balances.setter(user).set(current_balance + amount);
        
        Ok(())
    }
    
    // Check user's escrow balance
    pub fn get_escrow_balance(&self, user: Address) -> U256 {
        self.escrow_balances.get(user)
    }
    
    // Create a subscription using escrowed funds
    pub fn create_subscription(&mut self, amount: U256, interval_blocks: U256) -> Result<(), Vec<u8>> {
        let subscriber = self.vm().msg_sender();
        let current_block = self.vm().block_number();
        
        if amount == U256::ZERO {
            return Err(b"Amount must be greater than 0".to_vec());
        }
        
        if interval_blocks == U256::ZERO {
            return Err(b"Interval must be greater than 0".to_vec());
        }
        
        // Check if user has enough escrowed funds for at least one payment
        let escrow_balance = self.escrow_balances.get(subscriber);
        if escrow_balance < amount {
            return Err(b"Insufficient escrow balance".to_vec());
        }
        
        // Add to subscriber list if not already present
        if !self.active_subscriptions.get(subscriber) {
            let index = self.subscribers.len();
            self.subscribers.push(subscriber);
            self.subscriber_indices.setter(subscriber).set(U256::from(index));
        }
        
        // Store subscription details
        self.subscription_amounts.setter(subscriber).set(amount);
        self.subscription_intervals.setter(subscriber).set(interval_blocks);
        self.last_payment_blocks.setter(subscriber).set(U256::from(current_block));
        self.active_subscriptions.setter(subscriber).set(true);
        
        Ok(())
    }
    
    // Process payment from escrow (called by user with ETH)
    #[payable]
    pub fn process_payment(&mut self, subscriber: Address) -> Result<(), Vec<u8>> {
        let current_block = self.vm().block_number();
        
        if !self.active_subscriptions.get(subscriber) {
            return Err(b"No active subscription".to_vec());
        }
        
        let amount = self.subscription_amounts.get(subscriber);
        let paid_amount = self.vm().msg_value();
        
        if paid_amount < amount {
            return Err(b"Insufficient payment".to_vec());
        }
        
        // Update payment tracking
        self.last_payment_blocks.setter(subscriber).set(U256::from(current_block));
        self.total_payments.set(self.total_payments.get() + amount);
        
        // Transfer to provider
        if let Err(_) = self.vm().transfer_eth(self.provider.get(), amount) {
            return Err(b"Transfer failed".to_vec());
        }
        
        Ok(())
    }
    
    // Process payment from escrow manually
    pub fn process_payment_from_escrow(&mut self, subscriber: Address) -> Result<(), Vec<u8>> {
        // Verify payment is actually due
        if !self.is_payment_due(subscriber) {
            return Err(b"Payment not due".to_vec());
        }
        
        let current_block = self.vm().block_number();
        let amount = self.subscription_amounts.get(subscriber);
        
        // Check if user has enough escrowed funds
        let escrow_balance = self.escrow_balances.get(subscriber);
        if escrow_balance < amount {
            return Err(b"Insufficient escrow balance".to_vec());
        }
        
        // Deduct from escrow balance
        self.escrow_balances.setter(subscriber).set(escrow_balance - amount);
        
        // Update payment tracking
        self.last_payment_blocks.setter(subscriber).set(U256::from(current_block));
        self.total_payments.set(self.total_payments.get() + amount);
        
        // Transfer to provider from contract balance
        if let Err(_) = self.vm().transfer_eth(self.provider.get(), amount) {
            return Err(b"Transfer to provider failed".to_vec());
        }
        
        Ok(())
    }
    
    // Get list of all subscribers (for external monitoring)
    pub fn get_all_subscribers(&self) -> Vec<Address> {
        let mut result = Vec::new();
        for i in 0..self.subscribers.len() {
            if let Some(subscriber) = self.subscribers.get(i) {
                if self.active_subscriptions.get(subscriber) {
                    result.push(subscriber);
                }
            }
        }
        result
    }
    
    // Get total number of active subscriptions
    pub fn get_active_subscription_count(&self) -> U256 {
        let mut count = 0u64;
        for i in 0..self.subscribers.len() {
            if let Some(subscriber) = self.subscribers.get(i) {
                if self.active_subscriptions.get(subscriber) {
                    count += 1;
                }
            }
        }
        U256::from(count)
    }
    
    // Check if payment is due
    pub fn is_payment_due(&self, subscriber: Address) -> bool {
        if !self.active_subscriptions.get(subscriber) {
            return false;
        }
        
        let interval = self.subscription_intervals.get(subscriber);
        let last_payment = self.last_payment_blocks.get(subscriber);
        let current_block = U256::from(self.vm().block_number());
        
        current_block >= last_payment + interval
    }
    
    // Get subscription info
    pub fn get_subscription(&self, subscriber: Address) -> (U256, U256, U256, bool) {
        (
            self.subscription_amounts.get(subscriber),
            self.subscription_intervals.get(subscriber),
            self.last_payment_blocks.get(subscriber),
            self.active_subscriptions.get(subscriber)
        )
    }
    
    // Cancel subscription
    pub fn cancel_subscription(&mut self) -> Result<(), Vec<u8>> {
        let subscriber = self.vm().msg_sender();
        
        if !self.active_subscriptions.get(subscriber) {
            return Err(b"No active subscription".to_vec());
        }
        
        self.active_subscriptions.setter(subscriber).set(false);
        
        // Note: We keep the subscriber in the list but mark as inactive
        // This preserves historical data while preventing future payments
        
        Ok(())
    }
    
    // Withdraw remaining escrow balance (for cancelled subscriptions)
    pub fn withdraw_escrow(&mut self) -> Result<(), Vec<u8>> {
        let user = self.vm().msg_sender();
        let balance = self.escrow_balances.get(user);
        
        if balance == U256::ZERO {
            return Err(b"No balance to withdraw".to_vec());
        }
        
        // Clear the balance first (CEI pattern)
        self.escrow_balances.setter(user).set(U256::ZERO);
        
        // Transfer the balance back to user
        if let Err(_) = self.vm().transfer_eth(user, balance) {
            // Revert the balance change if transfer fails
            self.escrow_balances.setter(user).set(balance);
            return Err(b"Transfer failed".to_vec());
        }
        
        Ok(())
    }
}
