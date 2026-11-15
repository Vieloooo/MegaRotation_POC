// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

struct Policy {
    uint8 enabledMask;
    uint8 order0;
    uint8 order1;
    uint8 order2;
    uint64 l1Floor;
}

struct Cursor {
    uint64 height;
    uint64 origin;
    bytes32 framesDigest;
    uint8 sourceKind;
    bytes source;
}

struct BatchSubmission {
    Cursor prev;
    Cursor next;
    bytes32 commitment;
    bytes daCertificate;
    uint64 firstHeight;
    uint64 lastHeight;
    bytes gpos;
    bytes certifierSignature;
}

error Unauthorized();
error InvalidSignature();
error InvalidPolicy();
error InvalidCursor();
error InvalidRange();
error InvalidSequence();
error RequestConflict();
error StalePrev();
error InvalidOrigin();
error InvalidTarget();
error InvalidDA();
error Overflow();

interface IControlAuthority {
    event ForceRecorded(
        address indexed caller, bytes32 indexed requestId, uint64 indexed controlSeq,
        uint64 targetBranch, Policy policy, uint64 act
    );
    event ResumeRecorded(
        address indexed caller, bytes32 indexed requestId, uint64 indexed controlSeq,
        Policy policy, uint64 targetHeight, uint64 origin, uint64 act
    );

    function submitForce(bytes32 requestId, uint64 controlSeq, uint64 targetBranch, Policy calldata policy, bytes calldata signature)
        external returns (uint64 acceptedSeq, uint64 includedBlock, uint64 act);

    function submitResume(bytes32 requestId, uint64 controlSeq, Policy calldata policy, uint64 targetHeight, uint64 origin, bytes calldata signature)
        external returns (uint64 acceptedSeq, uint64 includedBlock, uint64 act);

    function hashPolicy(Policy calldata policy) external pure returns (bytes32);

    function forceDigest(address caller, bytes32 rid, uint64 seq, uint64 branch, Policy calldata policy)
        external view returns (bytes32);

    function resumeDigest(address caller, bytes32 rid, uint64 seq, Policy calldata policy, uint64 height, uint64 origin)
        external view returns (bytes32);
}

interface IBatchInbox {
    event CursorInitialized(Cursor cursor);
    event CursorAdvanced(
        address indexed caller, bytes32 indexed requestId, Cursor prev, Cursor next, bytes32 commitment
    );
    event CursorSet(Cursor oldCursor, Cursor next);

    function submitBatch(bytes32 requestId, BatchSubmission calldata submission)
        external returns (bytes32 acceptedCursorHash, uint64 includedBlock);
    function overrideCursor(Cursor calldata next) external;
    function currentCursor() external view returns (Cursor memory);
    function hashCursor(Cursor calldata cursor) external pure returns (bytes32);
    function daDigest(bytes32 commitment) external view returns (bytes32);
    function batchDigest(BatchSubmission calldata submission) external view returns (bytes32);
}
