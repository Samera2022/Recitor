use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 单词统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordStats {
    pub word: String,              // 单词
    pub total_count: usize,        // 总出现次数
    pub correct_count: usize,      // 答对次数
    pub wrong_count: usize,        // 答错次数
    pub last_seen: String,         // 最后出现时间
    pub last_result: bool,         // 最后一次结果
}

impl WordStats {
    pub fn new(word: String) -> Self {
        Self {
            word,
            total_count: 0,
            correct_count: 0,
            wrong_count: 0,
            last_seen: String::new(),
            last_result: false,
        }
    }
    
    /// 正确率（百分比）
    pub fn accuracy(&self) -> f32 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.correct_count as f32 / self.total_count as f32) * 100.0
        }
    }
    
    /// 是否是薄弱单词（正确率 < 60% 且出现过至少3次）
    pub fn is_weak(&self) -> bool {
        self.total_count >= 3 && self.accuracy() < 60.0
    }
}

/// 单词本统计数据
#[derive(Debug, Serialize, Deserialize)]
pub struct BookStats {
    pub book_id: String,
    pub words: HashMap<String, WordStats>,  // word -> stats
}

impl BookStats {
    pub fn new(book_id: String) -> Self {
        Self {
            book_id,
            words: HashMap::new(),
        }
    }
}

/// 统计数据
#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsData {
    pub books: HashMap<String, BookStats>,  // book_id -> stats
}

impl StatisticsData {
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }
}

pub struct StatisticsManager {
    data: StatisticsData,
    file_path: PathBuf,
}

impl StatisticsManager {
    pub fn new() -> Self {
        let file_path = Self::get_stats_file_path();
        let data = Self::load_from_file(&file_path).unwrap_or_else(|_| StatisticsData::new());
        
        Self { data, file_path }
    }
    
    fn get_stats_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(home);
        path.push(".recitor-statistics.json");
        path
    }
    
    fn load_from_file(path: &PathBuf) -> Result<StatisticsData, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let data = serde_json::from_str(&content)?;
        Ok(data)
    }
    
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }
    
    /// 记录答题结果
    pub fn record_result(&mut self, book_id: &str, word: &str, is_correct: bool) {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // 确保 book 存在
        if !self.data.books.contains_key(book_id) {
            self.data.books.insert(
                book_id.to_string(),
                BookStats::new(book_id.to_string()),
            );
        }
        
        let book_stats = self.data.books.get_mut(book_id).unwrap();
        
        // 确保 word 存在
        if !book_stats.words.contains_key(word) {
            book_stats.words.insert(
                word.to_string(),
                WordStats::new(word.to_string()),
            );
        }
        
        let word_stats = book_stats.words.get_mut(word).unwrap();
        word_stats.total_count += 1;
        word_stats.last_seen = now;
        word_stats.last_result = is_correct;
        
        if is_correct {
            word_stats.correct_count += 1;
        } else {
            word_stats.wrong_count += 1;
        }
        
        let _ = self.save_to_file();
    }
    
    /// 获取单词统计
    pub fn get_word_stats(&self, book_id: &str, word: &str) -> Option<WordStats> {
        self.data
            .books
            .get(book_id)
            .and_then(|book| book.words.get(word))
            .cloned()
    }
    
    /// 获取单词本的所有统计
    pub fn get_book_stats(&self, book_id: &str) -> Vec<WordStats> {
        self.data
            .books
            .get(book_id)
            .map(|book| book.words.values().cloned().collect())
            .unwrap_or_default()
    }
    
    /// 获取薄弱单词列表
    pub fn get_weak_words(&self, book_id: &str) -> Vec<WordStats> {
        self.get_book_stats(book_id)
            .into_iter()
            .filter(|stats| stats.is_weak())
            .collect()
    }
    
    /// 获取单词本汇总统计
    pub fn get_book_summary(&self, book_id: &str) -> (usize, usize, usize, f32) {
        let stats = self.get_book_stats(book_id);
        
        let total: usize = stats.iter().map(|s| s.total_count).sum();
        let correct: usize = stats.iter().map(|s| s.correct_count).sum();
        let wrong: usize = stats.iter().map(|s| s.wrong_count).sum();
        let accuracy = if total == 0 {
            0.0
        } else {
            (correct as f32 / total as f32) * 100.0
        };
        
        (total, correct, wrong, accuracy)
    }
    
    /// 重置单词统计
    pub fn reset_word_stats(&mut self, book_id: &str, word: &str) {
        if let Some(book_stats) = self.data.books.get_mut(book_id) {
            book_stats.words.remove(word);
            let _ = self.save_to_file();
        }
    }
    
    /// 重置单词本统计
    pub fn reset_book_stats(&mut self, book_id: &str) {
        self.data.books.remove(book_id);
        let _ = self.save_to_file();
    }
}

impl Default for StatisticsManager {
    fn default() -> Self {
        Self::new()
    }
}
