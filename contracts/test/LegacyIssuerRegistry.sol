// SPDX-License-Identifier: MIT
pragma solidity 0.8.28;

/// @dev Old-ABI test fixture: key authorization exists, credential status does not.
contract LegacyIssuerRegistry {
    function isAuthorized(address, bytes32) external pure returns (bool) {
        return true;
    }
}
