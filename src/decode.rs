pub fn decode_archives(
    source: &crate::BinarySource,
    destination: &crate::TextSource,
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Get list of archive files
    let archive_files = if let Some(files) = &source.archive {
        files.clone()
    } else if let Some(dir) = &source.archive_dir {
        // Read all files from directory
        std::fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect()
    } else {
        return Err("No archive source specified".into());
    };

    // Get list of text files
    let text_files = if let Some(files) = &destination.txt {
        files.clone()
    } else if let Some(dir) = &destination.text_dir {
        // Create vector of text file paths which will be created when writing
        archive_files
            .iter()
            .map(|archive_path| {
                let file_stem = archive_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                dir.join(format!("{}.txt", file_stem))
            })
            .collect()
    } else {
        return Err("No text destination specified".into());
    };

    println!("Archive files: {:?}", archive_files);
    println!("Text files: {:?}", text_files);

    Ok(())
}