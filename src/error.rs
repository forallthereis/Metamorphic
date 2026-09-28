pub(crate) type Result<T> = std::result::Result<T, MetamorhicError>;



#[derive(Debug)]
pub(crate) enum MetamorhicError {
    ForbiddenCommand(String),
    OutOfMemory(String),
}

#[macro_export]
macro_rules! return_error {
    ($error_variant:ident, $message_stem:literal, $($message_value:expr),*) => {
        return Err(
            crate::error::MetamorhicError::$error_variant(format!(
                $message_stem,
                $($message_value),*
            ))
        );
    };
}

impl crate::io::log::Loggable for MetamorhicError {

    fn log_content(&self) -> String {

        match self {
            MetamorhicError::ForbiddenCommand(message) => format!("Metamorphic encountered a forbidden command. Specifically, {}", message),
            MetamorhicError::OutOfMemory(message) => format!("The system ran out of memory. Specifically, {}", message),
        }

    }

}
