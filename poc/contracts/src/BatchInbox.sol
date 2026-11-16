// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import {
    IBatchInbox, Cursor, BatchSubmission, Unauthorized, InvalidSignature, InvalidCursor,
    InvalidRange, RequestConflict, StalePrev, InvalidOrigin, InvalidTarget, InvalidDA
} from "./L1ABI.sol";
import {TypedData} from "./TypedData.sol";

contract BatchInbox is IBatchInbox {
    bytes32 private constant NAME_HASH = keccak256("SequencerRotationBatchInbox");
    bytes32 private constant CURSOR_TYPE =
        keccak256("Cursor(uint64 height,uint64 origin,bytes32 framesDigest,uint8 sourceKind,bytes source)");
    bytes32 private constant DA_TYPE = keccak256("DAReceipt(bytes32 commitment)");
    bytes32 private constant BATCH_TYPE = keccak256(
        "BatchAttestation(bytes32 prevHash,bytes32 nextHash,bytes32 commitment,bytes32 daCertificateHash,uint64 firstHeight,uint64 lastHeight,bytes32 gposHash)"
    );

    address public immutable authority;
    address public immutable submitter;
    address public immutable certifier;
    address public immutable daSigner;
    bytes32 public immutable configRef;

    struct Accepted {
        bool exists;
        bytes32 content;
        bytes32 cursorHash;
        uint64 blockNumber;
    }

    Cursor private cursor;
    mapping(address => mapping(bytes32 => Accepted)) private requests;

    constructor(
        bytes32 configRef_, address submitter_, address certifier_, address daSigner_,
        uint64 genesisHeight, uint64 genesisOrigin
    ) {
        if (
            submitter_ == address(0) || certifier_ == address(0) || daSigner_ == address(0)
                || certifier_ == daSigner_ || configRef_ == bytes32(0)
        ) revert Unauthorized();
        if (genesisOrigin > block.number) revert InvalidOrigin();
        authority = msg.sender;
        submitter = submitter_;
        certifier = certifier_;
        daSigner = daSigner_;
        configRef = configRef_;
        Cursor memory genesis = Cursor(genesisHeight, genesisOrigin, bytes32(0), 0, "");
        cursor = genesis;
        emit CursorInitialized(genesis);
    }

    function currentCursor() external view returns (Cursor memory) {
        return cursor;
    }

    function hashCursor(Cursor memory value) public pure returns (bytes32) {
        return keccak256(abi.encode(
            CURSOR_TYPE, value.height, value.origin, value.framesDigest, value.sourceKind, keccak256(value.source)
        ));
    }

    function daDigest(bytes32 commitment) public view returns (bytes32) {
        return TypedData.digest(NAME_HASH, configRef, keccak256(abi.encode(DA_TYPE, commitment)));
    }

    function batchDigest(BatchSubmission calldata submission) external view returns (bytes32) {
        return TypedData.digest(NAME_HASH, configRef, batchHash(submission));
    }

    function batchHash(BatchSubmission calldata submission) private pure returns (bytes32) {
        return keccak256(abi.encode(
            BATCH_TYPE, hashCursor(submission.prev), hashCursor(submission.next), submission.commitment,
            keccak256(submission.daCertificate), submission.firstHeight, submission.lastHeight, keccak256(submission.gpos)
        ));
    }

    function submitBatch(bytes32 requestId, BatchSubmission calldata submission)
        external returns (bytes32 acceptedCursorHash, uint64 includedBlock)
    {
        if (msg.sender != submitter) revert Unauthorized();
        TypedData.requireCanonicalArguments(abi.encode(requestId, submission));
        if (!TypedData.signedBy(daDigest(submission.commitment), submission.daCertificate, daSigner)) revert InvalidDA();
        bytes32 content = batchHash(submission);
        if (!TypedData.signedBy(TypedData.digest(NAME_HASH, configRef, content), submission.certifierSignature, certifier)) {
            revert InvalidSignature();
        }

        Accepted storage result = requests[msg.sender][requestId];
        if (result.exists) {
            if (result.content != content) revert RequestConflict();
            return (result.cursorHash, result.blockNumber);
        }

        Cursor calldata prev = submission.prev;
        Cursor calldata next = submission.next;
        if (
            prev.framesDigest != bytes32(0) || next.framesDigest != bytes32(0) || !validSource(prev)
                || next.sourceKind != 1 || !validSource(next) || keccak256(next.source) != keccak256(submission.gpos)
        ) revert InvalidCursor();
        if (hashCursor(prev) != hashCursor(cursor)) revert StalePrev();
        if (
            uint256(submission.firstHeight) != uint256(prev.height) + 1
                || submission.lastHeight != next.height || submission.lastHeight < submission.firstHeight
                || submission.lastHeight - submission.firstHeight > 1
        ) revert InvalidRange();
        if (next.origin > block.number) revert InvalidOrigin();

        acceptedCursorHash = hashCursor(next);
        includedBlock = TypedData.narrow(block.number);
        cursor = next;
        requests[msg.sender][requestId] = Accepted(true, content, acceptedCursorHash, includedBlock);
        emit CursorAdvanced(msg.sender, requestId, prev, next, submission.commitment);
    }

    function overrideCursor(Cursor calldata next) external {
        if (msg.sender != authority) revert Unauthorized();
        TypedData.requireCanonicalArguments(abi.encode(next));
        if (next.height <= cursor.height) revert InvalidTarget();
        if (next.origin > block.number) revert InvalidOrigin();
        if (next.framesDigest != bytes32(0) || next.sourceKind != 2 || !validSource(next)) revert InvalidCursor();

        Cursor memory oldCursor = cursor;
        cursor = next;
        emit CursorSet(oldCursor, next);
    }

    function validSource(Cursor calldata value) private pure returns (bool) {
        if (value.sourceKind == 0) return value.source.length == 0;
        if (value.sourceKind == 1) return value.source.length > 0 && value.source.length <= 1024;
        return value.sourceKind == 2 && value.source.length == 32 && uint256(bytes32(value.source)) <= type(uint64).max;
    }
}
