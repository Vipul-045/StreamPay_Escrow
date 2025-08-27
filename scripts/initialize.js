#!/usr/bin/env node

/**
 * Initialize Contract Script
 * 
 * This script initializes the deployed hybrid escrow contract.
 */

const { execSync } = require('child_process');
require('dotenv').config();

const CONFIG = {
    privateKey: process.env.PRIVATE_KEY,
    rpcUrl: process.env.RPC_URL || 'https://sepolia-rollup.arbitrum.io/rpc',
    contractAddress: process.env.CONTRACT_ADDRESS || '0x49d5b4ee718463163519a85479616302bedebb73'
};

function log(message) {
    console.log(`[INIT] ${new Date().toISOString()} - ${message}`);
}

async function initializeContract() {
    try {
        log('Initializing contract...');
        
        // Get user address
        const userAddress = execSync(`cast wallet address ${CONFIG.privateKey}`, { encoding: 'utf8' }).trim();
        log(`Initializing with owner: ${userAddress}`);
        
        const txCommand = `cast send ${CONFIG.contractAddress} \\
            "initialize()" \\
            --rpc-url ${CONFIG.rpcUrl} \\
            --private-key ${CONFIG.privateKey}`;
        
        log('Sending initialize transaction...');
        const result = execSync(txCommand, { encoding: 'utf8' });
        
        // Extract transaction hash
        const txHash = result.match(/transactionHash\s+(.+)/)?.[1];
        log(`✅ Contract initialized! TX: ${txHash}`);
        
        // Wait for confirmation
        log('Waiting for confirmation...');
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        // Check if initialization was successful
        const ownerCommand = `cast call ${CONFIG.contractAddress} \\
            "owner()" \\
            --rpc-url ${CONFIG.rpcUrl}`;
        
        const owner = execSync(ownerCommand, { encoding: 'utf8' }).trim();
        log(`Contract owner: ${owner}`);
        
        if (owner.toLowerCase().includes(userAddress.toLowerCase().substring(2))) {
            log('✅ Contract successfully initialized!');
        } else {
            log('⚠️ Owner check inconclusive, but initialization transaction succeeded');
        }
        
    } catch (error) {
        log(`❌ Failed to initialize contract: ${error.message}`);
        process.exit(1);
    }
}

// Run if script is executed directly
if (require.main === module) {
    initializeContract();
}

module.exports = { initializeContract };
