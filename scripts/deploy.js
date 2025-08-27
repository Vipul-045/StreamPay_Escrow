#!/usr/bin/env node

/**
 * Deploy Script for Hybrid Escrow Recurring Payments Contract
 * 
 * This script deploys the contract to Arbitrum Sepolia testnet
 * and initializes it for immediate use.
 */

require('dotenv').config();
const { execSync } = require('child_process');
const fs = require('fs');

// Configuration
const CONFIG = {
    rpcUrl: 'https://sepolia-rollup.arbitrum.io/rpc',
    privateKey: process.env.PRIVATE_KEY, // Set via environment variable
    chainId: 421614, // Arbitrum Sepolia
};

function log(message) {
    console.log(`[DEPLOY] ${new Date().toISOString()} - ${message}`);
}

async function deployContract() {
    try {
        log('Starting deployment process...');
        
        // Check if private key is set
        if (!CONFIG.privateKey) {
            throw new Error('PRIVATE_KEY environment variable not set');
        }
        
        // Build the contract first
        log('Building contract...');
        execSync('cargo build --release', { stdio: 'inherit' });
        
        // Skip ABI generation for now since it's causing issues
        log('Skipping ABI generation (will be auto-generated on deployment)...');
        
        // Deploy using cargo stylus with no verification
        log('Deploying to Arbitrum Sepolia...');
        const deployCommand = `cargo stylus deploy \\
            --endpoint ${CONFIG.rpcUrl} \\
            --private-key ${CONFIG.privateKey} \\
            --no-verify`;
        
        const deployOutput = execSync(deployCommand, { encoding: 'utf8' });
        
        // Extract contract address from output
        const addressMatch = deployOutput.match(/deployed code at address (0x[a-fA-F0-9]{40})/);
        if (!addressMatch) {
            throw new Error('Could not extract contract address from deploy output');
        }
        
        const contractAddress = addressMatch[1];
        log(`Contract deployed at: ${contractAddress}`);
        
        // Initialize the contract
        log('Initializing contract...');
        const initCommand = `cast send ${contractAddress} \\
            "initialize()" \\
            --rpc-url ${CONFIG.rpcUrl} \\
            --private-key ${CONFIG.privateKey}`;
        
        execSync(initCommand, { stdio: 'inherit' });
        log('Contract initialized successfully');
        
        // Save deployment info
        const deploymentInfo = {
            contractAddress,
            chainId: CONFIG.chainId,
            deployedAt: new Date().toISOString(),
            deployer: getAddressFromPrivateKey(CONFIG.privateKey),
            rpcUrl: CONFIG.rpcUrl,
        };
        
        fs.writeFileSync('deployment.json', JSON.stringify(deploymentInfo, null, 2));
        log('Deployment info saved to deployment.json');
        
        // Create environment file for other scripts
        const envContent = `
# Hybrid Escrow Contract Deployment
CONTRACT_ADDRESS=${contractAddress}
RPC_URL=${CONFIG.rpcUrl}
CHAIN_ID=${CONFIG.chainId}
DEPLOYED_AT=${deploymentInfo.deployedAt}
`;
        
        fs.writeFileSync('.env.deployed', envContent);
        log('Environment file created at .env.deployed');
        
        log('🎉 Deployment completed successfully!');
        log(`📋 Contract Address: ${contractAddress}`);
        log(`🌐 Network: Arbitrum Sepolia (${CONFIG.chainId})`);
        log(`⏰ Deployed at: ${deploymentInfo.deployedAt}`);
        
        return contractAddress;
        
    } catch (error) {
        log(`❌ Deployment failed: ${error.message}`);
        process.exit(1);
    }
}

function getAddressFromPrivateKey(privateKey) {
    try {
        const address = execSync(`cast wallet address ${privateKey}`, { encoding: 'utf8' }).trim();
        return address;
    } catch (error) {
        return 'Unknown';
    }
}

// Run deployment if script is executed directly
if (require.main === module) {
    deployContract();
}

module.exports = { deployContract };
