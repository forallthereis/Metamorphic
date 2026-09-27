pub(crate) type Result<T> = std::result::Result<T, MetamorhicError>;



#[derive(Debug)]
pub(crate) enum MetamorhicError {
    ForbiddenCommand(String),
    OutOfMemory(String),
}

impl crate::io::log::Loggable for MetamorhicError {

    fn log_content(&self) -> String {

        match self {
            MetamorhicError::ForbiddenCommand(message) => format!("Metamorphic encountered a forbidden command. Specifically, {}", message),
            MetamorhicError::OutOfMemory(message) => format!("The system ran out of memory. Specifically, {}", message),
        }

    }

}
