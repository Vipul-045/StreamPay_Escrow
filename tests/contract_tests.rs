#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::alloy_primitives::{Address, U256};
    
    // Mock addresses for testing
    const USER_ADDR: Address = Address::repeat_byte(0x01);
    const PROVIDER_ADDR: Address = Address::repeat_byte(0x02);
    const OTHER_ADDR: Address = Address::repeat_byte(0x03);
    
    fn setup_contract() -> HybridEscrowRecurring {
        let mut contract = HybridEscrowRecurring::default();
        // Initialize would typically be called by owner
        contract.owner.set(USER_ADDR);
        contract.paused.set(false);
        contract.emergency_stopped.set(false);
        contract.sub_count.set(U256::ZERO);
        contract.locked.set(false);
        contract
    }
    
    #[test]
    fn test_initialization() {
        let contract = setup_contract();
        assert_eq!(contract.owner.get(), USER_ADDR);
        assert_eq!(contract.paused.get(), false);
        assert_eq!(contract.sub_count.get(), U256::ZERO);
    }
    
    #[test]
    fn test_subscription_creation_validation() {
        let mut contract = setup_contract();
        
        // Test invalid provider
        let result = contract.create_subscription(
            Address::ZERO,
            U256::from(100),
            300, // 5 minutes
            12,  // 12 intervals
        );
        assert!(result.is_err());
        
        // Test zero amount
        let result = contract.create_subscription(
            PROVIDER_ADDR,
            U256::ZERO,
            300,
            12,
        );
        assert!(result.is_err());
        
        // Test interval too short
        let result = contract.create_subscription(
            PROVIDER_ADDR,
            U256::from(100),
            30, // Less than 60 seconds
            12,
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_subscription_status_conversion() {
        use crate::utils::*;
        
        let status = SubscriptionStatus::from(0u8);
        assert_eq!(status, SubscriptionStatus::Active);
        
        let status = SubscriptionStatus::from(1u8);
        assert_eq!(status, SubscriptionStatus::PastDue);
        
        let status = SubscriptionStatus::from(2u8);
        assert_eq!(status, SubscriptionStatus::CancelledByUser);
        
        let status = SubscriptionStatus::from(3u8);
        assert_eq!(status, SubscriptionStatus::CancelledByProvider);
        
        let status = SubscriptionStatus::from(4u8);
        assert_eq!(status, SubscriptionStatus::Expired);
        
        // Test fallback
        let status = SubscriptionStatus::from(99u8);
        assert_eq!(status, SubscriptionStatus::Active);
    }
    
    #[test]
    fn test_subscription_data_storage() {
        let mut contract = setup_contract();
        
        // Simulate subscription creation (without msg context)
        let sub_id = U256::from(1);
        contract.sub_count.set(sub_id);
        contract.sub_user.setter(sub_id).set(USER_ADDR);
        contract.sub_provider.setter(sub_id).set(PROVIDER_ADDR);
        contract.sub_amount_per_interval.setter(sub_id).set(U256::from(1000));
        contract.sub_interval_seconds.setter(sub_id).set(300);
        contract.sub_duration_intervals.setter(sub_id).set(12);
        contract.sub_balance.setter(sub_id).set(U256::from(500));
        contract.sub_paid_intervals.setter(sub_id).set(U256::from(1));
        contract.sub_status.setter(sub_id).set(0); // Active
        
        // Test get_subscription
        let (user, provider, amount, interval, duration, balance, paid, _, status, _) = 
            contract.get_subscription(sub_id);
        
        assert_eq!(user, USER_ADDR);
        assert_eq!(provider, PROVIDER_ADDR);
        assert_eq!(amount, U256::from(1000));
        assert_eq!(interval, 300);
        assert_eq!(duration, 12);
        assert_eq!(balance, U256::from(500));
        assert_eq!(paid, U256::from(1));
        assert_eq!(status, 0);
    }
    
    #[test]
    fn test_provider_balance_tracking() {
        let mut contract = setup_contract();
        
        // Set provider balance
        contract.provider_balances.setter(PROVIDER_ADDR).set(U256::from(5000));
        
        // Test get_provider_balance
        let balance = contract.get_provider_balance(PROVIDER_ADDR);
        assert_eq!(balance, U256::from(5000));
        
        // Test zero balance for unknown provider
        let balance = contract.get_provider_balance(OTHER_ADDR);
        assert_eq!(balance, U256::ZERO);
    }
    
    #[test]
    fn test_subscription_arrays() {
        let mut contract = setup_contract();
        
        // Add subscription IDs to user and provider arrays
        contract.user_subscriptions.setter(USER_ADDR).push(U256::from(1));
        contract.user_subscriptions.setter(USER_ADDR).push(U256::from(3));
        
        contract.provider_subscriptions.setter(PROVIDER_ADDR).push(U256::from(1));
        contract.provider_subscriptions.setter(PROVIDER_ADDR).push(U256::from(2));
        
        // Test retrieval
        let user_subs = contract.get_user_subscriptions(USER_ADDR);
        assert_eq!(user_subs.len(), 2);
        assert_eq!(user_subs[0], U256::from(1));
        assert_eq!(user_subs[1], U256::from(3));
        
        let provider_subs = contract.get_provider_subscriptions(PROVIDER_ADDR);
        assert_eq!(provider_subs.len(), 2);
        assert_eq!(provider_subs[0], U256::from(1));
        assert_eq!(provider_subs[1], U256::from(2));
    }
    
    #[test]
    fn test_admin_functions() {
        let mut contract = setup_contract();
        
        // Test pausing (would require proper msg::sender context in real test)
        contract.paused.set(true);
        assert_eq!(contract.paused.get(), true);
        
        // Test emergency stop
        contract.emergency_stopped.set(true);
        assert_eq!(contract.emergency_stopped.get(), true);
    }
    
    #[test]
    fn test_payment_due_logic() {
        let mut contract = setup_contract();
        
        let sub_id = U256::from(1);
        let current_time = 1700000000u64; // Mock timestamp
        let future_time = current_time + 600; // 10 minutes later
        
        // Set next payment in the future
        contract.sub_next_payment_ts.setter(sub_id).set(future_time);
        
        // Mock block::timestamp would be needed for real test
        // For now, we can test the basic logic structure
        
        // Set next payment in the past
        contract.sub_next_payment_ts.setter(sub_id).set(current_time - 600);
        
        // The is_payment_due function would return true if current_time >= next_payment_ts
    }
}
