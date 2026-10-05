//! 单词本批量导入 / 导出。
//!
//! 支持三种格式：
//! - `json`：结构化、无损，含 `hint_type` 与释义数组，推荐用于备份与迁移；
//! - `tsv` ：一行一词，`单词<TAB>释义1/释义2` 或带 `#` 标题头，便于表格软件编辑；
//! - `txt` ：每行 `单词: 释义1 / 释义2`，最易手写，但会丢失 `hint_type`。

use crate::word_bank::{HintType, WordBook, WordEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 规范化后的释义集合
fn meanings_to_vec(meanings: &str) -> Vec<String> {
    meanings
        .split('/')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn meanings_to_string(meanings: &[String]) -> String {
    meanings.join(" / ")
}

// ============================ JSON ============================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonWord {
    answer: String,
    meanings: Vec<String>,
    #[serde(default)]
    hint_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonBook {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    words: Vec<JsonWord>,
    #[serde(default)]
    sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonExport {
    format: String,
    version: u32,
    books: Vec<JsonBook>,
}

const JSON_FORMAT: &str = "recitor-books";
const JSON_VERSION: u32 = 1;

// ============================ 输出结构 ============================

/// 导出的单词（不含来源本引用，导入时一律落成普通本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadWord {
    pub answer: String,
    pub meanings: Vec<String>,
    pub hint_type: String,
}

/// 一本书的导入 / 导出载体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadBook {
    pub name: String,
    pub description: String,
    pub words: Vec<PayloadWord>,
}

impl PayloadBook {
    pub fn new(name: &str, description: &str, words: Vec<PayloadWord>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            words,
        }
    }
}

/// 从领域模型构造导出载体
pub fn book_to_payload(book: &WordBook) -> PayloadBook {
    let words = book
        .words
        .iter()
        .map(|w| PayloadWord {
            answer: w.answer.clone(),
            meanings: w.meanings.clone(),
            hint_type: w.hint_type.to_string(),
        })
        .collect();
    PayloadBook::new(&book.name, &book.description, words)
}

/// 把载体写回领域模型（用于导入时落成普通本）
impl PayloadBook {
    pub fn to_word_book(&self, id: &str) -> WordBook {
        let words = self
            .words
            .iter()
            .map(|w| WordEntry {
                answer: w.answer.clone(),
                meanings: dedup_meanings(&w.meanings),
                hint_type: HintType::from_string(&w.hint_type),
            })
            .collect();
        WordBook::new(id, &self.name, &self.description, words)
    }
}

fn dedup_meanings(meanings: &[String]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();
    for m in meanings {
        let trimmed = m.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_lowercase();
        if seen.insert(key) {
            out.push(trimmed.to_string());
        }
    }
    out
}

// ============================ 序列化 ============================

pub fn serialize(books: &[PayloadBook], format: &str) -> Result<String, String> {
    match normalize_format(format).as_str() {
        "json" => serialize_json(books),
        "tsv" => Ok(serialize_tsv(books)),
        "txt" => Ok(serialize_txt(books)),
        other => Err(format!("不支持的导出格式：{}", other)),
    }
}

pub fn normalize_format(format: &str) -> String {
    match format.trim().to_lowercase().as_str() {
        "json" | ".json" => "json".to_string(),
        "txt" | ".txt" => "txt".to_string(),
        _ => "tsv".to_string(),
    }
}

pub fn extension(format: &str) -> &'static str {
    match normalize_format(format).as_str() {
        "json" => "json",
        "txt" => "txt",
        _ => "tsv",
    }
}

fn serialize_json(books: &[PayloadBook]) -> Result<String, String> {
    let export = JsonExport {
        format: JSON_FORMAT.to_string(),
        version: JSON_VERSION,
        books: books
            .iter()
            .map(|b| JsonBook {
                name: b.name.clone(),
                description: b.description.clone(),
                words: b
                    .words
                    .iter()
                    .map(|w| JsonWord {
                        answer: w.answer.clone(),
                        meanings: w.meanings.clone(),
                        hint_type: Some(w.hint_type.clone()),
                    })
                    .collect(),
                sources: Vec::new(),
            })
            .collect(),
    };
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

fn serialize_tsv(books: &[PayloadBook]) -> String {
    let mut out = String::new();
    for book in books {
        out.push_str(&format!("# {}", book.name));
        if !book.description.trim().is_empty() {
            out.push_str(&format!("\t{}", book.description));
        }
        out.push('\n');
        for word in &book.words {
            out.push_str(&word.answer);
            out.push('\t');
            out.push_str(&meanings_to_string(&word.meanings));
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

fn serialize_txt(books: &[PayloadBook]) -> String {
    let mut out = String::new();
    for book in books {
        out.push_str(&format!("## {}\n", book.name));
        if !book.description.trim().is_empty() {
            out.push_str(&format!("# {}\n", book.description));
        }
        for word in &book.words {
            out.push_str(&format!(
                "{}: {}\n",
                word.answer,
                meanings_to_string(&word.meanings)
            ));
        }
        out.push('\n');
    }
    out
}

// ============================ 解析 ============================

pub fn parse(content: &str, format: &str) -> Result<Vec<PayloadBook>, String> {
    match normalize_format(format).as_str() {
        "json" => parse_json(content),
        "txt" => Ok(parse_txt(content)),
        _ => Ok(parse_tsv(content)),
    }
}

fn parse_json(content: &str) -> Result<Vec<PayloadBook>, String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with('{') {
        return Err("JSON 文件格式不正确：应以 { 开头".to_string());
    }

    let export: JsonExport =
        serde_json::from_str(content).map_err(|e| format!("JSON 解析失败：{}", e))?;

    if export.format != JSON_FORMAT {
        return Err(format!(
            "JSON 不是本工具的单词本导出格式（format={}）",
            export.format
        ));
    }

    let mut books = Vec::new();
    for jb in export.books {
        let words = jb
            .words
            .into_iter()
            .filter(|w| !w.answer.trim().is_empty() && !w.meanings.is_empty())
            .map(|w| PayloadWord {
                answer: w.answer.trim().to_string(),
                meanings: dedup_meanings(&w.meanings),
                hint_type: w
                    .hint_type
                    .unwrap_or_else(|| HintType::Meaning.to_string()),
            })
            .collect::<Vec<_>>();
        if !jb.name.trim().is_empty() {
            books.push(PayloadBook::new(
                jb.name.trim(),
                jb.description.trim(),
                words,
            ));
        }
    }

    if books.is_empty() {
        return Err("JSON 中没有可导入的单词本".to_string());
    }
    Ok(books)
}

/// 形如 `scream`, `高声喊 / 尖叫` 的单词行
fn is_word_line(line: &str) -> bool {
    line.contains('\t') || (line.contains(':') && !line.starts_with('#'))
}

fn parse_tsv(content: &str) -> Vec<PayloadBook> {
    let mut books: Vec<PayloadBook> = Vec::new();
    let mut current: Option<PayloadBook> = None;
    let mut unnamed: Vec<PayloadWord> = Vec::new();

    let push_current = |books: &mut Vec<PayloadBook>, current: &mut Option<PayloadBook>| {
        if let Some(book) = current.take() {
            if !book.words.is_empty() || !book.description.is_empty() {
                books.push(book);
            }
        }
    };

    for raw in content.lines() {
        let line = raw.trim_end();
        if line.trim().is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix('#') {
            if let Some(book) = current.take() {
                if !book.words.is_empty() || !book.description.is_empty() {
                    books.push(book);
                }
            }
            let mut parts = rest.splitn(2, '\t');
            let name = parts.next().unwrap_or("").trim().to_string();
            let desc = parts.next().unwrap_or("").trim().to_string();
            current = Some(PayloadBook::new(&name, &desc, Vec::new()));
            continue;
        }

        let (answer, meanings_raw) = if let Some((a, m)) = line.split_once('\t') {
            (a.trim(), m.trim())
        } else if let Some((a, m)) = line.split_once(':') {
            (a.trim(), m.trim())
        } else {
            (line.trim(), "")
        };

        if answer.is_empty() {
            continue;
        }

        let word = PayloadWord {
            answer: answer.to_string(),
            meanings: meanings_to_vec(meanings_raw),
            hint_type: HintType::Meaning.to_string(),
        };

        match current.as_mut() {
            Some(book) => book.words.push(word),
            None => unnamed.push(word),
        }
    }

    push_current(&mut books, &mut current);

    if !unnamed.is_empty() {
        books.insert(0, PayloadBook::new("导入单词", "从文件导入", unnamed));
    }

    books
}

fn parse_txt(content: &str) -> Vec<PayloadBook> {
    let mut books: Vec<PayloadBook> = Vec::new();
    let mut current: Option<PayloadBook> = None;
    let mut unnamed: Vec<PayloadWord> = Vec::new();

    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("##") {
            if let Some(book) = current.take() {
                if !book.words.is_empty() {
                    books.push(book);
                }
            }
            current = Some(PayloadBook::new(rest.trim(), "", Vec::new()));
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            match current.as_mut() {
                Some(book) if book.words.is_empty() && book.description.is_empty() => {
                    book.description = rest.trim().to_string();
                }
                _ => {
                    if let Some(book) = current.take() {
                        if !book.words.is_empty() {
                            books.push(book);
                        }
                    }
                    current = Some(PayloadBook::new(rest.trim(), "", Vec::new()));
                }
            }
            continue;
        }

        // 支持 `单词: 释义` / `单词 = 释义` / `单词<TAB>释义` / 纯单词
        let (answer, meanings_raw) = if let Some((a, m)) = line.split_once(':') {
            (a.trim(), m.trim())
        } else if let Some((a, m)) = line.split_once('=') {
            (a.trim(), m.trim())
        } else if let Some((a, m)) = line.split_once('\t') {
            (a.trim(), m.trim())
        } else {
            (line, "")
        };

        if answer.is_empty() {
            continue;
        }

        let word = PayloadWord {
            answer: answer.to_string(),
            meanings: meanings_to_vec(meanings_raw),
            hint_type: HintType::Meaning.to_string(),
        };

        match current.as_mut() {
            Some(book) => book.words.push(word),
            None => unnamed.push(word),
        }
    }

    if let Some(book) = current.take() {
        if !book.words.is_empty() {
            books.push(book);
        }
    }

    if !unnamed.is_empty() {
        books.insert(0, PayloadBook::new("导入单词", "从文件导入", unnamed));
    }

    books
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<PayloadBook> {
        vec![PayloadBook::new(
            "示例本",
            "描述",
            vec![
                PayloadWord {
                    answer: "scream".into(),
                    meanings: vec!["高声喊".into(), "尖叫".into()],
                    hint_type: "Meaning".into(),
                },
                PayloadWord {
                    answer: "arise".into(),
                    meanings: vec!["a? from".into()],
                    hint_type: "PartialSpell".into(),
                },
            ],
        )]
    }

    #[test]
    fn json_roundtrip() {
        let text = serialize(&sample(), "json").unwrap();
        let back = parse(&text, "json").unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].words.len(), 2);
        assert_eq!(back[0].words[1].hint_type, "PartialSpell");
    }

    #[test]
    fn tsv_roundtrip() {
        let text = serialize(&sample(), "tsv").unwrap();
        assert!(text.contains("# 示例本"));
        let back = parse(&text, "tsv").unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].words[0].meanings, vec!["高声喊", "尖叫"]);
    }

    #[test]
    fn txt_roundtrip() {
        let text = serialize(&sample(), "txt").unwrap();
        let back = parse(&text, "txt").unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].words[0].answer, "scream");
    }

    #[test]
    fn tsv_without_header() {
        let back = parse("scream\t高声喊/尖叫\nplatform\t月台\n", "tsv").unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].words.len(), 2);
        assert_eq!(back[0].name, "导入单词");
    }
}
