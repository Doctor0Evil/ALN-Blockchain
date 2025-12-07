// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/**
 * ALN20NonMintable
 * - Mint occurs only in constructor
 * - Immutable provenance metadata
 * - No external mint function
 */
contract ALN20NonMintable is ERC20 {
    bytes32 public immutable sourceChainId; // e.g. keccak256("kaiyo-1") or bytes32 representation
    bytes public immutable sourceDenom;     // original denom string
    uint64 public immutable snapshotHeight; // snapshot height on Kujira
    bytes32 public immutable snapshotRoot;  // Merkle root of snapshot H_i values

    constructor(
        string memory name_,
        string memory symbol_,
        uint8 decimals_,
        address[] memory holders,
        uint256[] memory amounts,
        bytes32 sourceChainId_,
        bytes memory sourceDenom_,
        uint64 snapshotHeight_,
        bytes32 snapshotRoot_
    ) ERC20(name_, symbol_) {
        require(holders.length == amounts.length, "length mismatch");
        sourceChainId = sourceChainId_;
        sourceDenom = sourceDenom_;
        snapshotHeight = snapshotHeight_;
        snapshotRoot = snapshotRoot_;

        uint256 total = 0;
        for (uint256 i = 0; i < holders.length; i++) {
            _mint(holders[i], amounts[i]);
            total += amounts[i];
        }
        // totalSupply is defined by constructor _mint() calls
        require(total == totalSupply(), "total supply invariant");
    }

    function decimals() public pure override returns (uint8) {
        return 6; // ALN canonical decimals; adjust as appropriate
    }

    // No external mint() functions
}
