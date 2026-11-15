fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: native-worker ENDPOINT LAUNCH_JSON");
        return std::process::ExitCode::FAILURE;
    }
    match poc_native_runtime::native_replica::run_worker(&args[0], std::path::Path::new(&args[1])) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("native-worker: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
