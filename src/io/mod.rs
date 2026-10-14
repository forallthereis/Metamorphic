use std::path::Path;

use lp_parser_rs::LpProblem;

pub mod cli;
pub mod log;
pub mod lp;
pub mod mps;
pub mod sol;





pub(crate) trait Reader {
    fn read(&self, file_path: &Path) -> crate::error::Result<LpProblem>;
}
