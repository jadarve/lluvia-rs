use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Failed to load resource: {0}")]
    FailedToLoad(String),
}

pub trait Repository: Sync + Send {
    fn load(&self, path: &str) -> Result<Vec<u8>, RepositoryError>;
}

///////////////////////////////////////////////////////////////////////////////
// Repository impl
///////////////////////////////////////////////////////////////////////////////

// Force rebuild for HornSchunck layout change
pub(crate) static NODES_DIR: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources/nodes/");

pub struct InternalRepository {}

impl Repository for InternalRepository {
    fn load(&self, path: &str) -> Result<Vec<u8>, RepositoryError> {
        let file = NODES_DIR
            .get_file(path)
            .ok_or_else(|| RepositoryError::FailedToLoad(path.to_string()))?;

        let file_content = file.contents().to_vec();

        Ok(file_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_dir_entry(entry: &include_dir::DirEntry) {
        if let Some(file) = entry.as_file() {
            println!("{:?}", file.path());
        } else if let Some(dir) = entry.as_dir() {
            for entry in dir.entries() {
                print_dir_entry(entry);
            }
        }
    }

    #[test]
    fn test_list_nodes() {
        for d in NODES_DIR.entries() {
            print_dir_entry(d);
        }
    }
}
