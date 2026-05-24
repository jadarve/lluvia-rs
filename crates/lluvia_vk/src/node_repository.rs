pub(crate) static NODES_DIR: include_dir::Dir = include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources/nodes/");

pub struct NodeRepository {}

impl NodeRepository {
    #[allow(dead_code)]
    fn new() -> Self {
        for _d in NODES_DIR.dirs() {}

        Self {}
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
