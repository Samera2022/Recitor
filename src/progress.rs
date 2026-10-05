use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookProgress {
    pub completed_words: usize,  // 已完成的单词数
    pub last_updated: String,     // 最后更新时间
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgressData {
    pub books: HashMap<String, BookProgress>,  // book_id -> progress
}

impl ProgressData {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }
}

pub struct ProgressManager {
    data: ProgressData,
    file_path: PathBuf,
}

impl ProgressManager {
    pub fn new() -> Self {
        let file_path = Self::get_progress_file_path();
        let data = Self::load_from_file(&file_path).unwrap_or_else(|_| ProgressData::new());
        
        Self { data, file_path }
    }
    
    fn get_progress_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(home);
        path.push(".recitor-progress.json");
        path
    }
    
    fn load_from_file(path: &PathBuf) -> Result<ProgressData, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let data = serde_json::from_str(&content)?;
        Ok(data)
    }
    
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }
    
    /// 获取单词本进度
    pub fn get_progress(&self, book_id: &str) -> usize {
        self.data
            .books
            .get(book_id)
            .map(|p| p.completed_words)
            .unwrap_or(0)
    }
    
    /// 更新单词本进度
    pub fn update_progress(&mut self, book_id: &str, completed: usize) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        self.data.books.insert(
            book_id.to_string(),
            BookProgress {
                completed_words: completed,
                last_updated: now,
            },
        );
        
        let _ = self.save_to_file();
    }
    
    /// 重置单词本进度
    pub fn reset_progress(&mut self, book_id: &str) {
        self.data.books.remove(book_id);
        let _ = self.save_to_file();
    }
    
    /// 获取所有进度
    pub fn get_all_progress(&self) -> HashMap<String, usize> {
        self.data
            .books
            .iter()
            .map(|(id, progress)| (id.clone(), progress.completed_words))
            .collect()
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}
