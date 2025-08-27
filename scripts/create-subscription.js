#!/usr/bin/env node

/**
 * Create Subscription Demo Script
 * 
 * This script demonstrates creating a subscription with ETH deposit
 * for the hybrid escrow recurring payments system.
 */

const { execSync } = require('child_process');
const fs = require('fs');
require('dotenv').config();

// Load deployment info
let DEPLOYMENT;
try {
    DEPLOYMENT = JSON.parse(fs.readFileSync('deployment.json', 'utf8'));
} catch (error) {
    console.error('❌ deployment.json not found. Run deploy.js first.');
    process.exit(1);
}

// Configuration from environment variables
const CONFIG = {
    privateKey: process.env.PRIVATE_KEY,
    rpcUrl: process.env.RPC_URL || 'https://sepolia-rollup.arbitrum.io/rpc',
    contractAddress: process.env.CONTRACT_ADDRESS || '0x15bb9a22a98790f1f6085a5a441b15a9dd86fd88',
    providerAddress: process.env.PUBLIC_KEY || '0xd27e6Bf5bF3AdbBE1d85c1c0537F1Dd5302A7E76',
    amountPerInterval: '5000000000000000', // 0.005 ETH in wei
    intervalSeconds: '300', // 5 minutes
    durationIntervals: '12', // 1 hour total (12 x 5 minutes)
    depositAmount: '60000000000000000' // 0.06 ETH in wei (12 intervals)
};

function log(message) {
    console.log(`[CREATE-SUB] ${new Date().toISOString()} - ${message}`);
}

async function createSubscription() {
    try {
        log('Creating subscription...');
        
        if (!CONFIG.privateKey) {
            throw new Error('PRIVATE_KEY environment variable not set');
        }
        
        // Get user address
        const userAddress = execSync(`cast wallet address ${CONFIG.privateKey}`, { encoding: 'utf8' }).trim();
        log(`User: ${userAddress}`);
        log(`Provider: ${CONFIG.providerAddress}`);
        log(`Amount per interval: ${CONFIG.amountPerInterval} wei (${parseFloat(CONFIG.amountPerInterval) / 1e18} ETH)`);
        log(`Interval: ${CONFIG.intervalSeconds} seconds`);
        log(`Duration: ${CONFIG.durationIntervals} intervals`);
        log(`Deposit: ${CONFIG.depositAmount} wei (${parseFloat(CONFIG.depositAmount) / 1e18} ETH)`);
        
        // Check user balance
        const balance = execSync(`cast balance ${userAddress} --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
        const balanceEth = parseFloat(balance) / 1e18;
        log(`User balance: ${balanceEth} ETH`);
        
        if (parseFloat(balance) < parseFloat(CONFIG.depositAmount)) {
            throw new Error(`Insufficient balance. Need ${parseFloat(CONFIG.depositAmount) / 1e18} ETH, have ${balanceEth} ETH`);
        }
        
        // Create subscription
        log('Sending transaction...');
        const txCommand = `cast send ${CONFIG.contractAddress} \\
            "createSubscription(address,uint256,uint64,uint64)" \\
            ${CONFIG.providerAddress} \\
            ${CONFIG.amountPerInterval} \\
            ${CONFIG.intervalSeconds} \\
            ${CONFIG.durationIntervals} \\
            --value ${CONFIG.depositAmount} \\
            --rpc-url ${CONFIG.rpcUrl} \\
            --private-key ${CONFIG.privateKey}`;
        
        const txOutput = execSync(txCommand, { encoding: 'utf8' });
        log('Transaction sent!');
        
        // Extract transaction hash
        const txHashMatch = txOutput.match(/transactionHash\\s+(0x[a-fA-F0-9]{64})/);
        const txHash = txHashMatch ? txHashMatch[1] : 'Unknown';
        
        log(`Transaction hash: ${txHash}`);
        
        // Wait a moment for transaction to be mined
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        // Get subscription count to determine the subscription ID
        log('Getting subscription ID...');
        const subCount = execSync(`cast call ${CONFIG.contractAddress} \\
            "getSubscriptionCount()" \\
            --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
        
        const subscriptionId = parseInt(subCount, 16);
        log(`Subscription ID: ${subscriptionId}`);
        
        // Get subscription details
        log('Fetching subscription details...');
        const subDetails = execSync(`cast call ${CONFIG.contractAddress} \\
            "getSubscription(uint256)" \\
            ${subscriptionId} \\
            --rpc-url ${CONFIG.rpcUrl}`, { encoding: 'utf8' }).trim();
        
        log('Subscription details:', subDetails);
        
        // Save subscription info
        const subscriptionInfo = {
            subscriptionId,
            user: userAddress,
            provider: CONFIG.providerAddress,
            amountPerInterval: CONFIG.amountPerInterval,
            intervalSeconds: CONFIG.intervalSeconds,
            durationIntervals: CONFIG.durationIntervals,
            depositAmount: CONFIG.depositAmount,
            createdAt: new Date().toISOString(),
            transactionHash: txHash,
            contractAddress: CONFIG.contractAddress,
        };
        
        const fileName = `subscription_${subscriptionId}.json`;
        fs.writeFileSync(fileName, JSON.stringify(subscriptionInfo, null, 2));
        log(`Subscription info saved to ${fileName}`);
        
        log('🎉 Subscription created successfully!');
        log(`📋 Subscription ID: ${subscriptionId}`);
        log(`💰 Initial deposit: ${parseFloat(CONFIG.depositAmount) / 1e18} ETH`);
        log(`⏰ Next payment in: ${CONFIG.intervalSeconds} seconds`);
        log(`🔄 Total intervals: ${CONFIG.durationIntervals}`);
        
        return subscriptionId;
        
    } catch (error) {
        log(`❌ Failed to create subscription: ${error.message}`);
        process.exit(1);
    }
}

// Run if script is executed directly
if (require.main === module) {
    createSubscription();
}

module.exports = { createSubscription };
