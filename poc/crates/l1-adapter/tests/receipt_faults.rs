use l1_adapter::{
    Action, Command, Config, Driver, Outcome, Query,
    rpc::{Rpc, RpcError},
};
use serde_json::{Value, json};
use std::{cell::RefCell, fs::File, rc::Rc};

#[derive(Clone, Copy, Default)]
enum Inject {
    #[default]
    None,
    MissingBlock,
    HeaderOnly,
    MissingReceipt,
    MissingLogs,
    MissingResume,
    WrongEmitter,
    WrongGlobalIndex,
    WrongTransaction,
    WrongParent,
}

#[derive(Default)]
struct Chain {
    head: usize,
    mines: usize,
    fault: Inject,
}

struct ReplayRpc {
    bundle: Value,
    chain: Rc<RefCell<Chain>>,
}

impl Rpc for ReplayRpc {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let mut chain = self.chain.borrow_mut();
        let value = match method {
            "eth_chainId" => json!("0x7a69"),
            "anvil_getAutomine" => json!(false),
            "eth_blockNumber" => json!(format!("0x{:x}", chain.head)),
            "evm_setNextBlockTimestamp" => Value::Null,
            "evm_mine" => {
                chain.head += 1;
                chain.mines += 1;
                json!("0x0")
            }
            "eth_getBlockByNumber" => {
                let number = usize::from_str_radix(&params[0].as_str().unwrap()[2..], 16).unwrap();
                assert!(number <= chain.head);
                let mut block = self.bundle["blocks"][number].clone();
                if number == 4 {
                    match chain.fault {
                        Inject::MissingBlock => block = Value::Null,
                        Inject::HeaderOnly => {
                            block.as_object_mut().unwrap().remove("transactions");
                        }
                        Inject::WrongParent => {
                            block["parentHash"] = json!(format!("0x{}", "00".repeat(32)));
                        }
                        _ => {}
                    }
                }
                block
            }
            "eth_getTransactionReceipt" => {
                let mut receipt = self.bundle["receipts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|receipt| receipt["transactionHash"] == params[0])
                    .unwrap()
                    .clone();
                if receipt["blockNumber"] == "0x4" && receipt["transactionIndex"] == "0x2" {
                    match chain.fault {
                        Inject::MissingReceipt => receipt = Value::Null,
                        Inject::MissingLogs => {
                            receipt.as_object_mut().unwrap().remove("logs");
                        }
                        Inject::MissingResume => {
                            receipt["logs"].as_array_mut().unwrap().pop();
                        }
                        Inject::WrongEmitter => {
                            receipt["logs"][1]["address"] = json!(format!("0x{}", "99".repeat(20)));
                        }
                        Inject::WrongGlobalIndex => {
                            receipt["logs"][1]["logIndex"] = json!("0x1");
                        }
                        Inject::WrongTransaction => {
                            receipt["transactionHash"] = json!(format!("0x{}", "99".repeat(32)));
                        }
                        _ => {}
                    }
                }
                receipt
            }
            _ => panic!("unexpected RPC {method}"),
        };
        Ok(value)
    }
}

fn setup(bundle_path: &str) -> (Driver<ReplayRpc>, Rc<RefCell<Chain>>) {
    let bundle: Value = serde_json::from_reader(File::open(bundle_path).unwrap()).unwrap();
    let c = &bundle["config"];
    let string = |key: &str| c[key].as_str().unwrap().to_owned();
    let number = |key: &str| c[key].as_u64().unwrap();
    let config = Config {
        chain_id: number("chainId"),
        config_ref: string("configRef"),
        authority: string("authority"),
        inbox: string("inbox"),
        submitter: string("submitter"),
        genesis_height: number("genesisHeight"),
        genesis_origin: number("genesisOrigin"),
        genesis_timestamp_seconds: number("genesisTimestampSeconds"),
        slot_seconds: number("slotSeconds"),
        announcement_seconds: number("announcementSeconds"),
        finality_delay_us: 2_000_000,
        readers: vec!["A".into(), "B".into()],
    };
    let chain = Rc::new(RefCell::new(Chain::default()));
    let driver = Driver::new(
        config,
        ReplayRpc {
            bundle,
            chain: chain.clone(),
        },
    )
    .unwrap();
    (driver, chain)
}

#[test]
#[ignore = "requires the real Anvil scenario raw-rpc-bundle.json"]
fn complete_receipts_are_required_before_inclusion_coverage() {
    let bundle = std::env::var("L1_ADAPTER_RPC_BUNDLE").expect("provide the scenario bundle");
    let (mut driver, chain) = setup(&bundle);
    let mut id = 0;
    let mut apply = |driver: &mut Driver<ReplayRpc>, action| {
        id += 1;
        driver.apply(Command { id, action })
    };
    for slot in 1..=3 {
        let reply = apply(
            &mut driver,
            Action::Tick {
                reference_us: slot * 2_000_000,
            },
        );
        assert!(matches!(reply.outcome, Outcome::Ready { .. }));
        let reply = apply(&mut driver, Action::Mine { slot });
        assert_eq!(reply.included_height, slot);
    }
    assert!(matches!(
        apply(&mut driver, Action::Finalize { through_height: 2 }).outcome,
        Outcome::Ready { .. }
    ));
    assert!(matches!(
        apply(
            &mut driver,
            Action::Deliver {
                reader: "A".into(),
                through_height: 2,
                from_height: None,
            }
        )
        .outcome,
        Outcome::Ready { .. }
    ));
    apply(
        &mut driver,
        Action::Tick {
            reference_us: 8_000_000,
        },
    );
    let mine = Command {
        id: id + 1,
        action: Action::Mine { slot: 4 },
    };

    for fault in [
        Inject::MissingBlock,
        Inject::HeaderOnly,
        Inject::MissingReceipt,
        Inject::MissingLogs,
    ] {
        chain.borrow_mut().fault = fault;
        let reply = driver.apply(mine.clone());
        assert!(matches!(reply.outcome, Outcome::Wait { .. }), "{reply:?}");
        assert_eq!(reply.included_height, 3);
        assert_eq!(reply.finalized_height, 2);
        assert_eq!(reply.visible_heights["A"], 2);
        assert_eq!(reply.visible_heights["B"], 0);
        assert_eq!(chain.borrow().mines, 4);
    }
    for fault in [
        Inject::MissingResume,
        Inject::WrongEmitter,
        Inject::WrongGlobalIndex,
        Inject::WrongTransaction,
        Inject::WrongParent,
    ] {
        chain.borrow_mut().fault = fault;
        let reply = driver.apply(mine.clone());
        assert!(matches!(reply.outcome, Outcome::Reject { .. }), "{reply:?}");
        assert_eq!(reply.included_height, 3);
        assert_eq!(reply.finalized_height, 2);
        assert_eq!(reply.visible_heights["A"], 2);
        assert_eq!(reply.visible_heights["B"], 0);
        assert_eq!(chain.borrow().mines, 4);
    }
    let blocked = driver.apply(Command {
        id: mine.id + 1,
        action: Action::Mine { slot: 5 },
    });
    assert!(matches!(blocked.outcome, Outcome::Wait { .. }));
    let gated = driver.apply(Command {
        id: mine.id + 2,
        action: Action::Read {
            reader: "A".into(),
            query: Query::Block { height: 4 },
        },
    });
    assert!(matches!(gated.outcome, Outcome::Wait { .. }));
    chain.borrow_mut().fault = Inject::None;
    let repaired = driver.apply(mine.clone());
    assert!(
        matches!(repaired.outcome, Outcome::Ready { .. }),
        "{repaired:?}"
    );
    assert_eq!(repaired.included_height, 4);
    assert_eq!(repaired.finalized_height, 2);
    assert_eq!(driver.apply(mine), repaired);
    assert_eq!(chain.borrow().mines, 4);
}
