use camino::Utf8PathBuf;
use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

fn dig_files(maybe_file_paths: &mut Vec<PathBuf>, path: PathBuf) -> anyhow::Result<()> {
    if path.is_dir() {
        let paths = std::fs::read_dir(&path)?;
        for path_result in paths {
            let full_path = path_result?.path();
            dig_files(maybe_file_paths, full_path)?;
        }
    } else {
        maybe_file_paths.push(path);
    }
    Ok(())
}

pub fn list_files<T: Into<PathBuf>>(
    path: T,
    extension_filter: &str,
) -> anyhow::Result<Vec<Utf8PathBuf>> {
    let mut file_paths = vec![];
    let path = path.into();
    dig_files(&mut file_paths, path)?;
    Ok(file_paths
        .iter()
        .filter(|path| {
            path.extension()
                .map_or(false, |extension| extension == extension_filter)
        })
        .map(|path| Utf8PathBuf::from(path.to_string_lossy().as_ref()))
        .collect())
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: std::convert::AsRef<std::path::Path>,
{
    let file = File::open::<P>(path)?;
    Ok(io::BufReader::new(file).lines())
}
