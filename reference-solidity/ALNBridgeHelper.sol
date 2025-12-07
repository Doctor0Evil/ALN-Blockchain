// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract ALNBridgeHelper {
    mapping(bytes32 => bool) public claimed;
    address public admin;

    event Claimed(bytes32 indexed snapshotHash, address indexed recipient, uint256 energyAmount, uint256 strategyAmount);

    constructor(address admin_) {
        admin = admin_;
    }

    // claim: caller must be admin or authorized off-chain
    function claim(address auetToken, address cspToken, bytes32 snapshotHash, address to, uint256 energyAmount, uint256 strategyAmount) external {
        require(!claimed[snapshotHash], "already claimed");
        // NOTE: Caller verification should be done via off-chain checks and on-chain admin role enforcement
        // Transfer tokens from bridge fund to recipient
        if (energyAmount > 0) {
            require(IERC20(auetToken).transfer(to, energyAmount), "transfer failed");
        }
        if (strategyAmount > 0) {
            require(IERC20(cspToken).transfer(to, strategyAmount), "transfer failed");
        }
        claimed[snapshotHash] = true;
        emit Claimed(snapshotHash, to, energyAmount, strategyAmount);
    }
}
