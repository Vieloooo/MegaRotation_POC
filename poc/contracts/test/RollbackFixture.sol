// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import {BatchInbox} from "../src/BatchInbox.sol";
import {Cursor} from "../src/L1ABI.sol";

/// Deployed only by the fixture. The production Authority never calls this contract.
contract RollbackFixture {
    error IntentionalRollback();
    event Recorded(uint64 seq);

    BatchInbox public immutable inbox;
    uint64 public seq;
    mapping(bytes32 => bool) public seen;

    constructor(bytes32 salt, address submitter, address certifier, address daSigner) {
        inbox = new BatchInbox(salt, submitter, certifier, daSigner, 0, 0);
    }

    function resume(bytes32 rid, uint64 nextSeq, Cursor calldata next, bool failAfterWrite) external {
        seq = nextSeq;
        seen[rid] = true;
        inbox.overrideCursor(next);
        emit Recorded(nextSeq);
        if (failAfterWrite) revert IntentionalRollback();
    }

    function overrideRaw(bytes calldata input) external {
        (bool ok,) = address(inbox).call(input);
        require(ok);
    }
}
