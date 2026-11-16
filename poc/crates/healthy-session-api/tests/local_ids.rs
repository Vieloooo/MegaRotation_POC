use poc_healthy_session_api::{
    ApiError, ConfigRef, Hash32, IdKind, LocalId, Pid, WorkerScope, derive_local_id,
};

fn hash(byte: u8) -> Hash32 {
    Hash32::new([byte; 32])
}

fn scope(pid: u8, run: u8, config: u8) -> WorkerScope {
    WorkerScope {
        run: hash(run),
        config: ConfigRef::new(hash(config)),
        worker: Pid::new(pid).unwrap(),
        incarnation: 1,
    }
}

// Literal expectations from verification/healthy-session-program/api-v1/goldens.json.
#[test]
fn local_id_vectors() {
    let vectors = [
        (
            IdKind::Operation,
            9,
            7,
            0x11,
            0x22,
            0x33,
            "21ccd2b7d29da6b82c52b1db81704829812c0815ad8b1320c99fd1271bb40805",
        ),
        (
            IdKind::Program,
            9,
            7,
            0x11,
            0x22,
            0x33,
            "e4abd6fa408e44bd4909b58d69d17e633d04fba5b690c488fa25c45ba75bf8f1",
        ),
        (
            IdKind::Preparation,
            9,
            7,
            0x11,
            0x22,
            0x33,
            "ca9136cb1640cc16b65ba0a3b8ab36679584c8d23b33ec9f776ef6da66307980",
        ),
        (
            IdKind::Ready,
            9,
            7,
            0x11,
            0x22,
            0x33,
            "1d6540be767bd7b2c8eaa0cc0cf8e1b118d65ed51b47737b35b7009065591f7d",
        ),
        (
            IdKind::Action,
            9,
            7,
            0x11,
            0x22,
            0x33,
            "378abd8372b975ca5ec0a7a5d926c1bf9dfe89451ac0969b11ad8cf38500e851",
        ),
        (
            IdKind::ManagerDelta,
            9,
            7,
            0x11,
            0x22,
            0x33,
            "9a3aff0ea2122313497a3f08d601a7c95f4fbfea04778b19b31679447ebdc88f",
        ),
        (
            IdKind::ReplayBatch,
            9,
            7,
            0x11,
            0x22,
            0x33,
            "117e20cac3017a446ae04b69df473c4df261f8575fad7f7e023febd892301d6d",
        ),
        (
            IdKind::Delivery,
            9,
            7,
            0x11,
            0x22,
            0x33,
            "8dafb9a7984a31dc93d0aa895c9433140c42d8a7408a699b4592f9eace05f774",
        ),
        (
            IdKind::Action,
            10,
            7,
            0x11,
            0x22,
            0x33,
            "e19c244383676f7a97329d26ada91933adad80e74651d6c986da51cee3ab96c7",
        ),
        (
            IdKind::Action,
            9,
            0,
            0x11,
            0x22,
            0x33,
            "d4736e7591f7a1346ff3058db1c620d08eb7c31dc008d254463141bd711a1ebf",
        ),
        (
            IdKind::Action,
            9,
            14,
            0x11,
            0x22,
            0x33,
            "7c8dcdca0f00ee5a1ecd51d1489d1a764e0515586c06e0e5087ce25353db13b4",
        ),
        (
            IdKind::Action,
            9,
            7,
            0x11,
            0x00,
            0x33,
            "0a18b0b2ae1066e09f9347dfa73c191458cdb6ed1bcf128992e846bfb74f33a5",
        ),
        (
            IdKind::Action,
            9,
            7,
            0x11,
            0x22,
            0x00,
            "625950e4bcea0464cda70b5157a462aa125cd47a805ce41d2ff76b78bd5160d0",
        ),
        (
            IdKind::Action,
            9,
            8,
            0x11,
            0x22,
            0x33,
            "7dd0de3274921ad4e4e6058e94afc1bd8de73773a59db4601d4850efe823e654",
        ),
        (
            IdKind::Action,
            9,
            7,
            0x11,
            0x23,
            0x33,
            "2475e3751419fabb1ec89e140453a570d81c894d7eeb5871b80dbb63dc8fae28",
        ),
        (
            IdKind::Action,
            9,
            7,
            0x12,
            0x22,
            0x33,
            "2932163ac18314c631bdd432473f7c4d91a86d78ffb0c3070ab3e8f9dab9fd53",
        ),
    ];
    for (kind, ordinal, pid, run, config, subject, expected) in vectors {
        let id = derive_local_id(&scope(pid, run, config), kind, ordinal, hash(subject)).unwrap();
        assert_eq!(poc_types::hex(id.as_hash().as_bytes()), expected);
    }

    let action_nine_preimage = concat!(
        "000000000000001348312f4150492f436f6e74726f6c49642f76310000000000",
        "0000020001000000000000000105000000000000002011111111111111111111",
        "1111111111111111111111111111111111111111111100000000000000202222",
        "2222222222222222222222222222222222222222222222222222222222220000",
        "0000000000030001070000000000000008000000000000000100000000000000",
        "0800000000000000090000000000000020333333333333333333333333333333",
        "3333333333333333333333333333333333",
    );
    assert_eq!(action_nine_preimage.len() / 2, 209);
    let id = derive_local_id(&scope(7, 0x11, 0x22), IdKind::Action, 9, hash(0x33)).unwrap();
    let wrong_encodings = [
        "41c932440b7ecb97d6ce673f1bd83737992345953c9a4f84e7f7dc2610af72be",
        "8402f255098630f3bb8924d024d8910f8314e983ce323197640df2126f8b7293",
        "d2072c2e50cdbde87efc2db7d71121b980058785fe44076f9ac1a3e258e4eb6c",
    ];
    assert!(!wrong_encodings.contains(&poc_types::hex(id.as_hash().as_bytes()).as_str()));
}

#[test]
fn action_and_replay_subject_vectors() {
    let scope = scope(7, 0x11, 0x22);
    let delta = derive_local_id(&scope, IdKind::ManagerDelta, 9, hash(0x33)).unwrap();
    let actions = [
        "a9e540db91536960aebe8a0c5eb161fb129d0b6b7eb44701a08c6a6226b51ec1",
        "2f91045c3fdbc2a876c84873928d2c1b4f30d26103be1a3f2091ace1558ebd0e",
        "467f0575af4728f76e3fb0ca0bc8c77f70949b0ed43d265d3ab8843c6db46973",
    ];
    for (index, expected) in actions.into_iter().enumerate() {
        let id =
            derive_local_id(&scope, IdKind::Action, index as u64 + 1, *delta.as_hash()).unwrap();
        assert_eq!(poc_types::hex(id.as_hash().as_bytes()), expected);
    }

    let batches = [
        "9ef3f18f9e7c974fae96c4e4ef528aec53ab0a4df575c3950daffe62b8b339e5",
        "9d2d8f1d50eb424fdf7ffe604cb30ca6fae8c2d8d35942e2fdd39599e66dce19",
    ];
    for (index, expected) in batches.into_iter().enumerate() {
        let id =
            derive_local_id(&scope, IdKind::ReplayBatch, index as u64 + 1, hash(0x44)).unwrap();
        assert_eq!(poc_types::hex(id.as_hash().as_bytes()), expected);
    }
}

#[test]
fn rejects_invalid_local_identity_shape() {
    let mut scope = scope(7, 0, 0x22);
    assert_eq!(scope.validate_shape(), Err(ApiError::InvalidField("run")));
    scope.run = hash(0x11);
    for incarnation in [0, 2] {
        scope.incarnation = incarnation;
        assert_eq!(
            scope.validate_shape(),
            Err(ApiError::InvalidField("incarnation"))
        );
    }
    scope.incarnation = 1;
    assert_eq!(
        derive_local_id(&scope, IdKind::Operation, 0, hash(0x33)),
        Err(ApiError::InvalidField("ordinal")),
    );
    assert_eq!(
        LocalId::from_hash(hash(0)),
        Err(ApiError::InvalidField("local_id"))
    );
}
