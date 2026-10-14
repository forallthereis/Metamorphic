use std::{fs::File, io::Write};

use itertools::Itertools;

pub(crate) fn write(config: &crate::Config, solution: &crate::opt::VariableAssignment, obj_value: f64) -> crate::error::Result<()> {
    let mut sol_file_path = config.input_file_path.clone();
    sol_file_path.set_extension("sol");

    let mut sol_file = match File::create(&sol_file_path) {
        Ok(file) => file,
        Err(_) => crate::return_error!(
            BadOutput,
            "output file creation failed. The reason can be that the folder doesn't exist anymore or that Metamorphic doesn't have write permissions for it.",
        )
    };

    writeln!(sol_file, "# Objective value = {}", obj_value).unwrap();

    for (var_name, var_value) in solution.iter().sorted_by_key(|item| item.0) {
        writeln!(sol_file, "{} {}", var_name, var_value).unwrap();
    }

    Ok(())
}
