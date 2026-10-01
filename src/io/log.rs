use std::io::Write;

pub(crate) trait Loggable {
    fn log_content(&self) -> String;
}



pub struct Logger {
    format: LogFormat,
    output_file: Box<dyn Write>,
}

impl Logger {

    fn log_info(message: String) {
        todo!();
    }

    fn log_error(message: String) {
        todo!();
    }

}



enum LogFormat {
    PlainText,
    Markdown,
}
