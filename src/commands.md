commands for lib5.rs

cd /home/faygo/MVP/subscription-stylus && cargo stylus deploy --private-key b9367a176b1e7c4a8764d3d5b7b31fc48cb966d6f8337e16193593015af1b255 --endpoint https://sepolia-rollup.arbitrum.io/rpc --no-verify

cd /home/faygo/MVP/subscription-stylus && cast send 0x0000000000000000000000000000000000000069 "activateProgram(address)(uint64)" 0x83694447b30f96086434c2c409ca7a42198693f0 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key b9367a176b1e7c4a8764d3d5b7b31fc48cb966d6f8337e16193593015af1b255

cd /home/faygo/MVP/subscription-stylus && cast send 0x83694447b30f96086434c2c409ca7a42198693f0 "initialize()" --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key b9367a176b1e7c4a8764d3d5b7b31fc48cb966d6f8337e16193593015af1b255

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "get_admin()" --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "get_protocol_fee()(uint256)" --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cargo stylus export-abi

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "getAdmin()(address)" --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "getProtocolFee()(uint256)" --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast send 0x83694447b30f96086434c2c409ca7a42198693f0 "registerProvider(string)" "Test Provider Service" --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key b9367a176b1e7c4a8764d3d5b7b31fc48cb966d6f8337e16193593015af1b255

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "isProviderRegistered(address)(bool)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast send 0x83694447b30f96086434c2c409ca7a42198693f0 "createPlan(uint256,uint256)" 1000000000000000 86400 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key b9367a176b1e7c4a8764d3d5b7b31fc48cb966d6f8337e16193593015af1b255

cd /home/faygo/MVP/subscription-stylus && cast send 0x83694447b30f96086434c2c409ca7a42198693f0 "deposit()" --value 10000000000000000 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key b9367a176b1e7c4a8764d3d5b7b31fc48cb966d6f8337e16193593015af1b255

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "getUserBalance(address)(uint256)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast send 0x83694447b30f96086434c2c409ca7a42198693f0 "subscribe(uint256)" 1 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key b9367a176b1e7c4a8764d3d5b7b31fc48cb966d6f8337e16193593015af1b255

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "getUserBalance(address)(uint256)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "getProviderEarnings(address)(uint256)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

cd /home/faygo/MVP/subscription-stylus && cast send 0x83694447b30f96086434c2c409ca7a42198693f0 "withdraw(uint256)" 1000000000000000 --rpc-url https://sepolia-rollup.arbitrum.io/rpc --private-key b9367a176b1e7c4a8764d3d5b7b31fc48cb966d6f8337e16193593015af1b255

cd /home/faygo/MVP/subscription-stylus && cast call 0x83694447b30f96086434c2c409ca7a42198693f0 "getUserBalance(address)(uint256)" 0x23316A7AF939a09c0Ee9A57Dece71ba7f2A0F996 --rpc-url https://sepolia-rollup.arbitrum.io/rpc

