// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import {ControlAuthority} from "./ControlAuthority.sol";
import {Policy, Unauthorized, InvalidSignature, InvalidPolicy} from "./L1ABI.sol";
import {TypedData} from "./TypedData.sol";

contract CommitteeControlAuthority is ControlAuthority {
    uint8 public immutable threshold;
    uint8 public immutable memberCount;
    mapping(address => bool) public member;

    constructor(
        bytes32 configRef_, address[] memory members_, uint8 threshold_,
        address submitter_, address certifier_, address daSigner_,
        uint64 announcementSeconds_, uint64 genesisHeight_, uint64 genesisOrigin_
    ) ControlAuthority(
        configRef_, requireFirst(members_, threshold_), submitter_, certifier_, daSigner_,
        announcementSeconds_, genesisHeight_, genesisOrigin_
    ) {
        address previous;
        for (uint256 i = 0; i < members_.length; ++i) {
            address signer = members_[i];
            if (signer <= previous || signer == certifier_ || signer == daSigner_) revert Unauthorized();
            member[signer] = true;
            previous = signer;
        }
        threshold = threshold_;
        memberCount = uint8(members_.length);
    }

    function requireFirst(address[] memory members_, uint8 threshold_) private pure returns (address) {
        if (threshold_ == 0 || threshold_ > members_.length || members_.length > 16) revert Unauthorized();
        return members_[0];
    }

    function checkSignatures(bytes32 digest, bytes calldata signatures) internal view override {
        if (signatures.length != uint256(threshold) * 65) revert InvalidSignature();
        address previous;
        for (uint256 offset = 0; offset < signatures.length; offset += 65) {
            address signer = TypedData.recover(digest, signatures[offset:offset + 65]);
            if (signer <= previous || !member[signer]) revert InvalidSignature();
            previous = signer;
        }
    }

    function checkPolicy(Policy calldata p) internal pure override {
        if (p.enabledMask > 7 || p.order0 > 2 || p.order1 > 2 || p.order2 > 2
            || p.order0 == p.order1 || p.order1 == p.order2 || p.order0 == p.order2) revert InvalidPolicy();
    }
}
