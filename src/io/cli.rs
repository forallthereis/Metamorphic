use std::{fmt::Display, path::{Path, PathBuf}, thread};

use clap::Parser;
use inquire::{Select, Text, set_global_render_config, ui::{Color, RenderConfig, Styled}, validator::Validation};





pub(crate) fn inquire_config() -> crate::Config {
    let mut cli_config = RenderConfig::default();
    cli_config.unhighlighted_option_prefix = Styled::new("    ");
    cli_config.highlighted_option_prefix = Styled::new("   → ").with_fg(Color::LightCyan);
    set_global_render_config(cli_config);

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

    let thread_count_source = Select::new(
            "How should the number of worker threads be determined?",
            vec![
                ThreadCountSource::System,
                ThreadCountSource::User,
            ],
        )
        .prompt()
        .unwrap();

    let threads_count: usize = match thread_count_source {
        ThreadCountSource::System => thread::available_parallelism().unwrap().get() - 1,
        ThreadCountSource::User => Text::new("Number of threads to use:")
            .with_validator(|input: &str| {
                match input.parse::<usize>() {
                    Ok(_) => Ok(Validation::Valid),
                    Err(_) => Ok(Validation::Invalid("This value must be a non-negative integer.".into())),
                }
            })
            .prompt()
            .unwrap()
            .parse()
            .unwrap(),
    };

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

    crate::Config { input_file_path, threads_count, log_format, log_destination }
}



#[derive(Parser)]
#[command(
    name = "Metamorphic",
    version,
    about = "Tree-decomposition-based solver for integer linear programs",
)]
pub(crate) struct CLInput {
    /// Path to the input file. Supported formats: LP, MPS.
    pub input_file_path: PathBuf,
    /// Number of worker threads to be used for solving the given optimisation problem instance.
    /// This must be either a positive integer or 'tcauto'.
    /// If the value 'tcauto' is given, Metamorphic will decide automatically how many threads to use.
    pub thread_count: CLInputThreadCount,
    /// Where the logs should be written.
    /// This must be either 'logfile', 'logstdout' or 'nolog'.
    /// If the value 'logfile' is given, logs will be saved in a file called '<INPUT FILE NAME>.log' that will
    /// be created in the folder of the input file.
    /// Existing file with the same name will be overwritten.
    /// If the value 'logstdout' is given, logs will be printed into the standard output channel (i.e. this console).
    /// If the value 'nolog' is given, no logs will be made.
    pub log_destination: crate::io::log::LogDestination,
    /// The format of logs.
    /// This must be either 'logtxt' or 'logmd'.
    /// If the value 'logtxt' is given, logs will be formatted as plain text.
    /// If the value 'logmd' is given, logs will be formatted in Markdown.
    pub log_format: crate::io::log::LogFormat,
}



enum ThreadCountSource {
    System,
    User,
}

impl Display for ThreadCountSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadCountSource::System => write!(f, "[tcauto] Derive automatically"),
            ThreadCountSource::User => write!(f, "Set manually"),
        }
    }
}



#[derive(Clone)]
pub enum CLInputThreadCount {
    System,
    User(usize),
}

impl std::str::FromStr for CLInputThreadCount {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "tcauto" {
            return Ok(CLInputThreadCount::System);
        }

        let count: usize = value
            .parse()
            .map_err(|_| {
                "Expected 'tcauto' or a positive integer."
            })?;

        if count == 0 {
            return Err("Thread count must be greater than zero.");
        }

        Ok(CLInputThreadCount::User(count))
    }
}
