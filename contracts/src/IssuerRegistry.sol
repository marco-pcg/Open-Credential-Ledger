// SPDX-License-Identifier: MIT
pragma solidity 0.8.28;

/// @notice Institutional trust registry. Never put student data on chain.
contract IssuerRegistry {
    address public owner;
    address public pendingOwner;
    mapping(address => mapping(bytes32 => bool)) private authorized;
    event AuthorizationChanged(address indexed issuer, bytes32 indexed keyHash, bool active);
    event OwnershipTransferStarted(address indexed candidate);
    event OwnershipTransferred(address indexed previous, address indexed current);
    error Unauthorized();
    error InvalidInput();

    constructor(address initialOwner) {
        if (initialOwner == address(0)) revert InvalidInput();
        owner = initialOwner;
    }
    modifier onlyOwner() {
        if (msg.sender != owner) revert Unauthorized();
        _;
    }

    function setAuthorization(address issuer, bytes32 keyHash, bool active) external onlyOwner {
        if (issuer == address(0) || keyHash == bytes32(0)) revert InvalidInput();
        authorized[issuer][keyHash] = active;
        emit AuthorizationChanged(issuer, keyHash, active);
    }

    function isAuthorized(address issuer, bytes32 keyHash) external view returns (bool) {
        return authorized[issuer][keyHash];
    }

    function transferOwnership(address candidate) external onlyOwner {
        if (candidate == address(0)) revert InvalidInput();
        pendingOwner = candidate;
        emit OwnershipTransferStarted(candidate);
    }

    function acceptOwnership() external {
        if (msg.sender != pendingOwner) revert Unauthorized();
        emit OwnershipTransferred(owner, msg.sender);
        owner = msg.sender;
        pendingOwner = address(0);
    }
}
