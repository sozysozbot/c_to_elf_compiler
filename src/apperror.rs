use std::fmt;

#[derive(Debug, Clone)]
pub struct AppError {
    pub message: String,
    pub input: String,
    pub filename: String,
    pub pos: usize,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut line = 0;
        let mut col = 0;
        let mut line_start = 0;
        for (i, c) in self.input.chars().take(self.pos).enumerate() {
            if c == '\n' {
                line += 1;
                col = 0;
                line_start = i + 1;
            } else {
                col += 1;
            }
        }
        writeln!(f, "{}:{}:{}", self.filename, line + 1, col + 1,)?;
        writeln!(
            f,
            "{}",
            self.input[line_start..]
                .chars()
                .take_while(|c| *c != '\n')
                .collect::<String>()
        )?;
        write!(f, "{}^ {}", " ".repeat(col), self.message)?;
        Ok(())
    }
}

impl std::error::Error for AppError {}
