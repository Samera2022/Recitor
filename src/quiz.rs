use crate::word_bank::{WordBank, WordEntry, HintType};
use crate::progress::ProgressManager;
use crate::statistics::{StatisticsManager, WordStats};
use rand::Rng;

pub struct Quiz {
    word_bank: WordBank,
    progress_manager: ProgressManager,
    statistics_manager: StatisticsManager,
    /// 当前单词本解析后的单词列表（组合本为来源并集）
    current_words: Vec<WordEntry>,
    current_word: Option<WordEntry>,
    current_hint: String,
    score: i32,
}

impl Quiz {
    pub fn new() -> Self {
        let mut quiz = Self {
            word_bank: WordBank::new(),
            progress_manager: ProgressManager::new(),
            statistics_manager: StatisticsManager::new(),
            current_words: Vec::new(),
            current_word: None,
            current_hint: String::new(),
            score: 0,
        };
        quiz.refresh_current_words();
        quiz
    }

    /// 按当前选中的单词本（含组合本来源）重新解析单词列表
    fn refresh_current_words(&mut self) {
        let index = self.word_bank.current_index();
        self.current_words = self.word_bank.resolve_words(index);
    }
    
    pub fn get_word_books_info(&self) -> Vec<(String, String, String, usize, usize, bool, String)> {
        self.word_bank
            .get_books()
            .iter()
            .enumerate()
            .map(|(index, book)| {
                let progress = self.progress_manager.get_progress(&book.id);
                let sources = book
                    .sources
                    .iter()
                    .filter_map(|id| {
                        self.word_bank
                            .get_books()
                            .iter()
                            .find(|b| &b.id == id)
                            .map(|b| b.name.clone())
                    })
                    .collect::<Vec<_>>()
                    .join(" + ");
                (
                    book.id.clone(),
                    book.name.clone(),
                    book.description.clone(),
                    self.word_bank.resolve_words(index).len(),
                    progress,
                    book.is_composite(),
                    sources,
                )
            })
            .collect()
    }
    
    pub fn select_book(&mut self, index: usize) -> bool {
        let result = self.word_bank.set_current_book(index);
        if result {
            self.refresh_current_words();
            let book_id = self.word_bank.get_current_book().id.clone();
            let progress = self.progress_manager.get_progress(&book_id);
            self.score = progress as i32;
        }
        result
    }
    
    pub fn get_current_book_name(&self) -> String {
        self.word_bank.get_current_book().name.clone()
    }
    
    pub fn get_current_book_words(&self) -> Vec<(String, String, String)> {
        self.current_words
            .iter()
            .map(|w| {
                (
                    w.answer.clone(),
                    w.get_meanings_string(),
                    w.hint_type.to_string(),
                )
            })
            .collect()
    }

    /// 当前单词本是否为只读组合本
    pub fn current_book_is_composite(&self) -> bool {
        self.word_bank.get_current_book().is_composite()
    }
    
    pub fn add_word(&mut self, answer: String, meanings: String, hint_type_str: String) -> bool {
        let meanings_vec: Vec<String> = meanings
            .split(" / ")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if answer.is_empty() || meanings_vec.is_empty() {
            return false;
        }
        
        let hint_type = HintType::from_string(&hint_type_str);
        let word = WordEntry {
            answer,
            meanings: meanings_vec,
            hint_type,
        };
        
        if self.current_book_is_composite() {
            return false;
        }
        self.word_bank.get_current_book_mut().add_word(word);
        self.refresh_current_words();
        let _ = self.word_bank.save();
        true
    }
    
    pub fn update_word(&mut self, index: usize, answer: String, meanings: String, hint_type_str: String) -> bool {
        let meanings_vec: Vec<String> = meanings
            .split(" / ")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if answer.is_empty() || meanings_vec.is_empty() {
            return false;
        }
        
        let hint_type = HintType::from_string(&hint_type_str);
        let word = WordEntry {
            answer,
            meanings: meanings_vec,
            hint_type,
        };
        
        if self.current_book_is_composite() {
            return false;
        }
        let updated = self.word_bank.get_current_book_mut().update_word(index, word);
        if updated {
            self.refresh_current_words();
            let _ = self.word_bank.save();
        }
        updated
    }
    
    pub fn delete_word(&mut self, index: usize) -> bool {
        if self.current_book_is_composite() {
            return false;
        }
        let deleted = self.word_bank.get_current_book_mut().delete_word(index);
        if deleted {
            self.refresh_current_words();
            let _ = self.word_bank.save();
        }
        deleted
    }
    
    pub fn get_word_details(&self, index: usize) -> Option<(String, String, String)> {
        self.current_words.get(index).map(|w| {
            (
                w.answer.clone(),
                w.get_meanings_string(),
                w.hint_type.to_string(),
            )
        })
    }

    // ===== 单词本管理 =====

    pub fn create_book(&mut self, name: String, description: String) -> bool {
        let name = name.trim().to_string();
        if name.is_empty() {
            return false;
        }
        let description = if description.trim().is_empty() {
            "自定义单词本".to_string()
        } else {
            description.trim().to_string()
        };
        self.word_bank.create_book(&name, &description);
        true
    }

    pub fn create_composite_book(
        &mut self,
        name: String,
        description: String,
        sources: Vec<String>,
    ) -> bool {
        let name = name.trim().to_string();
        if name.is_empty() || sources.is_empty() {
            return false;
        }
        let description = if description.trim().is_empty() {
            format!("组合单词本（{} 个来源）", sources.len())
        } else {
            description.trim().to_string()
        };
        self.word_bank
            .create_composite_book(&name, &description, sources);
        true
    }

    pub fn rename_book(&mut self, index: usize, name: String, description: String) -> bool {
        let name = name.trim().to_string();
        if name.is_empty() {
            return false;
        }
        self.word_bank
            .rename_book(index, &name, description.trim())
    }

    pub fn delete_book(&mut self, index: usize) -> bool {
        let deleted = self.word_bank.delete_book(index);
        if deleted {
            self.refresh_current_words();
        }
        deleted
    }

    pub fn reset_books(&mut self) {
        self.word_bank.reset_to_defaults();
        self.refresh_current_words();
    }

    /// 某个单词本当前配置的来源 id 列表
    pub fn get_book_sources(&self, index: usize) -> Vec<String> {
        self.word_bank
            .get_books()
            .get(index)
            .map(|b| b.sources.clone())
            .unwrap_or_default()
    }

    /// 可作为组合来源的普通单词本（排除自身、排除组合本）
    pub fn get_plain_book_options(&self, exclude_index: i32) -> Vec<(String, String, usize)> {
        self.word_bank
            .get_books()
            .iter()
            .enumerate()
            .filter(|(i, b)| !b.is_composite() && Some(*i) != usize::try_from(exclude_index).ok())
            .map(|(_, b)| (b.id.clone(), b.name.clone(), b.words.len()))
            .collect()
    }

    /// 更新组合本的来源
    pub fn set_book_sources(&mut self, index: usize, sources: Vec<String>) -> bool {
        self.word_bank.set_book_sources(index, sources)
    }
    
    /// 获取当前单词本的统计信息
    pub fn get_current_book_statistics(&self) -> Vec<WordStats> {
        let book_id = self.word_bank.get_current_book().id.clone();
        self.statistics_manager.get_book_stats(&book_id)
    }
    
    /// 获取当前单词本的汇总统计
    pub fn get_current_book_summary(&self) -> (usize, usize, usize, f32) {
        let book_id = self.word_bank.get_current_book().id.clone();
        self.statistics_manager.get_book_summary(&book_id)
    }
    
    /// 重置单词统计
    pub fn reset_word_statistics(&mut self, word: &str) {
        let book_id = self.word_bank.get_current_book().id.clone();
        self.statistics_manager.reset_word_stats(&book_id, word);
    }
    
    /// 重置当前单词本统计
    pub fn reset_book_statistics(&mut self) {
        let book_id = self.word_bank.get_current_book().id.clone();
        self.statistics_manager.reset_book_stats(&book_id);
    }
    
    pub fn start(&mut self) {
        self.next_question();
    }
    
    pub fn next_question(&mut self) {
        let word_count = self.current_words.len();
        if word_count == 0 {
            return;
        }
        
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..word_count);
        let word = self.current_words[index].clone();
        self.current_hint = word.get_random_hint();
        self.current_word = Some(word);
    }
    
    pub fn check_answer(&mut self, user_answer: &str) -> bool {
        if let Some(ref word) = self.current_word {
            let correct = user_answer.trim().to_lowercase() == word.answer.to_lowercase();
            
            // 记录统计
            let book_id = self.word_bank.get_current_book().id.clone();
            self.statistics_manager.record_result(&book_id, &word.answer, correct);
            
            if correct {
                self.score += 1;
                self.save_progress();
            }
            correct
        } else {
            false
        }
    }
    
    fn save_progress(&mut self) {
        let book_id = self.word_bank.get_current_book().id.clone();
        self.progress_manager.update_progress(&book_id, self.score as usize);
    }
    
    pub fn get_current_tip(&self) -> String {
        self.current_hint.clone()
    }
    
    pub fn get_current_answer(&self) -> String {
        self.current_word
            .as_ref()
            .map(|w| w.answer.clone())
            .unwrap_or_default()
    }
    
    pub fn get_current_all_meanings(&self) -> String {
        self.current_word
            .as_ref()
            .map(|w| w.meanings.join(" / "))
            .unwrap_or_default()
    }
    
    pub fn get_score(&self) -> i32 {
        self.score
    }

    // ===== 批量导入 / 导出 =====

    /// 导出当前所有普通单词本
    pub fn export_books(&self) -> Vec<crate::porting::PayloadBook> {
        self.word_bank.export_books()
    }

    /// 导入单词本，merge=true 时合并重名本，false 则跳过重名本
    pub fn import_books(&mut self, payloads: Vec<crate::porting::PayloadBook>, merge: bool) {
        self.word_bank.import_books(payloads, merge);
        self.refresh_current_words();
    }
}

impl Default for Quiz {
    fn default() -> Self {
        Self::new()
    }
}
