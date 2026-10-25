use std::path::Path;

use lp_parser_rs::{LpProblem, parser};





pub struct Reader;

impl Reader {
    pub fn read(&self, file_path: &Path) -> crate::error::Result<LpProblem> {
        let lp_problem_contents = match parser::parse_file(&file_path) {
            Ok(lp_problem_contents) => lp_problem_contents,
            Err(e) => crate::return_error!(
                Generic,
                "{}",
                e.to_string()
            ),
        };

        let lp_problem_result = match file_path.extension().unwrap().to_ascii_lowercase().to_str().unwrap() {
            "lp" => LpProblem::parse(&lp_problem_contents),
            "mps" => LpProblem::parse_mps(&lp_problem_contents),
            _ => unreachable!(),
        };

        let lp_problem = match lp_problem_result {
            Ok(lp_problem) => lp_problem,
            Err(e) => crate::return_error!(
                Generic,
                "{}",
                e.to_string()
            ),
        };

        Ok(lp_problem)
    }
}
