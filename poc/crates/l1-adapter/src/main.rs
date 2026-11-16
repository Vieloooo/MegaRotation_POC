use l1_adapter::{
    Command, Config, Driver, EvidenceSource,
    rpc::{AnvilRpc, Rpc, RpcError, RpcEvidence},
};
use serde_json::{Value, json};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, Write},
};

struct TraceRpc {
    anvil: AnvilRpc,
    mine_count: u64,
    drop_mine_answer: Option<u64>,
}

impl Rpc for TraceRpc {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let result = self.anvil.call(method, params);
        if method == "evm_mine" {
            self.mine_count += 1;
            if self.drop_mine_answer == Some(self.mine_count) && result.is_ok() {
                // The real mutation and its response are retained in evidence.
                return Err(RpcError::Transport(
                    "fixture dropped completed mine response".into(),
                ));
            }
        }
        result
    }
}

impl EvidenceSource for TraceRpc {
    fn evidence(&mut self) -> &mut Vec<RpcEvidence> {
        &mut self.anvil.evidence
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if !(3..=4).contains(&args.len()) {
        return Err("usage: l1-adapter LOOPBACK_ADDR CONFIG_JSON [DROP_MINE_ANSWER_NUMBER]".into());
    }
    let config: Config = serde_json::from_reader(File::open(&args[2])?)?;
    let rpc = TraceRpc {
        anvil: AnvilRpc::new(args[1].parse()?)?,
        mine_count: 0,
        drop_mine_answer: args.get(3).map(|arg| arg.parse()).transpose()?,
    };
    let mut driver = Driver::new(config, rpc).map_err(|error| format!("{error:?}"))?;
    let mut stdout = io::stdout().lock();
    writeln!(
        stdout,
        "{}",
        json!({"startup_rpc": std::mem::take(driver.rpc_evidence())})
    )?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let command: Command = serde_json::from_str(&line)?;
        let reply = driver.apply(command);
        let rpc = std::mem::take(driver.rpc_evidence());
        writeln!(stdout, "{}", json!({"reply": reply, "rpc": rpc}))?;
    }
    Ok(())
}
