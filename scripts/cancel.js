#!/usr/bin/env node

/**
 * Cancel Subscription Script
 * 
 * This script allows users or providers to cancel a subscription
 * and refund any remaining balance.
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
    
    // Cancellation parameters
    subscriptionId: process.env.SUB_ID || process.argv[2],
};

function log(message) {
    console.log(`[CANCEL] ${new Date().toISOString()} - ${message}`);
}

function getSubscriptionDetails(subscriptionId) {
    try {
        const result = execSync(`cast call ${CONFIG.contractAddress} \\
            "getSubscription(uint256)" \\
            ${subscriptionId} \\
            --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
        
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

function formatEthAmount(weiAmount) {
    return (parseFloat(weiAmount) / 1e18).toFixed(6);
}

function getStatusName(status) {
    const statusNames = ['Active', 'PastDue', 'CancelledByUser', 'CancelledByProvider', 'Expired'];
    return statusNames[status] || 'Unknown';
}

async function cancelSubscription() {
    try {
        log('Starting subscription cancellation...');
        
        if (!CONFIG.privateKey) {
            throw new Error('PRIVATE_KEY environment variable not set');
        }
        
        if (!CONFIG.subscriptionId) {
            throw new Error('Subscription ID required. Usage: node cancel.js <subscription_id>');
        }
        
        const subscriptionId = parseInt(CONFIG.subscriptionId);
        
        // Get caller address
        const callerAddress = execSync(`cast wallet address ${CONFIG.privateKey}`, { encoding: 'utf8' }).trim();
        log(`Caller: ${callerAddress}`);
        log(`Subscription ID: ${subscriptionId}`);
        
        // Get current subscription details
        log('Fetching subscription details...');
        const details = getSubscriptionDetails(subscriptionId);
        
        if (!details) {
            throw new Error(`Subscription ${subscriptionId} not found`);
        }
        
        const status = getStatusName(details.status);
        const balanceEth = formatEthAmount(details.balance);
        const amountPerIntervalEth = formatEthAmount(details.amountPerInterval);
        
        log(`User: ${details.user}`);
        log(`Provider: ${details.provider}`);
        log(`Current status: ${status}`);
        log(`Current balance: ${balanceEth} ETH`);
        log(`Amount per interval: ${amountPerIntervalEth} ETH`);
        log(`Paid intervals: ${details.paidIntervals}/${details.durationIntervals || '∞'}`);
        
        // Check authorization
        const isUser = callerAddress.toLowerCase() === details.user.toLowerCase();
        const isProvider = callerAddress.toLowerCase() === details.provider.toLowerCase();
        
        if (!isUser && !isProvider) {
            throw new Error(`Not authorized to cancel. You must be either the user (${details.user}) or provider (${details.provider})`);
        }
        
        const callerRole = isUser ? 'User' : 'Provider';
        log(`Caller role: ${callerRole}`);
        
        // Check if already cancelled
        if (details.status === 2 || details.status === 3) { // CancelledByUser or CancelledByProvider
            throw new Error(`Subscription already cancelled (status: ${status})`);
        }
        
        // Show what will happen
        const refundAmount = parseFloat(details.balance);
        const refundEth = formatEthAmount(details.balance);
        
        if (refundAmount > 0) {
            log(`💰 ${refundEth} ETH will be refunded to user: ${details.user}`);
        } else {
            log(`💰 No refund (balance is 0)`);
        }
        
        // Confirm cancellation
        log('⚠️  This action cannot be undone!');
        
        // In a real app, you'd want user confirmation here
        // For demo purposes, we'll proceed automatically
        
        // Perform cancellation
        log('Sending cancellation transaction...');
        const txCommand = `cast send ${CONFIG.contractAddress} \\
            "cancelSubscription(uint256)" \\
            ${subscriptionId} \\
            --rpc-url ${CONFIG.rpcUrl} \\
            --private-key ${CONFIG.privateKey}`;
        
        const txOutput = execSync(txCommand, { encoding: 'utf8' });
        
        // Extract transaction hash
        const txHashMatch = txOutput.match(/transactionHash\\s+(0x[a-fA-F0-9]{64})/);
        const txHash = txHashMatch ? txHashMatch[1] : 'Unknown';
        
        log(`Transaction sent! TX: ${txHash}`);
        
        // Wait for transaction to be mined
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        // Get updated subscription details
        log('Fetching updated subscription details...');
        const updatedDetails = getSubscriptionDetails(subscriptionId);
        
        if (updatedDetails) {
            const newStatus = getStatusName(updatedDetails.status);
            const newBalanceEth = formatEthAmount(updatedDetails.balance);
            
            log(`Updated status: ${newStatus}`);
            log(`Updated balance: ${newBalanceEth} ETH`);
            
            // Verify cancellation was successful
            const expectedStatus = isUser ? 'CancelledByUser' : 'CancelledByProvider';
            if (newStatus === expectedStatus) {
                log('✅ Cancellation confirmed!');
            } else {
                log(`⚠️  Unexpected status: ${newStatus} (expected: ${expectedStatus})`);
            }
            
            // Verify balance was cleared
            if (parseFloat(updatedDetails.balance) === 0) {
                log('✅ Balance refunded successfully!');
            } else {
                log(`⚠️  Balance not fully refunded: ${newBalanceEth} ETH remaining`);
            }
        }
        
        // Check user balance to confirm refund
        if (refundAmount > 0) {
            log('Checking user balance for refund confirmation...');
            const userBalance = execSync(`cast balance ${details.user} --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
            const userBalanceEth = parseFloat(userBalance) / 1e18;
            log(`User balance: ${userBalanceEth} ETH`);
        }
        
        // Save cancellation record
        const cancellationRecord = {
            subscriptionId,
            cancelledBy: callerAddress,
            cancelledByRole: callerRole,
            refundAmount: details.balance,
            refundAmountEth: refundEth,
            transactionHash: txHash,
            cancelledAt: new Date().toISOString(),
            beforeStatus: status,
            afterStatus: updatedDetails ? getStatusName(updatedDetails.status) : 'Unknown',
            user: details.user,
            provider: details.provider,
            totalPaidIntervals: details.paidIntervals,
        };
        
        const fileName = `cancellation_${subscriptionId}_${Date.now()}.json`;
        fs.writeFileSync(fileName, JSON.stringify(cancellationRecord, null, 2));
        log(`Cancellation record saved to ${fileName}`);
        
        log('🎉 Subscription cancelled successfully!');
        log(`❌ Status: ${expectedStatus}`);
        log(`💰 Refunded: ${refundEth} ETH to ${details.user}`);
        log(`📊 Total intervals paid: ${details.paidIntervals}`);
        
    } catch (error) {
        log(`❌ Cancellation failed: ${error.message}`);
        process.exit(1);
    }
}

// Run if script is executed directly
if (require.main === module) {
    cancelSubscription();
}

module.exports = { cancelSubscription };
