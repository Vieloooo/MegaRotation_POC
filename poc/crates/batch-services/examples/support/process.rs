//! Single-threaded Linux scenario process fault harness. Closed public-reader
//! capabilities survive into the test child through fork's memory snapshot.

use batch_services::{
    FaultPoint, crypto::hex, frame::commitment, transaction::PreparedTransaction,
};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::Path,
    time::Duration,
};

thread_local! { static TARGET: Cell<Option<FaultPoint>> = const { Cell::new(None) }; }
const INJECTED: i32 = 91;

pub fn inject(point: FaultPoint) {
    if TARGET.with(|target| target.get() == Some(point)) {
        // The exact production persistence hook already completed. Do not run
        // destructors or flush unrelated buffers during the injected crash.
        unsafe { libc::_exit(INJECTED) }
    }
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let bytes = fs::read(entry.path()).unwrap();
            (
                entry.file_name().into_string().unwrap(),
                hex(commitment(&bytes).as_bytes()),
            )
        })
        .collect()
}

pub fn crash_at(root: &Path, role: &str, point: FaultPoint, op: impl FnOnce()) -> Value {
    let before = snapshot(root);
    // This executable has one thread. The fault child closes over immutable test
    // inputs and opens a single role root after fork. It never touches Anvil.
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
    if pid == 0 {
        TARGET.with(|target| target.set(Some(point)));
        op();
        unsafe { libc::_exit(92) }
    }
    let exit = child_status(pid);
    assert!(
        libc::WIFEXITED(exit) && libc::WEXITSTATUS(exit) == INJECTED,
        "{role}/{point:?}: child exit status {exit}"
    );
    json!({"role":role,"boundary":point,"pid":pid,"exit_code":INJECTED,"journal_keccak_before":before,
        "journal_keccak_after":snapshot(root)})
}

pub fn crash_after_send(
    root: &Path,
    open: impl FnOnce() -> (batch_services::batcher::Batcher, PreparedTransaction),
    send: impl FnOnce(&PreparedTransaction),
) -> Value {
    let before = snapshot(root);
    let (parent, mut child) = UnixStream::pair().unwrap();
    parent
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0, "fork failed: {}", std::io::Error::last_os_error());
    if pid == 0 {
        drop(parent);
        let (_actor, tx) = open();
        serde_json::to_writer(&mut child, &tx).unwrap();
        child.write_all(b"\n").unwrap();
        child.flush().unwrap();
        // The live service waits for an explicit reply. It does not get one.
        let mut reply = String::new();
        let _ = BufReader::new(child).read_line(&mut reply);
        unsafe { libc::_exit(93) }
    }
    drop(child);
    let mut child = ChildGuard(pid);
    let mut reader = BufReader::new(parent);
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).unwrap() > 0,
        "outbox child ended before output"
    );
    let tx: PreparedTransaction = serde_json::from_str(&line).unwrap();
    send(&tx);
    let exit = child.kill();
    assert!(libc::WIFSIGNALED(exit) && libc::WTERMSIG(exit) == libc::SIGKILL);
    json!({"role":"batcher","boundary":"raw_sent_before_reply","pid":pid,"signal":"SIGKILL",
        "nonce":tx.nonce(),"tx_hash":hex(tx.hash().as_bytes()),"raw":hex(tx.raw()),
        "journal_keccak_before":before,"journal_keccak_after":snapshot(root)})
}

fn child_status(pid: i32) -> i32 {
    loop {
        let mut status = 0;
        let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
        if waited == pid {
            return status;
        }
        assert_eq!(
            std::io::Error::last_os_error().kind(),
            std::io::ErrorKind::Interrupted
        );
    }
}

struct ChildGuard(i32);
impl ChildGuard {
    fn kill(&mut self) -> i32 {
        let pid = std::mem::replace(&mut self.0, 0);
        unsafe { libc::kill(pid, libc::SIGKILL) };
        child_status(pid)
    }
}
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.0 > 0 {
            self.kill();
        }
    }
}

pub fn recovered(case: &mut Value, root: &Path) {
    case["reopen_pid"] = json!(std::process::id());
    case["journal_keccak_reopened"] = json!(snapshot(root));
}
