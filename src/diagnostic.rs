use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::models::{DiagnosticReport, NoteIssue};
use crate::parser::{extract_image_refs, extract_links};
use crate::scanner::{scan_assets, scan_directory};

pub fn run_diagnostics(root_dir: &str) -> DiagnosticReport {
    let mut report = DiagnosticReport::new();

    let md_files = match scan_directory(root_dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("扫描目录失败：{}", e);
            return report;
        }
    };

    report.total_notes = md_files.len();

    let asset_files = match scan_assets(root_dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("扫描资产失败：{}", e);
            vec![]
        }
    };

    report.total_assets = asset_files.len();

    // 用文件名 stem 做索引，先不处理同名笔记冲突；真正的 Obsidian 规则要复杂很多。
    let note_index: HashMap<String, &PathBuf> = md_files
        .iter()
        .filter_map(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| (s.to_lowercase(), p))
        })
        .collect();

    let mut in_degree: HashMap<String, usize> = note_index.keys().map(|k| (k.clone(), 0)).collect();

    // key 直接借用 md_files 里的 PathBuf，避免这里复制一份路径列表。
    let mut out_degree: HashMap<&PathBuf, usize> = md_files.iter().map(|p| (p, 0usize)).collect();
    let mut referenced_images: HashSet<String> = HashSet::new();

    for file in &md_files {
        let links = match extract_links(file) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("无法读取文件 {:?}：{}", file, e);
                continue;
            }
        };

        for link in links {
            let link_stem = link.rsplit('/').next().unwrap_or(&link).to_lowercase();

            if note_index.contains_key(&link_stem) {
                *in_degree.entry(link_stem).or_insert(0) += 1;
                *out_degree.entry(file).or_insert(0) += 1;
            } else {
                // 这里的生命周期有点绕，为了不报所有权错误直接用了 .clone()，后面有空再优化。
                report.broken_links.push(NoteIssue::BrokenLink {
                    file: file.clone(),
                    link: link.clone(),
                });
            }
        }

        let image_refs = match extract_image_refs(file) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("无法读取图片引用 {:?}：{}", file, e);
                continue;
            }
        };

        for img_name in image_refs {
            referenced_images.insert(img_name.to_lowercase());
        }
    }

    for file in &md_files {
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let is_unreferenced = in_degree.get(&stem).copied().unwrap_or(0) == 0;
        let has_no_outgoing = out_degree.get(file).copied().unwrap_or(0) == 0;

        if is_unreferenced && has_no_outgoing {
            report
                .orphan_notes
                .push(NoteIssue::OrphanNote { file: file.clone() });
        }
    }

    for asset in &asset_files {
        let asset_name = asset
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if !referenced_images.contains(&asset_name) {
            report.dead_assets.push(NoteIssue::DeadAsset {
                file: asset.clone(),
            });
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn setup_vault(name: &str) -> PathBuf {
        let tmp = std::env::temp_dir().join(name);
        fs::create_dir_all(&tmp).unwrap();
        tmp
    }

    fn write_md(dir: &PathBuf, filename: &str, content: &str) {
        let path = dir.join(filename);
        fs::File::create(path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
    }

    #[test]
    fn test_no_broken_links() {
        let tmp = setup_vault("nd_diag_no_broken");
        write_md(&tmp, "A.md", "# A\n参见 [[B]]");
        write_md(&tmp, "B.md", "# B\n没有链接");

        let report = run_diagnostics(tmp.to_str().unwrap());
        assert_eq!(report.total_notes, 2);
        assert!(report.broken_links.is_empty());

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_detects_broken_link() {
        let tmp = setup_vault("nd_diag_broken");
        write_md(&tmp, "A.md", "# A\n参见 [[不存在的笔记]]");

        let report = run_diagnostics(tmp.to_str().unwrap());
        assert_eq!(report.broken_links.len(), 1);
        if let NoteIssue::BrokenLink { link, .. } = &report.broken_links[0] {
            assert_eq!(link, "不存在的笔记");
        } else {
            panic!("期望 BrokenLink 变体");
        }

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_nonexistent_root() {
        let report = run_diagnostics("/tmp/nd_nonexistent_vault_xyz");
        assert_eq!(report.total_notes, 0);
        assert!(report.broken_links.is_empty());
    }

    #[test]
    fn test_detects_orphan_note() {
        let tmp = setup_vault("nd_diag_orphan");
        write_md(&tmp, "A.md", "# A\n参见 [[B]]");
        write_md(&tmp, "B.md", "# B\n没有出链");
        write_md(&tmp, "C.md", "# C\n我是孤儿");

        let report = run_diagnostics(tmp.to_str().unwrap());
        assert_eq!(report.orphan_notes.len(), 1);
        if let NoteIssue::OrphanNote { file } = &report.orphan_notes[0] {
            assert_eq!(file.file_name().unwrap(), "C.md");
        } else {
            panic!("期望 OrphanNote 变体");
        }

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_referenced_but_no_outgoing_is_not_orphan() {
        let tmp = setup_vault("nd_diag_referenced");
        write_md(&tmp, "A.md", "# A\n参见 [[B]]");
        write_md(&tmp, "B.md", "# B\n没有出链");

        let report = run_diagnostics(tmp.to_str().unwrap());
        assert!(report.orphan_notes.is_empty());

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_detects_dead_asset() {
        let tmp = setup_vault("nd_diag_dead_asset");
        let assets = tmp.join("assets");
        fs::create_dir_all(&assets).unwrap();

        write_md(&tmp, "A.md", "# A\n![封面](assets/used.png)");
        fs::File::create(assets.join("used.png")).unwrap();
        fs::File::create(assets.join("unused.png")).unwrap();

        let report = run_diagnostics(tmp.to_str().unwrap());
        assert_eq!(report.dead_assets.len(), 1);
        if let NoteIssue::DeadAsset { file } = &report.dead_assets[0] {
            assert_eq!(file.file_name().unwrap(), "unused.png");
        } else {
            panic!("期望 DeadAsset 变体");
        }

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_no_dead_assets_when_all_referenced() {
        let tmp = setup_vault("nd_diag_no_dead_asset");
        let assets = tmp.join("assets");
        fs::create_dir_all(&assets).unwrap();

        write_md(&tmp, "A.md", "# A\n![图](assets/photo.jpg)");
        fs::File::create(assets.join("photo.jpg")).unwrap();

        let report = run_diagnostics(tmp.to_str().unwrap());
        assert!(report.dead_assets.is_empty());

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_integration_all_issue_types() {
        let tmp = setup_vault("nd_diag_integration");
        let assets = tmp.join("assets");
        fs::create_dir_all(&assets).unwrap();

        write_md(
            &tmp,
            "index.md",
            "# 首页\n\
             参见 [[日记]] 以及一篇 [[不存在]] 的笔记。\n\
             ![封面](assets/cover.png)",
        );
        write_md(&tmp, "日记.md", "# 日记\n今天天气不错。");
        write_md(&tmp, "孤儿.md", "# 孤儿\n我游离于知识网络之外。");

        fs::File::create(assets.join("cover.png")).unwrap();
        fs::File::create(assets.join("unused.jpg")).unwrap();

        let report = run_diagnostics(tmp.to_str().unwrap());

        assert_eq!(report.total_notes, 3, "应扫描到 3 个 .md 文件");
        assert_eq!(report.total_assets, 2, "应扫描到 2 个图片资产");
        assert_eq!(report.broken_links.len(), 1, "应发现 1 个死链");

        let broken_link_names: Vec<&str> = report
            .broken_links
            .iter()
            .filter_map(|i| {
                if let NoteIssue::BrokenLink { link, .. } = i {
                    Some(link.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(broken_link_names, vec!["不存在"], "死链目标应为「不存在」");

        assert_eq!(report.orphan_notes.len(), 1, "应发现 1 篇孤儿笔记");
        let orphan_names: Vec<&str> = report
            .orphan_notes
            .iter()
            .filter_map(|i| {
                if let NoteIssue::OrphanNote { file } = i {
                    file.file_name().and_then(|n| n.to_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(orphan_names, vec!["孤儿.md"], "孤儿笔记应为「孤儿.md」");

        assert_eq!(report.dead_assets.len(), 1, "应发现 1 个冗余图片");
        let dead_asset_names: Vec<&str> = report
            .dead_assets
            .iter()
            .filter_map(|i| {
                if let NoteIssue::DeadAsset { file } = i {
                    file.file_name().and_then(|n| n.to_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            dead_asset_names,
            vec!["unused.jpg"],
            "冗余资产应为「unused.jpg」"
        );

        assert!(
            !orphan_names.contains(&"日记.md"),
            "「日记.md」被 index.md 引用，不应被标记为孤儿"
        );

        fs::remove_dir_all(&tmp).unwrap();
    }
}
