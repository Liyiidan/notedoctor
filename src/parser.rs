use regex::Regex;
use std::fs;
use std::path::PathBuf;

pub fn extract_links(file_path: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let content = fs::read_to_string(file_path)?;

    // TODO: 这里用正则匹配比较暴力，如果后期文件超过上万个，可能得换用 AST 解析树，期末时间不够先跑通。
    // 只匹配单层 [[...]]，避免跨过多个双链。
    let re = Regex::new(r"\[\[([^\[\]]+)\]\]").expect("正则表达式编译失败");

    let links = re
        .captures_iter(&content)
        .map(|cap| {
            let inner = &cap[1];
            match inner.split_once('|') {
                Some((target, _alias)) => target.trim().to_string(),
                None => inner.trim().to_string(),
            }
        })
        .collect();

    Ok(links)
}

pub fn extract_image_refs(file_path: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let content = fs::read_to_string(file_path)?;

    // Markdown 图片语法：![alt](path)。这里只拿括号里的本地路径。
    let re = Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").expect("正则表达式编译失败");

    let refs = re
        .captures_iter(&content)
        .filter_map(|cap| {
            let raw_path = cap[1].trim();
            if raw_path.starts_with("http://") || raw_path.starts_with("https://") {
                return None;
            }
            PathBuf::from(raw_path)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .collect();

    Ok(refs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_md(name: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_extract_simple_links() {
        let path = write_temp_md("test_simple.md", "参见 [[笔记A]] 和 [[笔记B]]。");
        let links = extract_links(&path).unwrap();
        assert_eq!(links, vec!["笔记A", "笔记B"]);
    }

    #[test]
    fn test_extract_alias_links() {
        let path = write_temp_md("test_alias.md", "参见 [[项目计划|点击这里]]。");
        let links = extract_links(&path).unwrap();
        assert_eq!(links, vec!["项目计划"]);
    }

    #[test]
    fn test_no_links() {
        let path = write_temp_md("test_empty.md", "这个文件没有任何链接。");
        let links = extract_links(&path).unwrap();
        assert!(links.is_empty());
    }

    #[test]
    fn test_nonexistent_file() {
        let path = PathBuf::from("/tmp/notedoctor_nonexistent_file.md");
        let result = extract_links(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_image_refs_basic() {
        let path = write_temp_md(
            "test_img_basic.md",
            "![封面](assets/cover.png) 和 ![图标](icons/logo.jpg)",
        );
        let refs = extract_image_refs(&path).unwrap();
        assert_eq!(refs, vec!["cover.png", "logo.jpg"]);
    }

    #[test]
    fn test_extract_image_refs_skips_external() {
        let path = write_temp_md(
            "test_img_ext.md",
            "![外链](https://example.com/img.png) ![本地](local.jpg)",
        );
        let refs = extract_image_refs(&path).unwrap();
        assert_eq!(refs, vec!["local.jpg"]);
    }

    #[test]
    fn test_extract_image_refs_empty() {
        let path = write_temp_md("test_img_empty.md", "没有任何图片引用。");
        let refs = extract_image_refs(&path).unwrap();
        assert!(refs.is_empty());
    }
}
