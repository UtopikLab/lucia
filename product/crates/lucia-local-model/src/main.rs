use std::path::PathBuf;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 6
        || arguments[1] != "serve"
        || arguments[2] != "--socket"
        || arguments[4] != "--model"
    {
        eprintln!("Usage: lucia-local-model serve --socket ABSOLUTE_PATH --model MODEL_NAME");
        std::process::exit(2);
    }
    if let Err(error) = lucia_local_model::serve(&PathBuf::from(&arguments[3]), &arguments[5]) {
        eprintln!("LOCAL_MODEL_ERROR: {error}");
        std::process::exit(1);
    }
}
