use std::path::{Path, PathBuf};

use inquire::{Select, Text, validator::Validation};





pub(crate) fn inquire_config() -> crate::Config {

    let input_file_path = PathBuf::from(
        Text::new("Path to the input file:")
            .with_validator(|input: &str| {
                let path = Path::new(input);

                if !path.exists() {
                    return Ok(Validation::Invalid(
                        "The specified path does not exist.".into(),
                    ));
                }

                if !path.is_file() {
                    return Ok(Validation::Invalid(
                        "The specified path is not a file.".into(),
                    ));
                }

                match path.extension() {
                    Some(extension) => {
                        let extension = match extension.to_ascii_lowercase().as_os_str().to_str() {
                            Some(extension) => extension.to_string(),
                            None => return Ok(Validation::Invalid(
                                "File extension isn't supported.".into(),
                            )),
                        };
                        if !crate::SUPPORTED_INPUT_FILE_EXTENSIONS.contains(&extension.as_ref()) {
                            return Ok(Validation::Invalid(
                                "File extension isn't supported.".into(),
                            ));
                        }
                    },
                    None => return Ok(Validation::Invalid(
                        "File extension isn't supported.".into(),
                    )),
                }

                Ok(Validation::Valid)
            })
            .prompt()
            .unwrap()
    );

    let log_destination = Select::new(
            "Where should the logs be written?",
            vec![
                crate::io::log::LogDestination::File,
                crate::io::log::LogDestination::StdOut,
                crate::io::log::LogDestination::None,
            ],
        )
        .prompt()
        .unwrap();

    let log_format = match log_destination {

        crate::io::log::LogDestination::File | crate::io::log::LogDestination::StdOut =>
            Select::new(
                "What format should the logs use?",
                vec![
                    crate::io::log::LogFormat::PlainText,
                    crate::io::log::LogFormat::Markdown,
                ],
            )
            .prompt()
            .unwrap(),

        crate::io::log::LogDestination::None => crate::io::log::LogFormat::PlainText,

    };

    crate::Config { input_file_path, log_format, log_destination }

}
