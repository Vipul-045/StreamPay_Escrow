commands for lib5.rs

cd /home/faygo/MVP/subscription-stylus && cargo stylus deploy --private-key 487cc118443249eb8b08c190282f804554196a8ad9e8fa2df394b8658805a70d --endpoint https://sepolia-rollup.arbitrum.io/rpc --no-verify

cd /home/faygo/MVP/subscription-stylus && cast send 0x0000000000000000000000000000000000000069 "activateProgram(address)(uint64)" 0xad696d32efa1e12a3342769369217aaa450fa9ee --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key 487cc118443249eb8b08c190282f804554196a8ad9e8fa2df394b8658805a70d

cd /home/faygo/MVP/subscription-stylus && cast send 0xad696d32efa1e12a3342769369217aaa450fa9ee "initialize()" --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key 487cc118443249eb8b08c190282f804554196a8ad9e8fa2df394b8658805a70d

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "get_admin()" --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "get_protocol_fee()(uint256)" --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cargo stylus export-abi

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "getAdmin()(address)" --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "getProtocolFee()(uint256)" --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast send 0xad696d32efa1e12a3342769369217aaa450fa9ee "registerProvider(string)" "Test Provider Service" --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key 487cc118443249eb8b08c190282f804554196a8ad9e8fa2df394b8658805a70d

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "isProviderRegistered(address)(bool)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast send 0xad696d32efa1e12a3342769369217aaa450fa9ee "createPlan(uint256,uint256)" 1000000000000000 86400 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key 487cc118443249eb8b08c190282f804554196a8ad9e8fa2df394b8658805a70d

cd /home/faygo/MVP/subscription-stylus && cast send 0xad696d32efa1e12a3342769369217aaa450fa9ee "deposit()" --value 10000000000000000 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key 487cc118443249eb8b08c190282f804554196a8ad9e8fa2df394b8658805a70d

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "getUserBalance(address)(uint256)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast send 0xad696d32efa1e12a3342769369217aaa450fa9ee "subscribe(uint256)" 1 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key 487cc118443249eb8b08c190282f804554196a8ad9e8fa2df394b8658805a70d

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "getUserBalance(address)(uint256)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "getProviderEarnings(address)(uint256)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast send 0xad696d32efa1e12a3342769369217aaa450fa9ee "withdraw(uint256)" 1000000000000000 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key 487cc118443249eb8b08c190282f804554196a8ad9e8fa2df394b8658805a70d

cd /home/faygo/MVP/subscription-stylus && cast call 0xad696d32efa1e12a3342769369217aaa450fa9ee "getUserBalance(address)(uint256)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

