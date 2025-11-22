use anyhow::Result;
use crate::qti::model::{Quiz, Question, QuestionType};
use quick_xml::events::{BytesDecl, Event};
use quick_xml::Writer;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;
use uuid::Uuid;
use zip::write::FileOptions;

use std::collections::HashSet;
use std::io::Read;

pub fn generate_qti(quiz: &Quiz, output_path: &Path) -> Result<()> {
    let output_file_name = output_path.file_stem().unwrap().to_str().unwrap();
    let zip_filename = format!("{}.zip", output_file_name);
    let zip_path = output_path.with_file_name(zip_filename);
    
    let file = File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Collect resources (images)
    let base_dir = output_path.parent().unwrap();
    let resources = collect_resources(quiz, base_dir);

    // 1. imsmanifest.xml
    let manifest_xml = generate_manifest(quiz, &resources)?;
    zip.start_file("imsmanifest.xml", options)?;
    zip.write_all(manifest_xml.as_bytes())?;

    // 2. assessment.xml
    let assessment_xml = generate_assessment(quiz)?;
    let _assessment_id = format!("assessment_{}", Uuid::new_v4());
    
    zip.start_file("assessment.xml", options)?;
    zip.write_all(assessment_xml.as_bytes())?;

    // 3. Copy resources
    let base_dir = output_path.parent().unwrap();
    for resource in &resources {
        let src_path = base_dir.join(resource);
        if src_path.exists() {
            let file_name = src_path.file_name().unwrap().to_str().unwrap();
            let dest_path = format!("images/{}", file_name);
            zip.start_file(dest_path, options)?;
            
            let mut f = File::open(&src_path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else {
            eprintln!("Warning: Image not found: {:?}", src_path);
        }
    }

    zip.finish()?;
    
    println!("Generated QTI zip at: {:?}", zip_path);
    Ok(())
}


fn generate_manifest(_quiz: &Quiz, resources: &HashSet<String>) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // Root element
    let mut root = quick_xml::events::BytesStart::new("manifest");
    root.push_attribute(("identifier", "manifest"));
    root.push_attribute(("xmlns", "http://www.imsglobal.org/xsd/imscp_v1p1"));
    writer.write_event(Event::Start(root))?;

    // Metadata (optional, minimal)
    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("metadata")))?;
    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("schema")))?;
    writer.write_event(Event::Text(quick_xml::events::BytesText::new("IMS Content")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("schema")))?;
    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("schemaversion")))?;
    writer.write_event(Event::Text(quick_xml::events::BytesText::new("1.1")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("schemaversion")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("metadata")))?;

    // Organizations (empty)
    writer.write_event(Event::Empty(quick_xml::events::BytesStart::new("organizations")))?;

    // Resources
    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("resources")))?;
    
    let mut resource = quick_xml::events::BytesStart::new("resource");
    resource.push_attribute(("identifier", "assessment"));
    resource.push_attribute(("type", "imsqti_test_xmlv2p1"));
    resource.push_attribute(("href", "assessment.xml")); 
    
    
    writer.write_event(Event::Start(resource))?;

    let mut file = quick_xml::events::BytesStart::new("file");
    file.push_attribute(("href", "assessment.xml"));
    writer.write_event(Event::Empty(file))?;
    
    for res in resources {
        let file_name = Path::new(res).file_name().unwrap().to_str().unwrap();
        let mut img_file = quick_xml::events::BytesStart::new("file");
        img_file.push_attribute(("href", format!("images/{}", file_name).as_str()));
        writer.write_event(Event::Empty(img_file))?;
    }

    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("resource")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("resources")))?;

    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("manifest")))?;

    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

// ... (generate_assessment and others remain mostly same)

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::process::Command;

// ...

fn collect_resources(quiz: &Quiz, input_dir: &Path) -> HashSet<String> {
    let mut resources = HashSet::new();
    
    for question in &quiz.questions {
        extract_resources_from_text(&question.prompt, &mut resources, input_dir);
        for answer in &question.answers {
            extract_resources_from_text(&answer.text, &mut resources, input_dir);
        }
    }
    
    resources
}

fn extract_resources_from_text(text: &str, resources: &mut HashSet<String>, input_dir: &Path) {
    let tokens = lex_content(text);
    for token in tokens {
        match token {
            Token::Image { src, .. } => {
                resources.insert(src);
            }
            Token::Music(content) => {
                if check_verovio_installed() {
                    if let Ok(filename) = generate_verovio_svg(&content, input_dir) {
                        resources.insert(filename);
                    }
                }
            }
            _ => {}
        }
    }
}

fn generate_verovio_svg(content: &str, output_dir: &Path) -> Result<String> {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = hasher.finish();
    let filename = format!("music_{}.svg", hash);
    let output_path = output_dir.join(&filename);
    
    // Only generate if it doesn't exist to save time
    if !output_path.exists() {
        // Write content to temp file
        let temp_input = output_dir.join(format!("temp_{}.musicxml", hash));
        std::fs::write(&temp_input, content)?;
        
        // Run verovio
        let status = Command::new("verovio")
            .arg("-o")
            .arg(&output_path) 
            .arg(&temp_input)
            .output()?;
            
        // Clean up temp input
        let _ = std::fs::remove_file(temp_input);
        
        if !status.status.success() {
            eprintln!("Error running verovio: {:?}", String::from_utf8_lossy(&status.stderr));
            return Err(anyhow::anyhow!("Verovio failed"));
        }
    }
    
    Ok(filename)
}

// ...

use syntect::html::{highlighted_html_for_string, IncludeBackground};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use std::sync::OnceLock;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(|| SyntaxSet::load_defaults_newlines())
}

fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(|| ThemeSet::load_defaults())
}

// ...

fn write_content(writer: &mut Writer<Cursor<Vec<u8>>>, text: &str) -> Result<()> {
    let tokens = lex_content(text);
    for token in tokens {
        match token {
            Token::Text(content) => {
                writer.write_event(Event::Text(quick_xml::events::BytesText::new(&content)))?;
            }
            Token::LatexMath(latex) => {
                write_latex_mathml(writer, &latex)?;
            }
            Token::Chemistry(chem) => {
                let latex = transpile_mhchem(&chem);
                write_latex_mathml(writer, &latex)?;
            }
            Token::Image { src, alt } => {
                if alt.trim().is_empty() {
                    eprintln!("Warning: Accessibility issue - Image '{}' has missing or empty alt text.", src);
                }
                
                let file_name = Path::new(&src).file_name().unwrap().to_str().unwrap();
                let mut img = quick_xml::events::BytesStart::new("img");
                img.push_attribute(("src", format!("images/{}", file_name).as_str()));
                img.push_attribute(("alt", alt.as_str()));
                writer.write_event(Event::Empty(img))?;
            }
            Token::Music(musicxml) => {
                if check_verovio_installed() {
                    let mut hasher = DefaultHasher::new();
                    musicxml.hash(&mut hasher);
                    let hash = hasher.finish();
                    let filename = format!("music_{}.svg", hash);
                    
                    let mut img = quick_xml::events::BytesStart::new("img");
                    img.push_attribute(("src", format!("images/{}", filename).as_str()));
                    img.push_attribute(("alt", "Music Notation"));
                    writer.write_event(Event::Empty(img))?;
                } else {
                    eprintln!("Warning: Verovio not found. MusicXML block will be rendered as code.");
                    eprintln!("To enable music notation, please install Verovio: https://verovio.org");
                    
                    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("pre")))?;
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(&musicxml)))?;
                    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("pre")))?;
                }
            }
            Token::Code { lang, content } => {
                let ss = get_syntax_set();
                let ts = get_theme_set();
                // Use InspiredGitHub as a light theme that works well on white backgrounds
                let theme = &ts.themes["InspiredGitHub"];
                
                let syntax = ss.find_syntax_by_token(&lang)
                    .unwrap_or_else(|| ss.find_syntax_plain_text());
                
                // Generate HTML with inline styles
                let html = highlighted_html_for_string(&content, ss, syntax, theme)?;
                
                // Write the HTML directly. Since highlighted_html_for_string returns a full <pre> block,
                // we need to parse it or write it as raw bytes if we trust it.
                // However, writing raw bytes into quick-xml is tricky if we want to maintain structure.
                // But wait, quick-xml's Writer writes events.
                // We can parse the generated HTML and write events, OR we can write a CDATA block?
                // No, CDATA might not be rendered by the browser.
                // We should parse the HTML and write it as events to be safe and correct.
                
                // Actually, for simplicity and robustness, let's use the Reader to parse the HTML string
                // and pipe it to the Writer, just like we did for MathML.
                let mut reader = quick_xml::Reader::from_str(&html);
                
                loop {
                    match reader.read_event() {
                        Ok(Event::Eof) => break,
                        Ok(e) => writer.write_event(e)?,
                        Err(e) => return Err(anyhow::anyhow!("Error parsing generated HTML: {}", e)),
                    }
                }
            }
        }
    }
    Ok(())
}

fn check_verovio_installed() -> bool {
    let output = std::process::Command::new("verovio")
        .arg("--version")
        .output();
        
    match output {
        Ok(o) => {
            if !o.status.success() {
                eprintln!("Verovio check failed with status: {}", o.status);
                return false;
            }
            true
        },
        Err(e) => {
            eprintln!("Verovio check failed to execute: {}", e);
            false
        }
    }
}

fn generate_assessment(quiz: &Quiz) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut root = quick_xml::events::BytesStart::new("assessmentTest");
    root.push_attribute(("xmlns", "http://www.imsglobal.org/xsd/imsqti_v2p1"));
    root.push_attribute(("identifier", "assessment"));
    root.push_attribute(("title", quiz.title.as_str()));
    writer.write_event(Event::Start(root))?;

    // TestPart
    let mut test_part = quick_xml::events::BytesStart::new("testPart");
    test_part.push_attribute(("identifier", "part1"));
    test_part.push_attribute(("navigationMode", "linear"));
    test_part.push_attribute(("submissionMode", "individual"));
    writer.write_event(Event::Start(test_part))?;

    // AssessmentSection
    let mut section = quick_xml::events::BytesStart::new("assessmentSection");
    section.push_attribute(("identifier", "section1"));
    section.push_attribute(("title", "Section 1"));
    section.push_attribute(("visible", "true"));
    writer.write_event(Event::Start(section))?;

    // Questions
    for (i, question) in quiz.questions.iter().enumerate() {
        generate_question_item(&mut writer, question, i)?;
    }

    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("assessmentSection")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("testPart")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("assessmentTest")))?;

    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

fn generate_question_item(writer: &mut Writer<Cursor<Vec<u8>>>, question: &Question, index: usize) -> Result<()> {
    let id = format!("q{}", index + 1);
    
    let mut item = quick_xml::events::BytesStart::new("assessmentItem");
    item.push_attribute(("identifier", id.as_str()));
    item.push_attribute(("title", question.title.as_str()));
    item.push_attribute(("adaptive", "false"));
    item.push_attribute(("timeDependent", "false"));
    writer.write_event(Event::Start(item))?;

    // Response Declaration
    generate_response_declaration(writer, question)?;

    // Outcome Declaration (Points)
    let mut outcome = quick_xml::events::BytesStart::new("outcomeDeclaration");
    outcome.push_attribute(("identifier", "SCORE"));
    outcome.push_attribute(("cardinality", "single"));
    outcome.push_attribute(("baseType", "float"));
    writer.write_event(Event::Start(outcome))?;
    
    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("defaultValue")))?;
    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("value")))?;
    writer.write_event(Event::Text(quick_xml::events::BytesText::new("0")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("value")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("defaultValue")))?;
    
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("outcomeDeclaration")))?;

    // Item Body
    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("itemBody")))?;
    
    // Prompt
    // Using simpleChoice for MC/MA
    match question.question_type {
        QuestionType::MultipleChoice | QuestionType::MultipleAnswers | QuestionType::TrueFalse => {
            let mut choice_interaction = quick_xml::events::BytesStart::new("choiceInteraction");
            choice_interaction.push_attribute(("responseIdentifier", "RESPONSE"));
            choice_interaction.push_attribute(("shuffle", "true")); // Could be from quiz settings
            
            let max_choices = if question.question_type == QuestionType::MultipleAnswers { "0" } else { "1" };
            choice_interaction.push_attribute(("maxChoices", max_choices));

            writer.write_event(Event::Start(choice_interaction))?;
            
            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("prompt")))?;
            write_content(writer, &question.prompt)?;
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("prompt")))?;

            for (j, answer) in question.answers.iter().enumerate() {
                let choice_id = format!("choice_{}", j);
                let mut simple_choice = quick_xml::events::BytesStart::new("simpleChoice");
                simple_choice.push_attribute(("identifier", choice_id.as_str()));
                writer.write_event(Event::Start(simple_choice))?;
                write_content(writer, &answer.text)?;
                writer.write_event(Event::End(quick_xml::events::BytesEnd::new("simpleChoice")))?;
            }

            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("choiceInteraction")))?;
        },
        _ => {
            // Placeholder for other types
            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("p")))?;
            write_content(writer, &question.prompt)?;
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("p")))?;
        }
    }

    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("itemBody")))?;

    // Response Processing (Simple Match Correct)
    // This is complex in QTI 2.1, using a template is easier usually.
    // For now, we'll skip complex processing or use a standard template reference.
    let mut response_processing = quick_xml::events::BytesStart::new("responseProcessing");
    // response_processing.push_attribute(("template", "http://www.imsglobal.org/question/qti_v2p1/rptemplates/match_correct"));
    // Using inline processing for simplicity if possible, or just the template.
    // Template is safer for LMS compatibility.
    if question.question_type == QuestionType::MultipleChoice || question.question_type == QuestionType::TrueFalse {
         response_processing.push_attribute(("template", "http://www.imsglobal.org/question/qti_v2p1/rptemplates/match_correct"));
    }
    writer.write_event(Event::Empty(response_processing))?;

    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("assessmentItem")))?;
    Ok(())
}

fn generate_response_declaration(writer: &mut Writer<Cursor<Vec<u8>>>, question: &Question) -> Result<()> {
    let mut response_decl = quick_xml::events::BytesStart::new("responseDeclaration");
    response_decl.push_attribute(("identifier", "RESPONSE"));
    
    match question.question_type {
        QuestionType::MultipleChoice | QuestionType::MultipleAnswers | QuestionType::TrueFalse => {
            response_decl.push_attribute(("cardinality", if question.question_type == QuestionType::MultipleAnswers { "multiple" } else { "single" }));
            response_decl.push_attribute(("baseType", "identifier"));
            writer.write_event(Event::Start(response_decl))?;

            writer.write_event(Event::Start(quick_xml::events::BytesStart::new("correctResponse")))?;
            for (j, answer) in question.answers.iter().enumerate() {
                if answer.is_correct {
                    writer.write_event(Event::Start(quick_xml::events::BytesStart::new("value")))?;
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(&format!("choice_{}", j))))?;
                    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("value")))?;
                }
            }
            writer.write_event(Event::End(quick_xml::events::BytesEnd::new("correctResponse")))?;
        },
        _ => {
            // Others
            response_decl.push_attribute(("cardinality", "single"));
            response_decl.push_attribute(("baseType", "string"));
            writer.write_event(Event::Start(response_decl))?;
        }
    }

    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("responseDeclaration")))?;
    Ok(())
}

use crate::qti::lexer::{lex_content, Token};


fn write_latex_mathml(writer: &mut Writer<Cursor<Vec<u8>>>, latex: &str) -> Result<()> {
    if let Ok(mathml) = latex2mathml::latex_to_mathml(latex, latex2mathml::DisplayStyle::Inline) {
        // Parse the MathML string into events and write them
        let mut reader = quick_xml::Reader::from_str(&mathml);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(e) => writer.write_event(e)?,
                Err(e) => {
                    eprintln!("Error parsing generated MathML: {}", e);
                    // Fallback: write raw latex as text
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(&format!("${}$", latex))))?;
                    break;
                }
            }
            buf.clear();
        }
    } else {
        // Fallback
        writer.write_event(Event::Text(quick_xml::events::BytesText::new(&format!("${}$", latex))))?;
    }
    Ok(())
}

fn transpile_mhchem(ce: &str) -> String {
    // Very basic mhchem transpiler to standard LaTeX
    // 1. Wrap in \mathrm
    // 2. H2O -> H_{2}O (Numbers after letters)
    // 3. -> to \rightarrow
    // 4. + to +
    // 5. ^2+ to ^{2+}
    
    let mut result = String::new();
    let mut chars = ce.chars().peekable();
    
    result.push_str("\\mathrm{");
    
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            result.push_str("_{");
            result.push(c);
            // Consume subsequent digits
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    result.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push('}');
        } else if c == '-' && chars.peek() == Some(&'>') {
            chars.next(); // consume >
            result.push_str("}\\rightarrow\\mathrm{");
        } else if c == '^' {
            result.push('^');
            result.push('{');
            // Consume until space or end? Or just next token?
            // Simple assumption: ^2+ or ^+
            while let Some(&next) = chars.peek() {
                if next != ' ' {
                    result.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            result.push('}');
        } else if c == ' ' {
            result.push(' '); // Keep spaces
        } else {
            result.push(c);
        }
    }
    
    result.push('}');
    result
}
