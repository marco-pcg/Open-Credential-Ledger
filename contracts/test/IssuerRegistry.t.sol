// SPDX-License-Identifier: MIT
pragma solidity 0.8.28;
import "../src/IssuerRegistry.sol";

interface Vm {
    function prank(address) external;
    function expectRevert(bytes4) external;
}

contract IssuerRegistryTest {
    Vm constant vm = Vm(address(uint160(uint256(keccak256("hevm cheat code")))));
    IssuerRegistry registry;

    function setUp() public {
        registry = new IssuerRegistry(address(this));
    }

    function testAuthorizeAndRevoke() public {
        bytes32 key = keccak256("public key");
        require(!registry.isAuthorized(address(1), key));
        registry.setAuthorization(address(1), key, true);
        require(registry.isAuthorized(address(1), key));
        registry.setAuthorization(address(1), key, false);
        require(!registry.isAuthorized(address(1), key));
    }

    function testUnauthorized() public {
        vm.expectRevert(IssuerRegistry.Unauthorized.selector);
        vm.prank(address(2));
        registry.setAuthorization(address(1), keccak256("key"), true);
    }

    function testInvalidIssuer() public {
        vm.expectRevert(IssuerRegistry.InvalidInput.selector);
        registry.setAuthorization(address(0), keccak256("key"), true);
    }

    function testOwnership() public {
        registry.transferOwnership(address(2));
        require(registry.owner() == address(this));
        vm.prank(address(2));
        registry.acceptOwnership();
        require(registry.owner() == address(2));
        vm.expectRevert(IssuerRegistry.Unauthorized.selector);
        registry.setAuthorization(address(1), keccak256("key"), true);
    }

    function testFuzzAuthorization(address issuer, bytes32 key) public {
        if (issuer == address(0) || key == bytes32(0)) return;
        registry.setAuthorization(issuer, key, true);
        require(registry.isAuthorized(issuer, key));
    }
}

contract CredentialRevocationTest {
    Vm constant vm = Vm(address(uint160(uint256(keccak256("hevm cheat code")))));
    IssuerRegistry registry;

    function setUp() public {
        registry = new IssuerRegistry(address(this));
    }

    function testSingleCredentialAndIssuerIsolation() public {
        address issuer = address(1);
        bytes32 key = keccak256("key");
        bytes32 first = keccak256("first");
        bytes32 second = keccak256("second");
        registry.setAuthorization(issuer, key, true);
        registry.revokeCredential(issuer, first);
        (bool authorized, bool revoked) = registry.verificationStatus(issuer, key, first);
        require(authorized && revoked);
        require(registry.isAuthorized(issuer, key));
        require(!registry.isRevoked(issuer, second));
        require(!registry.isRevoked(address(2), first));
        registry.revokeCredential(issuer, first); // Idempotent, never a toggle.
        require(registry.isRevoked(issuer, first));
    }

    function testRevocationSurvivesKeyAndOwnerChanges() public {
        bytes32 handle = keccak256("credential");
        bytes32 key = keccak256("key");
        registry.revokeCredential(address(1), handle);
        registry.setAuthorization(address(1), key, true);
        registry.setAuthorization(address(1), key, false);
        registry.setAuthorization(address(1), keccak256("new key"), true);
        registry.transferOwnership(address(2));
        vm.prank(address(2));
        registry.acceptOwnership();
        require(registry.isRevoked(address(1), handle));
        vm.expectRevert(IssuerRegistry.Unauthorized.selector);
        registry.revokeCredential(address(1), keccak256("other"));
    }

    function testOnlyOwnerCanRevoke() public {
        vm.expectRevert(IssuerRegistry.Unauthorized.selector);
        vm.prank(address(2));
        registry.revokeCredential(address(1), keccak256("credential"));
    }

    function testInvalidRevocationInput() public {
        vm.expectRevert(IssuerRegistry.InvalidInput.selector);
        registry.revokeCredential(address(0), keccak256("id"));
        vm.expectRevert(IssuerRegistry.InvalidInput.selector);
        registry.revokeCredential(address(1), bytes32(0));
    }

    function testHandleEncodingMatchesRust() public pure {
        bytes32 handle = keccak256(
            abi.encodePacked(
                "OpenCredentialLedger:revocation:v1\n",
                address(1),
                "urn:ocl:credential:000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
            )
        );
        require(handle == 0x2a87ef5f7a42d1511f11908039af99c8d8dadc09eddc07d9244bb421ee853ae9);
    }

    function testFuzzRevocationScope(address issuer, bytes32 handle) public {
        if (issuer == address(0) || handle == bytes32(0)) return;
        registry.revokeCredential(issuer, handle);
        require(registry.isRevoked(issuer, handle));
        require(!registry.isRevoked(issuer, bytes32(uint256(handle) ^ 1)));
    }
}
