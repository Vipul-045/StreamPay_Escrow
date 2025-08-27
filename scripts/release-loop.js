#!/usr/bin/env node

/**
 * Release Payment Loop Script
 * 
 * This script continuously monitors subscriptions and releases payments
 * when they become due. Simulates automated payment processing.
 */

const { execSync } = require('child_process');
const fs = require('fs');

// Load deployment info
let DEPLOYMENT;
try {
    DEPLOYMENT = JSON.parse(fs.readFileSync('deployment.json', 'utf8'));
} catch (error) {
    console.error('❌ deployment.json not found. Run deploy.js first.');
    process.exit(1);
}

// Configuration
const CONFIG = {
    contractAddress: DEPLOYMENT.contractAddress,
    rpcUrl: DEPLOYMENT.rpcUrl,
    privateKey: process.env.PRIVATE_KEY,
    
    // Loop parameters
    checkInterval: parseInt(process.env.CHECK_INTERVAL || '30'), // Check every 30 seconds
    maxIterations: parseInt(process.env.MAX_ITERATIONS || '100'), // Limit for demo
    targetSubscriptionId: process.env.TARGET_SUB_ID || null, // Specific subscription to monitor
};

function log(message) {
    console.log(`[RELEASE-LOOP] ${new Date().toISOString()} - ${message}`);
}

function getSubscriptionDetails(subscriptionId) {
    try {
        const result = execSync(`cast call ${CONFIG.contractAddress} \\
            "getSubscription(uint256)" \\
            ${subscriptionId} \\
            --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
        
        // Parse the returned tuple
        // Returns: (user, provider, amount_per_interval, interval_seconds, duration_intervals, balance, paid_intervals, next_payment_ts, status, created_at)
        const parts = result.split('\\n');
        if (parts.length >= 10) {
            return {
                user: parts[0],
                provider: parts[1],
                amountPerInterval: parts[2],
                intervalSeconds: parseInt(parts[3], 16),
                durationIntervals: parseInt(parts[4], 16),
                balance: parts[5],
                paidIntervals: parseInt(parts[6], 16),
                nextPaymentTs: parseInt(parts[7], 16),
                status: parseInt(parts[8], 16),
                createdAt: parseInt(parts[9], 16),
            };
        }
        return null;
    } catch (error) {
        return null;
    }
}

function isPaymentDue(subscriptionId) {
    try {
        const result = execSync(`cast call ${CONFIG.contractAddress} \\
            "isPaymentDue(uint256)" \\
            ${subscriptionId} \\
            --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
        
        return result.toLowerCase().includes('true');
    } catch (error) {
        return false;
    }
}

async function releasePayment(subscriptionId) {
    try {
        log(`Releasing payment for subscription ${subscriptionId}...`);
        
        const txCommand = `cast send ${CONFIG.contractAddress} \\
            "releasePayment(uint256)" \\
            ${subscriptionId} \\
            --rpc-url ${CONFIG.rpcUrl} \\
            --private-key ${CONFIG.privateKey}`;
        
        const txOutput = execSync(txCommand, { encoding: 'utf8' });
        
        // Extract transaction hash
        const txHashMatch = txOutput.match(/transactionHash\\s+(0x[a-fA-F0-9]{64})/);
        const txHash = txHashMatch ? txHashMatch[1] : 'Unknown';
        
        log(`✅ Payment released! TX: ${txHash}`);
        return true;
        
    } catch (error) {
        log(`❌ Failed to release payment: ${error.message}`);
        return false;
    }
}

function getSubscriptionCount() {
    try {
        const result = execSync(`cast call ${CONFIG.contractAddress} \\
            "getSubscriptionCount()" \\
            --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
        
        return parseInt(result, 16);
    } catch (error) {
        return 0;
    }
}

function formatEthAmount(weiAmount) {
    return (parseFloat(weiAmount) / 1e18).toFixed(6);
}

function getStatusName(status) {
    const statusNames = ['Active', 'PastDue', 'CancelledByUser', 'CancelledByProvider', 'Expired'];
    return statusNames[status] || 'Unknown';
}

async function monitorAndRelease() {
    try {
        log('Starting payment release monitoring...');
        
        if (!CONFIG.privateKey) {
            throw new Error('PRIVATE_KEY environment variable not set');
        }
        
        const totalSubscriptions = getSubscriptionCount();
        log(`Total subscriptions: ${totalSubscriptions}`);
        
        if (totalSubscriptions === 0) {
            log('No subscriptions found. Create one first with create-subscription.js');
            return;
        }
        
        let iterations = 0;
        
        while (iterations < CONFIG.maxIterations) {
            iterations++;
            log(`\\n=== Iteration ${iterations}/${CONFIG.maxIterations} ===`);
            
            const subscriptionsToCheck = CONFIG.targetSubscriptionId 
                ? [parseInt(CONFIG.targetSubscriptionId)]
                : Array.from({length: totalSubscriptions}, (_, i) => i + 1);
            
            for (const subId of subscriptionsToCheck) {
                log(`\\nChecking subscription ${subId}...`);
                
                const details = getSubscriptionDetails(subId);
                if (!details) {
                    log(`Subscription ${subId} not found`);
                    continue;
                }
                
                const status = getStatusName(details.status);
                const balanceEth = formatEthAmount(details.balance);
                const amountEth = formatEthAmount(details.amountPerInterval);
                const currentTime = Math.floor(Date.now() / 1000);
                const timeUntilNext = details.nextPaymentTs - currentTime;
                
                log(`  Status: ${status}`);
                log(`  Balance: ${balanceEth} ETH`);
                log(`  Amount per interval: ${amountEth} ETH`);
                log(`  Paid intervals: ${details.paidIntervals}/${details.durationIntervals || '∞'}`);
                log(`  Time until next payment: ${timeUntilNext}s`);
                
                // Skip if not active
                if (details.status !== 0) { // 0 = Active
                    log(`  ⏩ Skipping (status: ${status})`);
                    continue;
                }
                
                // Check if payment is due
                if (isPaymentDue(subId)) {
                    log(`  💰 Payment is due!`);
                    
                    const success = await releasePayment(subId);
                    if (success) {
                        // Wait a moment and check updated details
                        await new Promise(resolve => setTimeout(resolve, 3000));
                        
                        const updatedDetails = getSubscriptionDetails(subId);
                        if (updatedDetails) {
                            const newBalanceEth = formatEthAmount(updatedDetails.balance);
                            const newStatus = getStatusName(updatedDetails.status);
                            log(`  📊 Updated - Balance: ${newBalanceEth} ETH, Status: ${newStatus}, Paid: ${updatedDetails.paidIntervals}`);
                        }
                    }
                } else {
                    log(`  ⏰ Payment not yet due`);
                }
            }
            
            if (iterations < CONFIG.maxIterations) {
                log(`\\n⏳ Waiting ${CONFIG.checkInterval} seconds until next check...`);
                await new Promise(resolve => setTimeout(resolve, CONFIG.checkInterval * 1000));
            }
        }
        
        log('\\n🏁 Monitoring completed!');
        
    } catch (error) {
        log(`❌ Monitoring failed: ${error.message}`);
        process.exit(1);
    }
}

// Run if script is executed directly
if (require.main === module) {
    monitorAndRelease();
}

module.exports = { monitorAndRelease, releasePayment, getSubscriptionDetails };
