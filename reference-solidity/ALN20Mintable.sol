// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * ALN20Mintable (Reference only)
 * - Controlled mint function via MINTER_ROLE
 * - finalization step to disable further minting
 */
contract ALN20Mintable is ERC20, AccessControl {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bool public mintingFinalized;

    constructor(
        string memory name_,
        string memory symbol_,
        address admin,
        uint256 initialSupply
    ) ERC20(name_, symbol_) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(MINTER_ROLE, admin);
        _mint(admin, initialSupply);
    }

    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        require(!mintingFinalized, "Minting disabled");
        _mint(to, amount);
    }

    function finalizeMinting() external onlyRole(DEFAULT_ADMIN_ROLE) {
        mintingFinalized = true;
    }

    function decimals() public pure override returns (uint8) {
        return 6;
    }
}
