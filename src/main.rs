mod diagnostic;
mod models;
mod parser;
mod scanner;

use colored::Colorize;
use inquire::Text;

use crate::models::NoteIssue;

fn print_banner() {
    println!("{}", "╔══════════════════════════════════════╗".cyan());
    println!("{}", "║       NoteDoctor  🩺  知识库体检     ║".cyan());
    println!("{}", "╚══════════════════════════════════════╝".cyan());
    println!();
}

fn print_report(report: &models::DiagnosticReport) {
    println!();
    println!("{}", "─── 诊断报告 ───────────────────────────".cyan());
    println!(
        "{}  扫描笔记总数：{}",
        "✔".green(),
        report.total_notes.to_string().green().bold()
    );
    println!(
        "{}  扫描图片总数：{}",
        "✔".green(),
        report.total_assets.to_string().green().bold()
    );
    println!(
        "{}  发现问题总数：{}",
        if report.total_issues() == 0 {
            "✔".green()
        } else {
            "✘".red()
        },
        report.total_issues().to_string().yellow().bold()
    );

    println!();

    if report.broken_links.is_empty() {
        println!("{}", "✔  未发现任何死链".green());
    } else {
        println!(
            "{}  死链数量：{}",
            "✘".red(),
            report.broken_links.len().to_string().red().bold()
        );
        println!("{}", "─── 死链详情 ───────────────────────────".red());
        for issue in &report.broken_links {
            if let NoteIssue::BrokenLink { file, link } = issue {
                let filename = file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件");
                println!(
                    "  {} 在 {} 中发现死链 {}",
                    "✘".red(),
                    filename.yellow(),
                    format!("[[{}]]", link).red().bold()
                );
            }
        }
    }

    println!();

    if report.orphan_notes.is_empty() {
        println!("{}", "✔  未发现任何孤儿笔记".green());
    } else {
        println!(
            "{}  孤儿笔记数量：{}",
            "⚠".yellow(),
            report.orphan_notes.len().to_string().yellow().bold()
        );
        println!("{}", "─── 孤儿笔记详情 ────────────────────────".yellow());
        for issue in &report.orphan_notes {
            if let NoteIssue::OrphanNote { file } = issue {
                let filename = file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件");
                println!(
                    "  {} {} {}",
                    "⚠".yellow(),
                    filename.yellow().bold(),
                    "（无入链也无出链）".yellow()
                );
            }
        }
    }

    println!();

    if report.dead_assets.is_empty() {
        println!("{}", "✔  未发现任何冗余图片".green());
    } else {
        println!(
            "{}  冗余图片数量：{}",
            "ℹ".blue(),
            report.dead_assets.len().to_string().blue().bold()
        );
        println!("{}", "─── 冗余图片详情 ────────────────────────".blue());
        for issue in &report.dead_assets {
            if let NoteIssue::DeadAsset { file } = issue {
                let filename = file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件");
                println!(
                    "  {} {} {}",
                    "ℹ".blue(),
                    filename.blue().bold(),
                    "（未被任何笔记引用）".blue()
                );
            }
        }
    }

    println!();
    if report.total_issues() == 0 {
        println!("{}", "✔  知识库状态健康！".green().bold());
    }
    println!("{}", "────────────────────────────────────────".cyan());
    println!();
}

fn main() {
    print_banner();

    let path = match Text::new("请输入要扫描的知识库路径：")
        .with_help_message("支持绝对路径，例如 /Users/me/obsidian-vault")
        .prompt()
    {
        Ok(input) => input.trim().to_string(),
        Err(_) => {
            println!("{}", "已取消操作。".yellow());
            return;
        }
    };

    if path.is_empty() {
        println!("{}", "✘  路径不能为空。".red());
        return;
    }

    println!();
    println!("{}", format!("正在扫描 {} ...", path).dimmed());

    let report = diagnostic::run_diagnostics(&path);

    if report.total_notes == 0 {
        println!(
            "{}",
            "✘  未在该路径下找到任何 .md 文件，请确认路径是否正确。".red()
        );
        return;
    }

    print_report(&report);
}
