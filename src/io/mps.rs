use std::path::Path;

use lp_parser_rs::{LpProblem, parser};





pub(crate) struct MPSReader;

impl crate::io::Reader for MPSReader {

    fn read(&self, file_path: &Path) -> crate::error::Result<LpProblem> {

        let lp_problem_contents = match parser::parse_file(&file_path) {
            Ok(lp_problem_contents) => lp_problem_contents,
            Err(e) => crate::return_error!(
                Generic,
                "{}",
                e.to_string()
            ),
        };

        let lp_problem = match LpProblem::parse_mps(&lp_problem_contents) {
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
