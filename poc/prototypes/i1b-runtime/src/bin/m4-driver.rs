fn main() -> Result<(), Box<dyn std::error::Error>> {
    poc_native_runtime::m1_smoke::run_m4_driver(&std::env::args().skip(1).collect::<Vec<_>>())
}
