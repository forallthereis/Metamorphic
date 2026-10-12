use std::{fmt::Display, fs::File, io::Write};

use chrono::{Datelike, Local, Timelike};





pub(crate) trait Loggable {
    fn log_content(self) -> String;
}

impl<S> Loggable for S
where
    S: Into<String>
{
    #[inline(always)]
    fn log_content(self) -> String {
        self.into()
    }
}



pub(crate) struct Logger {
    format: LogFormat,
    log_writer: Box<dyn Write>,
}

impl Logger {

    pub(crate) fn log_info<Item>(&mut self, item: Item)
    where
        Item: Loggable,
    {

        if self.format == LogFormat::None {
            return;
        }

        let now = Local::now();
        let timestamp_string = match self.format {
            LogFormat::None => unreachable!(),
            LogFormat::PlainText => format!(
                "{:04}-{:02}-{:02} {:02}-{:02}-{:02}|INFO|",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            ),
            LogFormat::Markdown => format!(
                "|{:04}-{:02}-{:02} {:02}-{:02}-{:02}|INFO|",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            ),
        };

        let content = self.fmt_log_content(item.log_content());

        writeln!(self.log_writer, "{}{}", timestamp_string, content).expect("Log destination is no longer accessible.");

    }

    pub(crate) fn log_error<Item>(&mut self, item: Item)
    where
        Item: Loggable,
    {

        if self.format == LogFormat::None {
            return;
        }

        let now = Local::now();
        let timestamp_string = match self.format {
            LogFormat::None => unreachable!(),
            LogFormat::PlainText => format!(
                "{:04}-{:02}-{:02} {:02}-{:02}-{:02}|FAIL|",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            ),
            LogFormat::Markdown => format!(
                "|{:04}-{:02}-{:02} {:02}-{:02}-{:02}|ERROR|",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            ),
        };

        let content = self.fmt_log_content(item.log_content());

        writeln!(self.log_writer, "{}{}", timestamp_string, content).expect("Log destination is no longer accessible.");

    }

    pub(crate) fn new(config: &crate::Config) -> Self {

        let mut log_file_path = config.input_file_path.clone();
        log_file_path.set_extension("log");

        let mut log_writer: Box<dyn Write> = match config.log_destination {
            LogDestination::None | LogDestination::StdOut => Box::new(std::io::stdout()),
            LogDestination::File => Box::new(File::create(log_file_path).expect("Couldn't create a log file.")),
        };

        let format = match config.log_destination {
            LogDestination::None => LogFormat::None,
            _ => config.log_format.clone(),
        };

        match format {
            LogFormat::Markdown => {writeln!(log_writer, "|Timestamp|Message type|Message|\n|:-----:|:-----:|-----|").expect("Log destination is no longer accessible.")},
            _ => (),
        }

        Logger { format, log_writer }

    }

    fn fmt_log_content(&self, message: String) -> String {

        match self.format {

            LogFormat::None => unreachable!(),

            LogFormat::PlainText => {

                const MAX_LENGTH: usize = 80;

                let mut lines = Vec::new();
                let mut current = String::new();

                let mut chars = message.chars().peekable();

                while let Some(ch) = chars.peek().copied() {
                    if ch.is_alphanumeric() {
                        // Read a complete word
                        let mut word = String::new();

                        while let Some(ch) = chars.peek().copied() {
                            if ch.is_alphanumeric() || ch == '.' {
                                word.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }

                        // Flush any existing chunk before a word that cannot fit
                        if word.chars().count() > MAX_LENGTH {

                            if !current.is_empty() {
                                lines.push(std::mem::take(&mut current));
                            }

                            let mut chunk = String::new();

                            for ch in word.chars() {

                                chunk.push(ch);

                                if chunk.chars().count() == MAX_LENGTH {
                                    lines.push(std::mem::take(&mut chunk));
                                }
                            }

                            if !chunk.is_empty() {
                                current = chunk;
                            }

                        } else {

                            let needed = word.chars().count();

                            if !current.is_empty() && current.chars().count() + needed > MAX_LENGTH {
                                lines.push(std::mem::take(&mut current));
                            }

                            current.push_str(&word);

                        }
                    } else {
                        // Read a run of non-word characters.
                        let mut separator = String::new();

                        while let Some(ch) = chars.peek().copied() {

                            if ch.is_alphanumeric() {
                                break;
                            }

                            separator.push(ch);
                            chars.next();

                        }

                        // Preserve separators, splitting them if necessary.
                        for ch in separator.chars() {

                            if current.chars().count() == MAX_LENGTH {
                                lines.push(std::mem::take(&mut current));
                            }

                            if ch == '\n' {
                                lines.push(std::mem::take(&mut current));
                                continue;
                            }

                            current.push(ch);

                        }
                    }
                }

                if !current.is_empty() {
                    lines.push(current);
                }

                lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, line)| {
                        if i == 0 {
                            line
                        } else {
                            format!("                   :    :{}", line)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")

            },

            LogFormat::Markdown => { format!("{}|", message.replace("\n", " ")) },

        }

    }

}



#[derive(Clone, PartialEq, Eq)]
pub(crate) enum LogFormat {
    None,
    PlainText,
    Markdown,
}

impl Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => unreachable!(),
            Self::PlainText => write!(f, "Plain text"),
            Self::Markdown => write!(f, "Markdown"),
        }
    }
}



pub(crate) enum LogDestination {
    None,
    StdOut,
    File,
}

impl Display for LogDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Nowhere, i.e. disable logs"),
            Self::StdOut => write!(f, "Standard output channel, i.e. stdout"),
            Self::File => write!(f, "File on a disk"),
        }
    }
}
