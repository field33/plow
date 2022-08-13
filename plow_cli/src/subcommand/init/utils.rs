use camino::Utf8PathBuf;
use std::path::PathBuf;

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
