// SPDX-License-Identifier: MIT
pragma solidity 0.8.28;

/// @notice Institutional trust registry. Never put student data on chain.
contract IssuerRegistry {
    address public owner;
    address public pendingOwner;
    mapping(address => mapping(bytes32 => bool)) private authorized;
    mapping(address => mapping(bytes32 => bool)) private revokedCredentials;
    event CredentialRevoked(address indexed issuer, bytes32 indexed handle);
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

    /// @notice Permanently revoke an issuer-scoped opaque handle; never pass student data.
    function revokeCredential(address issuer, bytes32 handle) external onlyOwner {
        if (issuer == address(0) || handle == bytes32(0)) revert InvalidInput();
        if (!revokedCredentials[issuer][handle]) {
            revokedCredentials[issuer][handle] = true;
            emit CredentialRevoked(issuer, handle);
        }
    }

    function isRevoked(address issuer, bytes32 handle) external view returns (bool) {
        return revokedCredentials[issuer][handle];
    }

    /// @notice Read key trust and credential status from the same EVM state snapshot.
    function verificationStatus(address issuer, bytes32 keyHash, bytes32 handle)
        external
        view
        returns (bool keyAuthorized, bool credentialRevoked)
    {
        return (authorized[issuer][keyHash], revokedCredentials[issuer][handle]);
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
