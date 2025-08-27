#!/usr/bin/env node

/**
 * Top Up Subscription Script
 * 
 * This script allows users to add more funds to their subscription
 * to keep it running longer.
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
    
    // Top-up parameters
    subscriptionId: process.env.SUB_ID || process.argv[2],
    topUpAmount: process.env.TOPUP_AMOUNT || process.argv[3] || '10000000000000000', // 0.01 ETH default
};

function log(message) {
    console.log(`[TOPUP] ${new Date().toISOString()} - ${message}`);
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

async function topUpSubscription() {
    try {
        log('Starting subscription top-up...');
        
        if (!CONFIG.privateKey) {
            throw new Error('PRIVATE_KEY environment variable not set');
        }
        
        if (!CONFIG.subscriptionId) {
            throw new Error('Subscription ID required. Usage: node topup.js <subscription_id> [amount_in_wei]');
        }
        
        const subscriptionId = parseInt(CONFIG.subscriptionId);
        
        // Get user address
        const userAddress = execSync(`cast wallet address ${CONFIG.privateKey}`, { encoding: 'utf8' }).trim();
        log(`User: ${userAddress}`);
        log(`Subscription ID: ${subscriptionId}`);
        log(`Top-up amount: ${CONFIG.topUpAmount} wei (${formatEthAmount(CONFIG.topUpAmount)} ETH)`);
        
        // Get current subscription details
        log('Fetching current subscription details...');
        const currentDetails = getSubscriptionDetails(subscriptionId);
        
        if (!currentDetails) {
            throw new Error(`Subscription ${subscriptionId} not found`);
        }
        
        // Verify ownership
        if (currentDetails.user.toLowerCase() !== userAddress.toLowerCase()) {
            throw new Error(`You don't own subscription ${subscriptionId}`);
        }
        
        const currentStatus = getStatusName(currentDetails.status);
        const currentBalanceEth = formatEthAmount(currentDetails.balance);
        const amountPerIntervalEth = formatEthAmount(currentDetails.amountPerInterval);
        
        log(`Current status: ${currentStatus}`);
        log(`Current balance: ${currentBalanceEth} ETH`);
        log(`Amount per interval: ${amountPerIntervalEth} ETH`);
        
        // Check user balance
        const userBalance = execSync(`cast balance ${userAddress} --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
        const userBalanceEth = parseFloat(userBalance) / 1e18;
        log(`User balance: ${userBalanceEth} ETH`);
        
        if (parseFloat(userBalance) < parseFloat(CONFIG.topUpAmount)) {
            throw new Error(`Insufficient balance. Need ${formatEthAmount(CONFIG.topUpAmount)} ETH, have ${userBalanceEth} ETH`);
        }
        
        // Calculate how many intervals this will fund
        const additionalIntervals = Math.floor(parseFloat(CONFIG.topUpAmount) / parseFloat(currentDetails.amountPerInterval));
        log(`This top-up will fund ${additionalIntervals} additional intervals`);
        
        // Perform top-up
        log('Sending top-up transaction...');
        const txCommand = `cast send ${CONFIG.contractAddress} \\
            "topUp(uint256)" \\
            ${subscriptionId} \\
            --value ${CONFIG.topUpAmount} \\
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
            const expectedBalance = parseFloat(currentDetails.balance) + parseFloat(CONFIG.topUpAmount);
            const expectedBalanceEth = formatEthAmount(expectedBalance.toString());
            
            log(`Updated status: ${newStatus}`);
            log(`Updated balance: ${newBalanceEth} ETH (expected: ${expectedBalanceEth} ETH)`);
            
            // Calculate total intervals now funded
            const totalIntervalsNowFunded = Math.floor(parseFloat(updatedDetails.balance) / parseFloat(updatedDetails.amountPerInterval));
            log(`Total intervals now funded: ${totalIntervalsNowFunded}`);
            
            // If was past due and now has sufficient balance, should be reactivated
            if (currentDetails.status === 1 && updatedDetails.status === 0) { // PastDue -> Active
                log('🎉 Subscription reactivated from past due status!');
            }
        }
        
        // Save top-up record
        const topUpRecord = {
            subscriptionId,
            user: userAddress,
            topUpAmount: CONFIG.topUpAmount,
            topUpAmountEth: formatEthAmount(CONFIG.topUpAmount),
            transactionHash: txHash,
            topUpAt: new Date().toISOString(),
            beforeBalance: currentDetails.balance,
            afterBalance: updatedDetails ? updatedDetails.balance : 'Unknown',
            beforeStatus: currentStatus,
            afterStatus: updatedDetails ? getStatusName(updatedDetails.status) : 'Unknown',
        };
        
        const fileName = `topup_${subscriptionId}_${Date.now()}.json`;
        fs.writeFileSync(fileName, JSON.stringify(topUpRecord, null, 2));
        log(`Top-up record saved to ${fileName}`);
        
        log('🎉 Top-up completed successfully!');
        log(`💰 Added: ${formatEthAmount(CONFIG.topUpAmount)} ETH`);
        log(`📊 New balance: ${updatedDetails ? formatEthAmount(updatedDetails.balance) : 'Unknown'} ETH`);
        log(`⚡ Additional intervals funded: ${additionalIntervals}`);
        
    } catch (error) {
        log(`❌ Top-up failed: ${error.message}`);
        process.exit(1);
    }
}

// Run if script is executed directly
if (require.main === module) {
    topUpSubscription();
}

module.exports = { topUpSubscription };
