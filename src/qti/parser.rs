use anyhow::{Context, Result};
use crate::qti::model::{Quiz, Question, QuestionType, Answer};
use serde::Deserialize;

#[derive(Deserialize)]
struct FrontMatter {
    title: String,
    description: Option<String>,
    #[serde(default)]
    shuffle_answers: bool,
}

pub fn parse_quiz(content: &str) -> Result<Quiz> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid file format: Missing YAML front matter");
    }

    let front_matter_str = parts[1];
    let markdown_content = parts[2];

    let front_matter: FrontMatter = serde_yaml::from_str(front_matter_str)
        .context("Failed to parse YAML front matter")?;

    let questions = parse_questions(markdown_content)?;

    Ok(Quiz {
        title: front_matter.title,
        description: front_matter.description,
        shuffle_answers: front_matter.shuffle_answers,
        questions,
    })
}

fn parse_questions(content: &str) -> Result<Vec<Question>> {
    let mut questions = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();
    let mut in_question_block = false;

    for line in content.lines() {
        if line.trim().starts_with(":::{.question") {
            if in_question_block {
                // This shouldn't happen in valid format, but handle it by closing previous
                if !current_lines.is_empty() {
                    questions.push(parse_single_question(&current_lines)?);
                    current_lines.clear();
                }
            }
            in_question_block = true;
            current_lines.push(line);
        } else if line.trim() == ":::" && in_question_block {
            current_lines.push(line);
            questions.push(parse_single_question(&current_lines)?);
            current_lines.clear();
            in_question_block = false;
        } else if in_question_block {
            current_lines.push(line);
        }
    }

    Ok(questions)
}

fn parse_single_question(lines: &[&str]) -> Result<Question> {
    // Parse attributes from the first line: :::{.question type=multiple_choice points=1}
    let header = lines[0].trim();
    let attributes_str = header
        .trim_start_matches(":::{.question")
        .trim_end_matches('}')
        .trim();

    let mut question_type = QuestionType::MultipleChoice;
    let mut points = 1.0;
    let mut title = String::new(); // Title is optional or derived

    for attr in attributes_str.split_whitespace() {
        if let Some((key, value)) = attr.split_once('=') {
            match key {
                "type" => {
                    question_type = match value {
                        "multiple_choice" => QuestionType::MultipleChoice,
                        "multiple_answers" => QuestionType::MultipleAnswers,
                        "true_false" => QuestionType::TrueFalse,
                        "short_answer" => QuestionType::ShortAnswer,
                        "essay" => QuestionType::Essay,
                        "file_upload" => QuestionType::FileUpload,
                        _ => QuestionType::MultipleChoice,
                    };
                }
                "points" => {
                    if let Ok(val) = value.parse::<f32>() {
                        points = val;
                    }
                }
                "title" => {
                    title = value.replace("_", " ").to_string(); // Simple handling for now
                }
                _ => {}
            }
        }
    }

    let mut prompt_lines = Vec::new();
    let mut answers = Vec::new();
    let mut parsing_answers = false;

    // Skip first and last line (:::)
    for line in &lines[1..lines.len()-1] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("- [") {
            parsing_answers = true;
            let is_correct = trimmed.starts_with("- [x]");
            let text = trimmed[5..].trim().to_string();
            answers.push(Answer {
                text,
                is_correct,
                feedback: None,
            });
        } else {
            if !parsing_answers {
                prompt_lines.push(*line);
            }
        }
    }
    
    // If title is empty, use truncated prompt
    if title.is_empty() {
        let prompt = prompt_lines.join(" ");
        title = prompt.chars().take(50).collect::<String>();
        if prompt.len() > 50 {
            title.push_str("...");
        }
    }

    Ok(Question {
        title,
        prompt: prompt_lines.join("\n").trim().to_string(),
        question_type,
        points,
        answers,
    })
}
