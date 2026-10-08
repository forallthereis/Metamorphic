use std::{env, path::PathBuf};

mod error;
mod io;
mod opt;





const F64_EQ_ULPS: i64 = 2;
const SUPPORTED_INPUT_FILE_EXTENSIONS: [&str; 1] = ["lp"];



fn main() {

    let args: Vec<_> = env::args().collect();

    let config = if args.len() == 1 {
        crate::io::cli::inquire_config()
    } else {
        todo!();
    };

}



struct Config {
    pub input_file_path: PathBuf,
    pub log_format: crate::io::log::LogFormat,
    pub log_destination: crate::io::log::LogDestination,
}
