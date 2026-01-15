fn main() {
    if let Err(err) = mdnotes::run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
