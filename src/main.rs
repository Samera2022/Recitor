mod word_bank;
mod porting;
mod quiz;
mod progress;
mod statistics;

use quiz::Quiz;
use std::rc::Rc;
use std::cell::RefCell;
use slint::Model;

slint::include_modules!();

/// 根据当前单词本状态构建选择界面所需的列表模型
fn build_books_model(quiz: &Quiz) -> slint::ModelRc<WordBookInfo> {
    let books: Vec<WordBookInfo> = quiz
        .get_word_books_info()
        .iter()
        .map(|(id, name, desc, count, progress, is_composite, sources)| WordBookInfo {
            id: id.clone().into(),
            name: name.clone().into(),
            description: desc.clone().into(),
            word_count: *count as i32,
            progress: *progress as i32,
            is_composite: *is_composite,
            sources: sources.clone().into(),
        })
        .collect();
    Rc::new(slint::VecModel::from(books)).into()
}

/// 构建单词本管理界面的来源勾选列表
fn build_source_model(quiz: &Quiz, editing_index: i32) -> slint::ModelRc<BookSourceInfo> {
    let selected: Vec<String> = if editing_index >= 0 {
        quiz.get_book_sources(editing_index as usize)
    } else {
        Vec::new()
    };

    let sources: Vec<BookSourceInfo> = quiz
        .get_plain_book_options(editing_index)
        .iter()
        .map(|(id, name, _count)| BookSourceInfo {
            id: id.clone().into(),
            name: name.clone().into(),
            checked: selected.iter().any(|s| s == id),
        })
        .collect();

    Rc::new(slint::VecModel::from(sources)).into()
}

/// 将来源勾选列表里被选中的 id 收集出来
fn collect_selected_sources(ui: &AppWindow) -> Vec<String> {
    ui.get_book_source_list()
        .iter()
        .filter(|s| s.checked)
        .map(|s| s.id.to_string())
        .collect()
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let quiz = Rc::new(RefCell::new(Quiz::new()));
    
    // 初始化单词本列表
    {
        let quiz_ref = quiz.borrow();
        ui.set_word_books(build_books_model(&quiz_ref));
    }
    
    // Handle book selection
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_select_book(move |index| {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            if quiz.select_book(index as usize) {
                let book_name = quiz.get_current_book_name();
                let score = quiz.get_score();
                let tip = if score > 0 {
                    format!("已选择：{}\n上次进度：{}题\n点击开始测验继续", book_name, score)
                } else {
                    format!("已选择：{}\n点击开始测验", book_name)
                };
                
                ui.set_show_book_selector(false);
                ui.set_show_quiz(true);
                ui.set_show_management(false);
                ui.set_selected_book_index(index);
                ui.set_current_tip(tip.into());
                ui.set_quiz_active(false);
                ui.set_show_feedback(false);
                ui.set_score(score);
            }
        });
    }
    
    // Handle book management
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_manage_book(move |index| {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            if quiz.select_book(index as usize) {
                let book_name = quiz.get_current_book_name();
                let words_info = quiz.get_current_book_words();
                
                let words: Vec<WordInfo> = words_info
                    .iter()
                    .map(|(answer, meanings, hint_type)| WordInfo {
                        answer: answer.clone().into(),
                        meanings: meanings.clone().into(),
                        hint_type: hint_type.clone().into(),
                    })
                    .collect();
                
                let words_model = Rc::new(slint::VecModel::from(words));
                ui.set_words_list(words_model.into());
                
                // 加载统计数据
                let stats_list = quiz.get_current_book_statistics();
                let stats: Vec<WordStatsInfo> = stats_list
                    .iter()
                    .map(|s| WordStatsInfo {
                        word: s.word.clone().into(),
                        total_count: s.total_count as i32,
                        correct_count: s.correct_count as i32,
                        wrong_count: s.wrong_count as i32,
                        accuracy: s.accuracy(),
                        last_seen: s.last_seen.clone().into(),
                        is_weak: s.is_weak(),
                    })
                    .collect();
                
                let stats_model = Rc::new(slint::VecModel::from(stats));
                ui.set_word_stats_list(stats_model.into());
                
                // 加载汇总统计
                let (total, correct, wrong, accuracy) = quiz.get_current_book_summary();
                ui.set_book_total_count(total as i32);
                ui.set_book_correct_count(correct as i32);
                ui.set_book_wrong_count(wrong as i32);
                ui.set_book_accuracy(accuracy);
                
                ui.set_current_book_name(book_name.into());
                ui.set_selected_word_index(-1);
                ui.set_edit_answer("".into());
                ui.set_edit_meanings("".into());
                ui.set_edit_hint_type("Meaning".into());
                
                ui.set_show_book_selector(false);
                ui.set_show_quiz(false);
                ui.set_show_management(true);
            }
        });
    }
    
    // Handle word selection
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_select_word(move |index| {
            let ui = ui_weak.unwrap();
            
            if index >= 0 {
                let quiz_ref = quiz_clone.borrow();
                if let Some((answer, meanings, hint_type)) = quiz_ref.get_word_details(index as usize) {
                    ui.set_selected_word_index(index);
                    ui.set_edit_answer(answer.into());
                    ui.set_edit_meanings(meanings.into());
                    ui.set_edit_hint_type(hint_type.into());
                }
            } else {
                ui.set_selected_word_index(-1);
                ui.set_edit_answer("".into());
                ui.set_edit_meanings("".into());
                ui.set_edit_hint_type("Meaning".into());
            }
        });
    }
    
    // Handle save word
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_save_word(move || {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            let answer = ui.get_edit_answer().to_string();
            let meanings = ui.get_edit_meanings().to_string();
            let hint_type = ui.get_edit_hint_type().to_string();
            let selected_index = ui.get_selected_word_index();
            
            let success = if selected_index >= 0 {
                quiz.update_word(selected_index as usize, answer, meanings, hint_type)
            } else {
                quiz.add_word(answer, meanings, hint_type)
            };
            
            if success {
                let words_info = quiz.get_current_book_words();
                let words: Vec<WordInfo> = words_info
                    .iter()
                    .map(|(answer, meanings, hint_type)| WordInfo {
                        answer: answer.clone().into(),
                        meanings: meanings.clone().into(),
                        hint_type: hint_type.clone().into(),
                    })
                    .collect();
                
                let words_model = Rc::new(slint::VecModel::from(words));
                ui.set_words_list(words_model.into());
                
                ui.set_selected_word_index(-1);
                ui.set_edit_answer("".into());
                ui.set_edit_meanings("".into());
                ui.set_edit_hint_type("Meaning".into());
            }
        });
    }
    
    // Handle delete word
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_delete_word(move |index| {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            if quiz.delete_word(index as usize) {
                let words_info = quiz.get_current_book_words();
                let words: Vec<WordInfo> = words_info
                    .iter()
                    .map(|(answer, meanings, hint_type)| WordInfo {
                        answer: answer.clone().into(),
                        meanings: meanings.clone().into(),
                        hint_type: hint_type.clone().into(),
                    })
                    .collect();
                
                let words_model = Rc::new(slint::VecModel::from(words));
                ui.set_words_list(words_model.into());
                
                ui.set_selected_word_index(-1);
                ui.set_edit_answer("".into());
                ui.set_edit_meanings("".into());
                ui.set_edit_hint_type("Meaning".into());
            }
        });
    }
    
    // Handle add new word
    {
        let ui_weak = ui.as_weak();
        ui.on_add_new_word(move || {
            let ui = ui_weak.unwrap();
            ui.set_selected_word_index(-1);
            ui.set_edit_answer("".into());
            ui.set_edit_meanings("".into());
            ui.set_edit_hint_type("Meaning".into());
        });
    }
    
    // Handle reset word stats
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_reset_word_stats(move |word| {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            quiz.reset_word_statistics(&word.to_string());
            
            // 刷新统计数据
            let stats_list = quiz.get_current_book_statistics();
            let stats: Vec<WordStatsInfo> = stats_list
                .iter()
                .map(|s| WordStatsInfo {
                    word: s.word.clone().into(),
                    total_count: s.total_count as i32,
                    correct_count: s.correct_count as i32,
                    wrong_count: s.wrong_count as i32,
                    accuracy: s.accuracy(),
                    last_seen: s.last_seen.clone().into(),
                    is_weak: s.is_weak(),
                })
                .collect();
            
            let stats_model = Rc::new(slint::VecModel::from(stats));
            ui.set_word_stats_list(stats_model.into());
            
            let (total, correct, wrong, accuracy) = quiz.get_current_book_summary();
            ui.set_book_total_count(total as i32);
            ui.set_book_correct_count(correct as i32);
            ui.set_book_wrong_count(wrong as i32);
            ui.set_book_accuracy(accuracy);
        });
    }
    
    // Handle reset book stats
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_reset_book_stats(move || {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            quiz.reset_book_statistics();
            
            // 清空统计显示
            ui.set_word_stats_list(Rc::new(slint::VecModel::from(Vec::<WordStatsInfo>::new())).into());
            ui.set_book_total_count(0);
            ui.set_book_correct_count(0);
            ui.set_book_wrong_count(0);
            ui.set_book_accuracy(0.0);
        });
    }
    
    // Handle back to books from management
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_back_to_books_from_management(move || {
            let ui = ui_weak.unwrap();
            let quiz_ref = quiz_clone.borrow();

            ui.set_word_books(build_books_model(&quiz_ref));
            
            ui.set_show_book_selector(true);
            ui.set_show_quiz(false);
            ui.set_show_management(false);
        });
    }
    

    // ===== 单词本管理界面 =====

    // 打开单词本管理界面
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_open_book_manager(move || {
            let ui = ui_weak.unwrap();
            let quiz_ref = quiz_clone.borrow();

            ui.set_word_books(build_books_model(&quiz_ref));
            ui.set_book_manager_edit_index(-1);
            ui.set_book_manager_name("".into());
            ui.set_book_manager_desc("".into());
            ui.set_book_manager_composite(false);
            ui.set_book_source_list(build_source_model(&quiz_ref, -1));

            ui.set_show_book_selector(false);
            ui.set_show_quiz(false);
            ui.set_show_management(false);
            ui.set_show_book_manager(true);
        });
    }

    // 关闭单词本管理界面
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_close_book_manager(move || {
            let ui = ui_weak.unwrap();
            let quiz_ref = quiz_clone.borrow();

            ui.set_word_books(build_books_model(&quiz_ref));
            ui.set_show_book_selector(true);
            ui.set_show_quiz(false);
            ui.set_show_management(false);
            ui.set_show_book_manager(false);
        });
    }

    // 新建单词本：清空表单
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_new_book(move || {
            let ui = ui_weak.unwrap();
            let quiz_ref = quiz_clone.borrow();

            ui.set_book_manager_edit_index(-1);
            ui.set_book_manager_name("".into());
            ui.set_book_manager_desc("".into());
            ui.set_book_manager_composite(false);
            ui.set_book_source_list(build_source_model(&quiz_ref, -1));
        });
    }

    // 选中某个单词本进行编辑
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_edit_book(move |index| {
            let ui = ui_weak.unwrap();
            let quiz_ref = quiz_clone.borrow();

            let books = quiz_ref.get_word_books_info();
            if let Some((_id, name, desc, _count, _progress, is_composite, _sources)) =
                books.get(index as usize)
            {
                ui.set_book_manager_edit_index(index);
                ui.set_book_manager_name(name.clone().into());
                ui.set_book_manager_desc(desc.clone().into());
                ui.set_book_manager_composite(*is_composite);
                ui.set_book_source_list(build_source_model(&quiz_ref, index));
            }
        });
    }

    // 保存（新建或更新）
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_save_book(move || {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();

            let name = ui.get_book_manager_name().to_string();
            let desc = ui.get_book_manager_desc().to_string();
            let editing_index = ui.get_book_manager_edit_index();
            let is_composite = ui.get_book_manager_composite();
            let sources = collect_selected_sources(&ui);

            let success = if is_composite {
                if editing_index >= 0 {
                    let updated = quiz.rename_book(editing_index as usize, name, desc);
                    if updated {
                        quiz.set_book_sources(editing_index as usize, sources);
                    }
                    updated
                } else {
                    quiz.create_composite_book(name, desc, sources)
                }
            } else if editing_index >= 0 {
                let updated = quiz.rename_book(editing_index as usize, name, desc);
                if updated {
                    quiz.set_book_sources(editing_index as usize, Vec::new());
                }
                updated
            } else {
                quiz.create_book(name, desc)
            };

            if success {
                ui.set_word_books(build_books_model(&quiz));
                ui.set_book_manager_edit_index(-1);
                ui.set_book_manager_name("".into());
                ui.set_book_manager_desc("".into());
                ui.set_book_manager_composite(false);
                ui.set_book_source_list(build_source_model(&quiz, -1));
            }
        });
    }

    // 删除单词本
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_delete_book(move |index| {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();

            if quiz.delete_book(index as usize) {
                ui.set_word_books(build_books_model(&quiz));
                ui.set_book_manager_edit_index(-1);
                ui.set_book_manager_name("".into());
                ui.set_book_manager_desc("".into());
                ui.set_book_manager_composite(false);
                ui.set_book_source_list(build_source_model(&quiz, -1));
            }
        });
    }

    // 切换来源勾选
    {
        let ui_weak = ui.as_weak();
        ui.on_toggle_book_source(move |index| {
            let ui = ui_weak.unwrap();
            let mut sources: Vec<BookSourceInfo> = ui
                .get_book_source_list()
                .iter()
                .map(|s| BookSourceInfo {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    checked: s.checked,
                })
                .collect();

            if let Some(item) = sources.get_mut(index as usize) {
                item.checked = !item.checked;
            }

            ui.set_book_source_list(Rc::new(slint::VecModel::from(sources)).into());
        });
    }

    // 恢复内置单词本
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_reset_all_books(move || {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();

            quiz.reset_books();

            ui.set_word_books(build_books_model(&quiz));
            ui.set_book_manager_edit_index(-1);
            ui.set_book_manager_name("".into());
            ui.set_book_manager_desc("".into());
            ui.set_book_manager_composite(false);
            ui.set_book_source_list(build_source_model(&quiz, -1));
        });
    }

    // ===== 批量导入 / 导出 =====

    // 导出单词本
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_export_books(move || {
            let ui = ui_weak.unwrap();
            let quiz_ref = quiz_clone.borrow();

            let books = quiz_ref.export_books();
            if books.is_empty() {
                ui.set_import_export_status("没有可导出的单词本（组合本不会被导出）".into());
                return;
            }

            // 使用 tokio runtime 同步运行文件对话框
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let result = rt.block_on(async {
                rfd::AsyncFileDialog::new()
                    .set_title("导出单词本")
                    .add_filter("JSON 格式", &["json"])
                    .add_filter("TSV 表格", &["tsv"])
                    .add_filter("TXT 文本", &["txt"])
                    .set_file_name("单词本导出.json")
                    .save_file()
                    .await
            });

            if let Some(file_handle) = result {
                let path = file_handle.path();
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("json");
                
                match crate::porting::serialize(&books, ext) {
                    Ok(content) => {
                        if let Err(e) = std::fs::write(path, content) {
                            ui.set_import_export_status(format!("导出失败：{}", e).into());
                        } else {
                            ui.set_import_export_status(
                                format!("已导出 {} 个单词本", books.len()).into()
                            );
                        }
                    }
                    Err(e) => {
                        ui.set_import_export_status(format!("序列化失败：{}", e).into());
                    }
                }
            }
        });
    }

    // 导入单词本
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_import_books(move || {
            let ui = ui_weak.unwrap();

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let result = rt.block_on(async {
                rfd::AsyncFileDialog::new()
                    .set_title("导入单词本")
                    .add_filter("支持的格式", &["json", "tsv", "txt"])
                    .add_filter("JSON 格式", &["json"])
                    .add_filter("TSV 表格", &["tsv"])
                    .add_filter("TXT 文本", &["txt"])
                    .pick_file()
                    .await
            });

            if let Some(file_handle) = result {
                let path = file_handle.path();
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("json");

                match std::fs::read_to_string(path) {
                    Ok(content) => match crate::porting::parse(&content, ext) {
                        Ok(payloads) => {
                            if payloads.is_empty() {
                                ui.set_import_export_status("文件中没有可导入的单词本".into());
                                return;
                            }
                            
                            let mut quiz = quiz_clone.borrow_mut();
                            // 默认使用合并模式：重名本合并单词
                            quiz.import_books(payloads.clone(), true);
                            
                            drop(quiz);
                            let quiz_ref = quiz_clone.borrow();
                            ui.set_word_books(build_books_model(&quiz_ref));
                            ui.set_import_export_status(
                                format!("已导入 {} 个单词本", payloads.len()).into()
                            );
                        }
                        Err(e) => {
                            ui.set_import_export_status(format!("解析失败：{}", e).into());
                        }
                    },
                    Err(e) => {
                        ui.set_import_export_status(format!("读取失败：{}", e).into());
                    }
                }
            }
        });
    }

        // Handle back to books from quiz
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_back_to_books(move || {
            let ui = ui_weak.unwrap();
            let quiz_ref = quiz_clone.borrow();

            ui.set_word_books(build_books_model(&quiz_ref));
            
            ui.set_show_book_selector(true);
            ui.set_show_quiz(false);
            ui.set_show_management(false);
            ui.set_quiz_active(false);
            ui.set_show_feedback(false);
            ui.set_user_input("".into());
        });
    }
    
    // Handle start quiz
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_start_quiz(move || {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            quiz.start();
            
            ui.set_quiz_active(true);
            ui.set_show_feedback(false);
            ui.set_score(quiz.get_score());
            ui.set_current_tip(quiz.get_current_tip().into());
            ui.set_user_input("".into());
            ui.set_feedback("".into());
            ui.set_is_correct(false);
        });
    }
    
    // Handle submit answer
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_submit_answer(move |answer| {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            let user_answer = answer.to_string();
            let is_correct = quiz.check_answer(&user_answer);
            
            if is_correct {
                ui.set_feedback("正确！".into());
                ui.set_score(quiz.get_score());
                ui.set_is_correct(true);
                ui.set_show_feedback(true);
            } else {
                let correct_answer = quiz.get_current_answer();
                let all_meanings = quiz.get_current_all_meanings();
                let feedback_text = format!(
                    "错误！\n\n正确答案：{}\n所有释义：{}",
                    correct_answer,
                    all_meanings
                );
                ui.set_feedback(feedback_text.into());
                ui.set_is_correct(false);
                ui.set_show_feedback(true);
                ui.set_quiz_active(false);
            }
        });
    }
    
    // Handle next question
    {
        let ui_weak = ui.as_weak();
        let quiz_clone = quiz.clone();
        ui.on_next_question(move || {
            let ui = ui_weak.unwrap();
            let mut quiz = quiz_clone.borrow_mut();
            
            quiz.next_question();
            
            ui.set_show_feedback(false);
            ui.set_current_tip(quiz.get_current_tip().into());
            ui.set_user_input("".into());
            ui.set_feedback("".into());
            ui.set_is_correct(false);
        });
    }
    
    ui.run()
}
