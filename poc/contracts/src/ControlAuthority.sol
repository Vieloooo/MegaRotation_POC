// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import {
    IControlAuthority, Policy, Cursor, Unauthorized, InvalidSignature, InvalidPolicy,
    InvalidSequence, RequestConflict, Overflow
} from "./L1ABI.sol";
import {BatchInbox} from "./BatchInbox.sol";
import {TypedData} from "./TypedData.sol";

contract ControlAuthority is IControlAuthority {
    bytes32 private constant NAME_HASH = keccak256("SequencerRotationControlAuthority");
    bytes32 private constant POLICY_TYPE = keccak256("Policy(uint8 enabledMask,uint8 order0,uint8 order1,uint8 order2,uint64 l1Floor)");
    bytes32 private constant FORCE_TYPE = keccak256("Force(address caller,bytes32 requestId,uint64 controlSeq,uint64 targetBranch,bytes32 policyHash)");
    bytes32 private constant RESUME_TYPE = keccak256("Resume(address caller,bytes32 requestId,uint64 controlSeq,bytes32 policyHash,uint64 targetHeight,uint64 origin)");

    address public immutable governance;
    BatchInbox public immutable inbox;
    bytes32 public immutable configRef;
    uint64 public immutable announcementSeconds;

    bool public hasControl;
    uint64 public lastControlSeq;

    struct Accepted {
        bool exists;
        bytes32 content;
        uint64 seq;
        uint64 blockNumber;
        uint64 act;
    }

    mapping(address => mapping(bytes32 => Accepted)) private requests;

    constructor(
        bytes32 configRef_, address governance_, address submitter_, address certifier_, address daSigner_,
        uint64 announcementSeconds_, uint64 genesisHeight, uint64 genesisOrigin
    ) {
        if (governance_ == address(0) || governance_ == certifier_ || governance_ == daSigner_) revert Unauthorized();
        if (announcementSeconds_ == 0) revert InvalidPolicy();
        configRef = configRef_;
        governance = governance_;
        announcementSeconds = announcementSeconds_;
        inbox = new BatchInbox(configRef_, submitter_, certifier_, daSigner_, genesisHeight, genesisOrigin);
    }

    function hashPolicy(Policy calldata policy) public pure returns (bytes32) {
        return keccak256(abi.encode(POLICY_TYPE, policy.enabledMask, policy.order0, policy.order1, policy.order2, policy.l1Floor));
    }

    function forceDigest(address caller, bytes32 rid, uint64 seq, uint64 branch, Policy calldata policy)
        external view returns (bytes32)
    {
        return TypedData.digest(NAME_HASH, configRef, forceHash(caller, rid, seq, branch, policy));
    }

    function resumeDigest(address caller, bytes32 rid, uint64 seq, Policy calldata policy, uint64 height, uint64 origin)
        external view returns (bytes32)
    {
        return TypedData.digest(NAME_HASH, configRef, resumeHash(caller, rid, seq, policy, height, origin));
    }

    function submitForce(bytes32 requestId, uint64 controlSeq, uint64 targetBranch, Policy calldata policy, bytes calldata signature)
        external returns (uint64 acceptedSeq, uint64 includedBlock, uint64 act)
    {
        TypedData.requireCanonicalArguments(abi.encode(requestId, controlSeq, targetBranch, policy, signature));
        bytes32 content = forceHash(msg.sender, requestId, controlSeq, targetBranch, policy);
        (Accepted memory result, bool replay) = accept(requestId, controlSeq, policy, content, signature);
        if (!replay) emit ForceRecorded(msg.sender, requestId, controlSeq, targetBranch, policy, result.act);
        return (result.seq, result.blockNumber, result.act);
    }

    function submitResume(bytes32 requestId, uint64 controlSeq, Policy calldata policy, uint64 targetHeight, uint64 origin, bytes calldata signature)
        external returns (uint64 acceptedSeq, uint64 includedBlock, uint64 act)
    {
        TypedData.requireCanonicalArguments(abi.encode(requestId, controlSeq, policy, targetHeight, origin, signature));
        bytes32 content = resumeHash(msg.sender, requestId, controlSeq, policy, targetHeight, origin);
        (Accepted memory result, bool replay) = accept(requestId, controlSeq, policy, content, signature);
        if (!replay) {
            inbox.overrideCursor(Cursor(targetHeight, origin, bytes32(0), 2, abi.encode(controlSeq)));
            emit ResumeRecorded(msg.sender, requestId, controlSeq, policy, targetHeight, origin, result.act);
        }
        return (result.seq, result.blockNumber, result.act);
    }

    function accept(bytes32 rid, uint64 seq, Policy calldata policy, bytes32 content, bytes calldata signature)
        private returns (Accepted memory result, bool replay)
    {
        checkSignatures(TypedData.digest(NAME_HASH, configRef, content), signature);
        result = requests[msg.sender][rid];
        if (result.exists) {
            if (result.content != content) revert RequestConflict();
            return (result, true);
        }
        if (hasControl && seq <= lastControlSeq) revert InvalidSequence();
        checkPolicy(policy);

        uint64 timestamp = TypedData.narrow(block.timestamp);
        if (timestamp > type(uint64).max - announcementSeconds) revert Overflow();
        result = Accepted(true, content, seq, TypedData.narrow(block.number), timestamp + announcementSeconds);
        hasControl = true;
        lastControlSeq = seq;
        requests[msg.sender][rid] = result;
    }

    function checkSignatures(bytes32 digest, bytes calldata signatures) internal view virtual {
        if (!TypedData.signedBy(digest, signatures, governance)) revert InvalidSignature();
    }

    function checkPolicy(Policy calldata policy) internal pure virtual {
        if (policy.enabledMask != 7 || policy.order0 != 0 || policy.order1 != 1 || policy.order2 != 2) revert InvalidPolicy();
    }

    function forceHash(address caller, bytes32 rid, uint64 seq, uint64 branch, Policy calldata policy)
        private pure returns (bytes32)
    {
        return keccak256(abi.encode(FORCE_TYPE, caller, rid, seq, branch, hashPolicy(policy)));
    }

    function resumeHash(address caller, bytes32 rid, uint64 seq, Policy calldata policy, uint64 height, uint64 origin)
        private pure returns (bytes32)
    {
        return keccak256(abi.encode(RESUME_TYPE, caller, rid, seq, hashPolicy(policy), height, origin));
    }
}
