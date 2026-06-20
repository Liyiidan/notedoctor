use std::path::PathBuf;

/// 诊断出的具体问题。
#[derive(Debug, Clone)]
pub enum NoteIssue {
    /// 文件里的 Obsidian 双链找不到目标笔记。
    BrokenLink { file: PathBuf, link: String },

    /// 没有入链，也没有出链的笔记。
    OrphanNote { file: PathBuf },

    /// 扫描到了图片文件，但没有任何 Markdown 引用它。
    DeadAsset { file: PathBuf },
}

/// 一次扫描汇总出来的结果。
#[derive(Debug)]
pub struct DiagnosticReport {
    pub total_notes: usize,
    pub total_assets: usize,
    pub broken_links: Vec<NoteIssue>,
    pub orphan_notes: Vec<NoteIssue>,
    pub dead_assets: Vec<NoteIssue>,
}

impl DiagnosticReport {
    pub fn new() -> Self {
        DiagnosticReport {
            total_notes: 0,
            total_assets: 0,
            broken_links: Vec::new(),
            orphan_notes: Vec::new(),
            dead_assets: Vec::new(),
        }
    }

    pub fn total_issues(&self) -> usize {
        self.broken_links.len() + self.orphan_notes.len() + self.dead_assets.len()
    }
}
