use std::io;
use std::path::PathBuf;

pub(crate) fn locate() -> io::Result<PathBuf> {
    let gui_path = std::env::current_exe()?;
    let shell_path = gui_path.with_file_name("terminal-shell");

    if !shell_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Build terminal-shell before starting the GUI."
        ));
    }

    Ok(shell_path)
} 