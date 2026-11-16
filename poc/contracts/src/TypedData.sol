// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import {Overflow} from "./L1ABI.sol";

library TypedData {
    bytes32 private constant DOMAIN_TYPE = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)"
    );
    bytes32 private constant VERSION_HASH = keccak256("1");
    uint256 private constant CURVE_ORDER =
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;
    uint256 private constant HALF_ORDER =
        0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF5D576E7357A4501DDFE92F46681B20A0;

    function requireCanonicalArguments(bytes memory encoded) internal pure {
        // Solidity's decoder permits trailing or aliased data. Keep accepted
        // calldata usable as the unique frozen ABI representation in receipts.
        if (keccak256(msg.data[4:]) != keccak256(encoded)) revert();
    }

    function digest(bytes32 nameHash, bytes32 salt, bytes32 structHash) internal view returns (bytes32) {
        bytes32 domain = keccak256(abi.encode(DOMAIN_TYPE, nameHash, VERSION_HASH, block.chainid, address(this), salt));
        return keccak256(bytes.concat(hex"1901", domain, structHash));
    }

    function signedBy(bytes32 message, bytes calldata signature, address expected) internal pure returns (bool) {
        if (signature.length != 65) return false;

        bytes32 r = bytes32(signature[:32]);
        bytes32 s = bytes32(signature[32:64]);
        uint8 v = uint8(signature[64]);
        if (v != 27 && v != 28) return false;
        if (uint256(r) == 0 || uint256(r) >= CURVE_ORDER || uint256(s) == 0 || uint256(s) > HALF_ORDER) return false;

        address recovered = ecrecover(message, v, r, s);
        return recovered != address(0) && recovered == expected;
    }

    function narrow(uint256 value) internal pure returns (uint64) {
        if (value > type(uint64).max) revert Overflow();
        return uint64(value);
    }
}
