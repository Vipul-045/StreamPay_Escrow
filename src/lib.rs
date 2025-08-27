//! Hybrid Escrow Recurring Payments Contract with Gelato Automation
//! 
//! Features:
//! - Subscription management
//! - Gelato keeper integration for auto-renewal
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
        
        // Subscription data
        mapping(address => uint256) subscription_amounts;
        mapping(address => uint256) subscription_intervals;
        mapping(address => uint256) last_payment_blocks;
        mapping(address => bool) active_subscriptions;
        
        // Gelato automation
        address gelato_automate;
        mapping(address => bytes32) gelato_task_ids;
        
        // Contract stats
        uint256 total_payments;
        uint256 last_processed_block;
        
        // Subscriber list for automation
        address[] subscribers;
        mapping(address => uint256) subscriber_indices;
    }
}

// Gelato Automate address on Arbitrum (simplified for demo)
const GELATO_AUTOMATE: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x2A, 0x6C, 0x10, 0x6a,
    0xe1, 0x3B, 0x55, 0x8B
]);

#[public]
impl HybridEscrowContract {
    pub fn initialize(&mut self) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(b"Already initialized".to_vec());
        }
        
        self.owner.set(self.vm().msg_sender());
        self.initialized.set(true);
        self.total_payments.set(U256::from(0));
        self.last_processed_block.set(U256::from(self.vm().block_number()));
        
        // Set Gelato Automate address
        self.gelato_automate.set(GELATO_AUTOMATE);
        
        Ok(())
    }
    
    pub fn owner(&self) -> Address {
        self.owner.get()
    }
    
    pub fn current_block_number(&self) -> u64 {
        self.vm().block_number()
    }
    
    pub fn total_payments(&self) -> U256 {
        self.total_payments.get()
    }
    
    // Create a subscription
    #[payable]
    pub fn create_subscription(&mut self, amount: U256, interval_blocks: U256) -> Result<(), Vec<u8>> {
        let subscriber = self.vm().msg_sender();
        let current_block = self.vm().block_number();
        
        if amount == U256::ZERO {
            return Err(b"Amount must be greater than 0".to_vec());
        }
        
        if interval_blocks == U256::ZERO {
            return Err(b"Interval must be greater than 0".to_vec());
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
    
    // Process payment for a subscription (called by Gelato or user)
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
        
        // Transfer to owner
        if let Err(_) = self.vm().transfer_eth(self.owner.get(), amount) {
            return Err(b"Transfer failed".to_vec());
        }
        
        Ok(())
    }
    
    // === GELATO AUTOMATION FUNCTIONS ===
    
    // Simple checker that just returns if any payment is due
    pub fn has_due_payments(&self) -> bool {
        let current_block = U256::from(self.vm().block_number());
        let subscriber_count = self.subscribers.len();
        
        for i in 0..subscriber_count {
            if let Some(subscriber) = self.subscribers.get(i) {
                if self.active_subscriptions.get(subscriber) {
                    let interval = self.subscription_intervals.get(subscriber);
                    let last_payment = self.last_payment_blocks.get(subscriber);
                    
                    if current_block >= last_payment + interval {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    // Check if any payments are due (for Gelato resolver)
    pub fn checker(&self) -> (bool, Vec<u8>) {
        let current_block = U256::from(self.vm().block_number());
        
        // Check all active subscriptions
        let subscriber_count = self.subscribers.len();
        for i in 0..subscriber_count {
            if let Some(subscriber) = self.subscribers.get(i) {
                if self.active_subscriptions.get(subscriber) {
                    let interval = self.subscription_intervals.get(subscriber);
                    let last_payment = self.last_payment_blocks.get(subscriber);
                    
                    if current_block >= last_payment + interval {
                        // Payment is due for this subscriber
                        let mut call_data = Vec::new();
                        call_data.extend_from_slice(&[0x8d, 0xa5, 0xcb, 0x5b]); // processPaymentAuto(address) selector
                        call_data.extend_from_slice(subscriber.as_slice());
                        return (true, call_data);
                    }
                }
            }
        }
        
        (false, Vec::new())
    }
    
    // Auto-process payment (called by Gelato)
    pub fn process_payment_auto(&mut self, subscriber: Address) -> Result<(), Vec<u8>> {
        // Verify payment is actually due
        if !self.is_payment_due(subscriber) {
            return Err(b"Payment not due".to_vec());
        }
        
        let current_block = self.vm().block_number();
        let amount = self.subscription_amounts.get(subscriber);
        
        // Update payment tracking (without requiring ETH payment for auto)
        self.last_payment_blocks.setter(subscriber).set(U256::from(current_block));
        self.total_payments.set(self.total_payments.get() + amount);
        
        // For true automation, this would pull from pre-approved allowance
        // or from escrowed funds. For now, we'll just update the tracking.
        
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
}
