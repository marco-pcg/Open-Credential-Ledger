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
