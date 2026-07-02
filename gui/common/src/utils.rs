

pub mod io {
    use std::{fs, path};

    pub fn open_file(path: impl AsRef<path::Path>) -> fs::File {
        let path = path.as_ref();

        if path.is_dir() {
            panic!("Path points to a directory!");
        }

        if !path.try_exists().expect("Failed to query file information!") {
            return fs::File::create(path).expect("Failed to create file!");
        }

        fs::File::open(path).expect("Failed to open file!")
    }

}

