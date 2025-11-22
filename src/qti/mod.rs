pub mod model;
pub mod parser;
pub mod generator;
mod lexer;

use anyhow::Result;
use std::path::Path;

pub fn convert_to_qti(input_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input_path)?;
    let quiz = parser::parse_quiz(&content)?;
    generator::generate_qti(&quiz, input_path)?;
    Ok(())
}
