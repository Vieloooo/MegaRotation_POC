fn main() -> std::process::ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match poc_native_runtime::m1_smoke::run(&args) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("m1-smoke: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
