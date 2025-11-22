use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Quiz {
    pub title: String,
    pub description: Option<String>,
    pub shuffle_answers: bool,
    pub questions: Vec<Question>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub title: String,
    pub prompt: String,
    pub question_type: QuestionType,
    pub points: f32,
    pub answers: Vec<Answer>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    MultipleChoice,
    MultipleAnswers,
    TrueFalse,
    ShortAnswer,
    Essay,
    FileUpload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Answer {
    pub text: String,
    pub is_correct: bool,
    pub feedback: Option<String>,
}
