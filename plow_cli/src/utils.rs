pub mod url;

use camino::{Utf8Path, Utf8PathBuf};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
};

pub fn file_names_are_same(file1: &Utf8Path, file2: &Utf8Path) -> anyhow::Result<bool> {
    let name1 = file1
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("File has no name."))?;
    let name2 = file2
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("File has no name."))?;
    Ok(name1 == name2)
}

pub fn files_are_same(file1: &Utf8Path, file2: &Utf8Path) -> anyhow::Result<bool> {
    let file1 = File::open(file1)?;
    let file2 = File::open(file2)?;

    // Check if file sizes are different
    if file1.metadata()?.len() != file2.metadata()?.len() {
        return Ok(false);
    }

    // Use buf readers since they are much faster
    let file1 = BufReader::new(file1);
    let file2 = BufReader::new(file2);

    // Do a byte to byte comparison of the two files
    for (bytes1, bytes2) in file1.bytes().zip(file2.bytes()) {
        if bytes1? != bytes2? {
            return Ok(false);
        }
    }
    Ok(true)
}

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

/// Generates a sha256 hash from field name, namespace and version.
pub fn hash_field(namespace: &str, name: &str, version: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}/{} {}", namespace, name, version));
    format!("{:x}", hasher.finalize())
}
