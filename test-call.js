const { ethers } = require('ethers');

async function testContract() {
    const provider = new ethers.providers.JsonRpcProvider('https://sepolia-rollup.arbitrum.io/rpc');
    const wallet = new ethers.Wallet('0xa9748e8e75e0ba6e7f5c3cf34b86e2b1093a1e3f3b4d7c2ed1e3f9a2e3a4d5b6', provider);
    
    // Test calling owner function on our previous deployed contract that was having issues
    const contractAddress = '0x15bb9a22a98790f1f6085a5a441b15a9dd86fd88';
    
    try {
        // Try to call owner function using ethers
        const contract = new ethers.Contract(contractAddress, [
            'function owner() view returns (address)',
            'function initialize()'
        ], wallet);
        
        console.log('Testing owner() call...');
        const owner = await contract.owner();
        console.log('Owner:', owner);
        
    } catch (error) {
        console.error('Error calling contract:', error.message);
    }
}

testContract();
