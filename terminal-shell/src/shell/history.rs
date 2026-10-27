use std::path::Path;
use std::io::Write;

pub(crate) fn save(path: &Path, entries: &[String]) ->
std::io::Result<()> {
    let mut contents = entries.join("\n");

    if !entries.is_empty() {
        contents.push('\n');
    }
    std::fs::write(path, contents)
}

pub(crate) fn load(path: &Path) -> std::io::Result<Vec<String>> {
    let contents = std::fs::read_to_string(path)?;

    let entries = contents
        .lines()
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    Ok(entries)
}

pub(crate) fn append(path: &Path, entries: &[String]) -> 
std::io::Result<()> {
    let mut contents = entries.join("\n");

    if !entries.is_empty() {
        contents.push('\n');
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    file.write_all(contents.as_bytes())
}