use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum Token {
    Text(String),
    LatexMath(String),
    Chemistry(String),
    Image { src: String, alt: String },
    Music(String),
    Code { lang: String, content: String },
}

pub fn lex_content(text: &str) -> Vec<Token> {
    // Matches $$...$$, $...$, \ce{...}, ![...](...), ```musicxml...```, or ```lang...```
    // Group 1: $$...$$
    // Group 2: $...$
    // Group 3: \ce{...}
    // Group 4: ![...](...) -> alt
    // Group 5: ![...](...) -> src
    // Group 6: ```musicxml...```
    // Group 7: ```lang...``` -> lang
    // Group 8: ```lang...``` -> content
    let re = Regex::new(r"(\$\$[\s\S]*?\$\$)|(\$[\s\S]*?\$) |(\\ce\{[\s\S]*?\})|(!\[(.*?)\]\((.*?)\))|(```musicxml\n([\s\S]*?)```)|(```(\w*)\n([\s\S]*?)```)").unwrap();
    let mut tokens = Vec::new();
    let mut last_end = 0;

    for cap in re.captures_iter(text) {
        let m = cap.get(0).unwrap();
        
        // Push preceding text if any
        if m.start() > last_end {
            tokens.push(Token::Text(text[last_end..m.start()].to_string()));
        }

        if let Some(block) = cap.get(1) {
            // Block Math $$...$$
            let latex = block.as_str().trim_matches('$').trim();
            tokens.push(Token::LatexMath(latex.to_string()));
        } else if let Some(block) = cap.get(2) {
            // Inline Math $...$
            let latex = block.as_str().trim_matches('$').trim();
            tokens.push(Token::LatexMath(latex.to_string()));
        } else if let Some(block) = cap.get(3) {
            // Chemistry \ce{...}
            let content = block.as_str();
            let inner = &content[4..content.len()-1];
            tokens.push(Token::Chemistry(inner.to_string()));
        } else if let Some(_) = cap.get(4) {
            // Image ![alt](src)
            let alt = cap.get(5).map_or("", |m| m.as_str()).to_string();
            let src = cap.get(6).map_or("", |m| m.as_str()).to_string();
            tokens.push(Token::Image { src, alt });
        } else if let Some(block) = cap.get(7) {
            // Music ```musicxml...```
            let content = cap.get(8).map_or("", |m| m.as_str());
            tokens.push(Token::Music(content.to_string()));
        } else if let Some(block) = cap.get(9) {
            // Code ```lang...```
            let lang = cap.get(10).map_or("", |m| m.as_str()).to_string();
            let content = cap.get(11).map_or("", |m| m.as_str()).to_string();
            tokens.push(Token::Code { lang, content });
        }
        
        last_end = m.end();
    }

    // Push remaining text
    if last_end < text.len() {
        tokens.push(Token::Text(text[last_end..].to_string()));
    }

    tokens
}
