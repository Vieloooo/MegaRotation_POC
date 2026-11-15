#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match poc_native_runtime::native_hub::run_hub(&args).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("native-hub: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
