use serde::{Deserialize, Serialize};
use rand::Rng;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEntry {
    pub answer: String,
    pub meanings: Vec<String>,
    pub hint_type: HintType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HintType {
    Meaning,
    PartialSpell,
    Context,
}

impl HintType {
    pub fn from_string(s: &str) -> Self {
        match s {
            "PartialSpell" => HintType::PartialSpell,
            "Context" => HintType::Context,
            _ => HintType::Meaning,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            HintType::Meaning => "Meaning".to_string(),
            HintType::PartialSpell => "PartialSpell".to_string(),
            HintType::Context => "Context".to_string(),
        }
    }
}

impl WordEntry {
    pub fn new(answer: &str, meanings: Vec<&str>) -> Self {
        Self {
            answer: answer.to_string(),
            meanings: meanings.iter().map(|s| s.to_string()).collect(),
            hint_type: HintType::Meaning,
        }
    }
    
    pub fn new_with_type(answer: &str, meanings: Vec<&str>, hint_type: HintType) -> Self {
        Self {
            answer: answer.to_string(),
            meanings: meanings.iter().map(|s| s.to_string()).collect(),
            hint_type,
        }
    }
    
    pub fn get_random_hint(&self) -> String {
        if self.meanings.is_empty() {
            return String::from("(无提示)");
        }
        
        let mut rng = rand::thread_rng();
        let count = rng.gen_range(1..=self.meanings.len());
        
        let mut selected_indices: Vec<usize> = (0..self.meanings.len()).collect();
        selected_indices.sort_by_key(|_| rng.gen::<u32>());
        selected_indices.truncate(count);
        selected_indices.sort();
        
        let selected_meanings: Vec<String> = selected_indices
            .iter()
            .map(|&i| self.meanings[i].clone())
            .collect();
        
        match self.hint_type {
            HintType::Meaning => {
                if selected_meanings.len() == 1 {
                    selected_meanings[0].clone()
                } else {
                    selected_meanings.join(" / ")
                }
            }
            HintType::PartialSpell | HintType::Context => {
                selected_meanings[0].clone()
            }
        }
    }
    
    /// 获取所有释义的字符串表示（用于 UI 显示）
    pub fn get_meanings_string(&self) -> String {
        self.meanings.join(" / ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordBook {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub words: Vec<WordEntry>,
    /// 组合本来源：非空时本单词本由这些来源本的单词并集构成（只读）
    #[serde(default)]
    pub sources: Vec<String>,
}

impl WordBook {
    pub fn new(id: &str, name: &str, description: &str, words: Vec<WordEntry>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            words,
            sources: Vec::new(),
        }
    }

    /// 创建组合本（由若干来源本的并集构成）
    pub fn new_composite(id: &str, name: &str, description: &str, sources: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            words: Vec::new(),
            sources,
        }
    }

    /// 是否是组合本（只读，单词由来源本聚合而来）
    pub fn is_composite(&self) -> bool {
        !self.sources.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.words.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
    
    /// 添加单词
    pub fn add_word(&mut self, word: WordEntry) {
        self.words.push(word);
    }
    
    /// 更新单词
    pub fn update_word(&mut self, index: usize, word: WordEntry) -> bool {
        if index < self.words.len() {
            self.words[index] = word;
            true
        } else {
            false
        }
    }
    
    /// 删除单词
    pub fn delete_word(&mut self, index: usize) -> bool {
        if index < self.words.len() {
            self.words.remove(index);
            true
        } else {
            false
        }
    }
    
    /// 获取所有单词信息（用于管理界面）
    pub fn get_words_info(&self) -> Vec<(String, String, String)> {
        self.words
            .iter()
            .map(|w| (w.answer.clone(), w.get_meanings_string(), w.hint_type.to_string()))
            .collect()
    }
}

pub struct WordBank {
    books: Vec<WordBook>,
    current_book_index: usize,
    file_path: PathBuf,
}

impl WordBank {
    pub fn new() -> Self {
        let file_path = Self::get_books_file_path();
        let books = Self::load_from_file(&file_path)
            .unwrap_or_else(|_| Self::create_default_books());

        Self {
            books,
            current_book_index: 0,
            file_path,
        }
    }

    fn get_books_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mut path = PathBuf::from(home);
        path.push(".recitor-books.json");
        path
    }

    fn load_from_file(path: &PathBuf) -> Result<Vec<WordBook>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let books = serde_json::from_str(&content)?;
        Ok(books)
    }

    /// 将当前所有单词本写回磁盘
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.books)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    /// 恢复出厂内置单词本（会覆盖当前全部单词本）
    pub fn reset_to_defaults(&mut self) {
        self.books = Self::create_default_books();
        self.current_book_index = 0;
        let _ = self.save();
    }

    /// 解析某个单词本的实际单词列表：组合本聚合来源本，普通本返回自身
    pub fn resolve_words(&self, index: usize) -> Vec<WordEntry> {
        let Some(book) = self.books.get(index) else {
            return Vec::new();
        };
        if !book.is_composite() {
            return book.words.clone();
        }

        let mut aggregated: Vec<WordEntry> = Vec::new();
        for source_id in &book.sources {
            if let Some(source) = self.books.iter().find(|b| &b.id == source_id) {
                for word in &source.words {
                    // 组合本内按单词原文去重，保留首次出现的条目
                    if !aggregated
                        .iter()
                        .any(|w| w.answer.eq_ignore_ascii_case(&word.answer))
                    {
                        aggregated.push(word.clone());
                    }
                }
            }
        }
        aggregated
    }

    /// 找到下一个未被占用的单词本 id
    fn next_available_id(&self) -> String {
        let mut n = 1;
        loop {
            let candidate = format!("custom-{}", n);
            if !self.books.iter().any(|b| b.id == candidate) {
                return candidate;
            }
            n += 1;
        }
    }

    /// 新建普通单词本，返回新本索引
    pub fn create_book(&mut self, name: &str, description: &str) -> usize {
        let id = self.next_available_id();
        self.books.push(WordBook::new(&id, name, description, Vec::new()));
        let _ = self.save();
        self.books.len() - 1
    }

    /// 新建组合单词本，返回新本索引
    pub fn create_composite_book(
        &mut self,
        name: &str,
        description: &str,
        sources: Vec<String>,
    ) -> usize {
        let id = self.next_available_id();
        self.books
            .push(WordBook::new_composite(&id, name, description, sources));
        let _ = self.save();
        self.books.len() - 1
    }

    /// 重命名单词本
    pub fn rename_book(&mut self, index: usize, name: &str, description: &str) -> bool {
        if index >= self.books.len() {
            return false;
        }
        self.books[index].name = name.to_string();
        self.books[index].description = description.to_string();
        let _ = self.save();
        true
    }

    /// 删除单词本，同时移除引用了它的组合本里的来源
    pub fn delete_book(&mut self, index: usize) -> bool {
        if index >= self.books.len() {
            return false;
        }
        let removed_id = self.books[index].id.clone();
        self.books.remove(index);

        for book in self.books.iter_mut() {
            book.sources.retain(|s| s != &removed_id);
        }

        if self.current_book_index >= self.books.len() {
            self.current_book_index = self.books.len().saturating_sub(1);
        }
        let _ = self.save();
        true
    }

    /// 将一组来源本 id 写入指定组合本（用于编辑组合关系）
    pub fn set_book_sources(&mut self, index: usize, sources: Vec<String>) -> bool {
        if index >= self.books.len() {
            return false;
        }
        self.books[index].sources = sources;
        let _ = self.save();
        true
    }
    
    pub fn get_books(&self) -> &[WordBook] {
        &self.books
    }
    
    pub fn set_current_book(&mut self, index: usize) -> bool {
        if index < self.books.len() {
            self.current_book_index = index;
            true
        } else {
            false
        }
    }

    pub fn current_index(&self) -> usize {
        self.current_book_index
    }
    
    pub fn get_current_book(&self) -> &WordBook {
        &self.books[self.current_book_index]
    }
    
    pub fn get_current_book_mut(&mut self) -> &mut WordBook {
        &mut self.books[self.current_book_index]
    }
    
    pub fn get_word(&self, index: usize) -> Option<&WordEntry> {
        self.get_current_book().words.get(index)
    }
    
    pub fn len(&self) -> usize {
        self.resolve_words(self.current_book_index).len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.get_current_book().is_empty()
    }
    
    fn create_default_books() -> Vec<WordBook> {
        vec![
            Self::create_basic_vocabulary(),
            Self::create_academic_vocabulary(),
            Self::create_phrase_vocabulary(),
            Self::create_all_vocabulary(),
        ]
    }
    
    fn create_basic_vocabulary() -> WordBook {
        WordBook::new(
            "basic",
            "基础词汇",
            "日常常用基础英语单词（30词）",
            vec![
                WordEntry::new("scream", vec!["高声喊", "尖叫", "呼啸"]),
                WordEntry::new("platform", vec!["月台", "平台", "讲台"]),
                WordEntry::new("rating", vec!["收视率", "评级", "等级"]),
                WordEntry::new("automatic", vec!["无意识的", "自动的", "机械的"]),
                WordEntry::new("channel", vec!["渠道", "频道", "海峡"]),
                WordEntry::new("memorable", vec!["难忘的", "值得纪念的"]),
                WordEntry::new("session", vec!["一段时间", "会议", "学期"]),
                WordEntry::new("league", vec!["联赛", "联盟", "等级"]),
                WordEntry::new("backfire", vec!["事与愿违", "回火", "逆火"]),
                WordEntry::new("literally", vec!["确实", "字面上地", "简直"]),
                WordEntry::new("examine", vec!["检查", "审查", "考试"]),
                WordEntry::new("warning", vec!["先兆", "警告", "预警"]),
                WordEntry::new("manner", vec!["方式", "态度", "举止"]),
                WordEntry::new("manners", vec!["礼仪", "礼貌", "规矩"]),
                WordEntry::new("minority", vec!["少数民族", "少数", "未成年"]),
                WordEntry::new("possession", vec!["物品", "拥有", "财产"]),
                WordEntry::new("remark", vec!["说", "评论", "注意"]),
                WordEntry::new("occupy", vec!["占领", "占据", "使忙碌"]),
                WordEntry::new("motive", vec!["动机", "目的", "主题"]),
                WordEntry::new("assistance", vec!["帮助", "援助", "协助"]),
                WordEntry::new("purse", vec!["财源", "钱包", "奖金"]),
                WordEntry::new("coverage", vec!["报道", "覆盖范围", "保险范围"]),
                WordEntry::new("expense", vec!["费用", "开销", "代价"]),
                WordEntry::new("statue", vec!["雕像", "塑像"]),
                WordEntry::new("bunch", vec!["束", "串", "群"]),
                WordEntry::new("delicacy", vec!["美味", "精致", "微妙"]),
                WordEntry::new("intently", vec!["专心地", "热切地", "一心一意地"]),
                WordEntry::new("accommodation", vec!["住宿", "调解", "适应"]),
                WordEntry::new("load", vec!["负载", "装载", "工作量"]),
                WordEntry::new("shelter", vec!["庇护所", "掩蔽", "保护"]),
            ],
        )
    }
    
    fn create_academic_vocabulary() -> WordBook {
        WordBook::new(
            "academic",
            "学术词汇",
            "考试常见学术英语单词（25词）",
            vec![
                WordEntry::new("resistance", vec!["抵制", "阻力", "抵抗"]),
                WordEntry::new("arise", vec!["出现", "产生", "起身"]),
                WordEntry::new("ultimately", vec!["最终", "根本上", "最后"]),
                WordEntry::new("grateful", vec!["感激", "感谢的", "令人愉快的"]),
                WordEntry::new("sustain", vec!["维持", "支撑", "遭受"]),
                WordEntry::new("contrast", vec!["形成对比", "对比", "差异"]),
                WordEntry::new("gravity", vec!["严重性", "重力", "庄严"]),
                WordEntry::new("exceptional", vec!["杰出", "异常的", "例外的"]),
                WordEntry::new("racial", vec!["种族的", "人种的"]),
                WordEntry::new("absorb", vec!["吸收", "理解", "使专心"]),
                WordEntry::new("recognition", vec!["认可", "识别", "承认"]),
                WordEntry::new("muscular", vec!["肌肉的", "强壮的", "有力的"]),
                WordEntry::new("episode", vec!["一集", "插曲", "事件"]),
                WordEntry::new("unfold", vec!["展开", "显露", "打开"]),
                WordEntry::new("surrounding", vec!["周围的", "环境", "附近的"]),
                WordEntry::new("applaud", vec!["鼓掌", "称赞", "赞许"]),
                WordEntry::new("shortage", vec!["短缺", "不足", "缺少"]),
                WordEntry::new("spring", vec!["春天", "跳跃", "泉水", "弹簧"]),
                WordEntry::new("withdraw", vec!["撤退", "提取", "退出"]),
                WordEntry::new("convinced", vec!["确信的", "信服的"]),
                WordEntry::new("ambition", vec!["雄心", "野心", "抱负"]),
                WordEntry::new("allow", vec!["允许", "给予", "考虑到"]),
                WordEntry::new("contrary", vec!["相反的", "对立的", "违反的"]),
                WordEntry::new_with_type("arise", vec!["a? from"], HintType::PartialSpell),
                WordEntry::new_with_type("recognition", vec!["beyond r?"], HintType::PartialSpell),
            ],
        )
    }
    
    fn create_phrase_vocabulary() -> WordBook {
        WordBook::new(
            "phrase",
            "短语搭配",
            "常见英语短语和固定搭配（24词）",
            vec![
                WordEntry::new_with_type("arise", vec!["a? from"], HintType::Context),
                WordEntry::new_with_type("grateful", vec!["g?"], HintType::PartialSpell),
                WordEntry::new_with_type("absorb", vec!["a? a lot of information"], HintType::Context),
                WordEntry::new_with_type("recognition", vec!["beyond r?"], HintType::PartialSpell),
                WordEntry::new_with_type("muscular", vec!["m?"], HintType::PartialSpell),
                WordEntry::new_with_type("episode", vec!["e?"], HintType::PartialSpell),
                WordEntry::new_with_type("unfold", vec!["u?"], HintType::PartialSpell),
                WordEntry::new_with_type("statue", vec!["s?"], HintType::PartialSpell),
                WordEntry::new_with_type("bunch", vec!["b?"], HintType::PartialSpell),
                WordEntry::new_with_type("delicacy", vec!["d?"], HintType::PartialSpell),
                WordEntry::new_with_type("intently", vec!["i?"], HintType::PartialSpell),
                WordEntry::new_with_type("accommodation", vec!["a?"], HintType::PartialSpell),
                WordEntry::new_with_type("load", vec!["l?"], HintType::PartialSpell),
                WordEntry::new_with_type("surrounding", vec!["the striking s? scenery"], HintType::Context),
                WordEntry::new_with_type("applaud", vec!["a? sb."], HintType::Context),
                WordEntry::new_with_type("motive", vec!["the m? of"], HintType::Context),
                WordEntry::new_with_type("shelter", vec!["fund his dog s?"], HintType::Context),
                WordEntry::new_with_type("exceptional", vec!["e? talent"], HintType::Context),
                WordEntry::new_with_type("shortage", vec!["global s? of work force"], HintType::Context),
                WordEntry::new_with_type("spring", vec!["s? up"], HintType::Context),
                WordEntry::new_with_type("withdraw", vec!["w? into oneself"], HintType::Context),
                WordEntry::new_with_type("racial", vec!["r? prejudice"], HintType::Context),
                WordEntry::new_with_type("convinced", vec!["be c? that"], HintType::Context),
                WordEntry::new_with_type("ambition", vec!["have no a? other than"], HintType::Context),
            ],
        )
    }
    
    fn create_all_vocabulary() -> WordBook {
        let mut all_words = Vec::new();
        all_words.extend(Self::create_basic_vocabulary().words);
        all_words.extend(Self::create_academic_vocabulary().words);
        all_words.extend(Self::create_phrase_vocabulary().words);
        
        WordBook::new(
            "all",
            "完整词库",
            "包含所有单词的综合测验（79词）",
            all_words,
        )
    }
}

impl Default for WordBank {
    fn default() -> Self {
        Self::new()
    }
}

// ===== 批量导入 / 导出 =====

use crate::porting::{book_to_payload, PayloadBook};

impl WordBank {
    /// 导出当前所有单词本（只含普通本，组合本被过滤）
    pub fn export_books(&self) -> Vec<PayloadBook> {
        self.books
            .iter()
            .filter(|b| !b.is_composite())
            .map(book_to_payload)
            .collect()
    }

    /// 批量导入单词本。merge=true 时，重名本合并单词（按单词原文去重）；merge=false 时，重名本被跳过。
    pub fn import_books(&mut self, payloads: Vec<PayloadBook>, merge: bool) {
        for payload in payloads {
            let name_key = payload.name.trim().to_lowercase();
            if let Some(existing) = self.books.iter_mut().find(|b| {
                !b.is_composite() && b.name.trim().to_lowercase() == name_key
            }) {
                if merge {
                    // 合并模式：把新单词加入已有本，按 answer 忽略大小写去重
                    for new_word in payload.words {
                        let answer_key = new_word.answer.to_lowercase();
                        if !existing
                            .words
                            .iter()
                            .any(|w| w.answer.to_lowercase() == answer_key)
                        {
                            existing.words.push(crate::word_bank::WordEntry {
                                answer: new_word.answer,
                                meanings: new_word.meanings,
                                hint_type: crate::word_bank::HintType::from_string(&new_word.hint_type),
                            });
                        }
                    }
                }
                // 否则跳过该单词本
            } else {
                // 无冲突，创建新本
                let id = self.next_available_id();
                let book = payload.to_word_book(&id);
                self.books.push(book);
            }
        }
        let _ = self.save();
    }
}
