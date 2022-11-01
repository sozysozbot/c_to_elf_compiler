use std::fmt;

#[derive(Debug, Clone)]
pub struct AppError {
    pub message: String,
    pub input: String,
    pub pos: usize,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.input)?;
        write!(f, "{}^ {}", " ".repeat(self.pos), self.message)?;
        Ok(())
    }
}

impl std::error::Error for AppError {}
