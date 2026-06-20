use std::path::PathBuf;
use walkdir::WalkDir;

pub fn scan_directory(dir_path: &str) -> Result<Vec<PathBuf>, String> {
    let path = PathBuf::from(dir_path);

    if !path.exists() {
        return Err(format!("路径不存在：{}", dir_path));
    }

    if !path.is_dir() {
        return Err(format!("路径不是一个目录：{}", dir_path));
    }

    let md_files: Vec<PathBuf> = WalkDir::new(&path)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .collect();

    Ok(md_files)
}

pub fn scan_assets(dir_path: &str) -> Result<Vec<PathBuf>, String> {
    let path = PathBuf::from(dir_path);

    if !path.exists() {
        return Err(format!("路径不存在：{}", dir_path));
    }

    if !path.is_dir() {
        return Err(format!("路径不是一个目录：{}", dir_path));
    }

    const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg"];

    let assets: Vec<PathBuf> = WalkDir::new(&path)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| IMAGE_EXTS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();

    Ok(assets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_scan_nonexistent_path() {
        let result = scan_directory("/tmp/notedoctor_nonexistent_dir_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_finds_md_files() {
        let tmp = std::env::temp_dir().join("notedoctor_test_scan");
        fs::create_dir_all(&tmp).unwrap();

        let md_file = tmp.join("note.md");
        let txt_file = tmp.join("ignore.txt");
        fs::File::create(&md_file)
            .unwrap()
            .write_all(b"# Test")
            .unwrap();
        fs::File::create(&txt_file)
            .unwrap()
            .write_all(b"ignored")
            .unwrap();

        let result = scan_directory(tmp.to_str().unwrap()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], md_file);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_scan_recursive() {
        let tmp = std::env::temp_dir().join("notedoctor_test_recursive");
        let sub = tmp.join("subdir");
        fs::create_dir_all(&sub).unwrap();

        fs::File::create(tmp.join("a.md")).unwrap();
        fs::File::create(sub.join("b.md")).unwrap();
        fs::File::create(sub.join("c.txt")).unwrap();

        let mut result = scan_directory(tmp.to_str().unwrap()).unwrap();
        result.sort();
        assert_eq!(result.len(), 2);

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn test_scan_assets_finds_images() {
        let tmp = std::env::temp_dir().join("notedoctor_test_assets");
        let sub = tmp.join("images");
        fs::create_dir_all(&sub).unwrap();

        fs::File::create(tmp.join("note.md")).unwrap();
        fs::File::create(sub.join("photo.png")).unwrap();
        fs::File::create(sub.join("banner.jpg")).unwrap();
        fs::File::create(sub.join("icon.jpeg")).unwrap();
        fs::File::create(sub.join("data.csv")).unwrap();

        let mut result = scan_assets(tmp.to_str().unwrap()).unwrap();
        result.sort();
        assert_eq!(result.len(), 3);

        fs::remove_dir_all(&tmp).unwrap();
    }
}
