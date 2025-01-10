use alloc::string::String;
use alloc::vec::Vec;

pub struct File {
    name: String,
    content: Vec<u8>,
}

pub struct FileSystem {
    files: Vec<File>,
}

impl File {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl FileSystem {
    pub fn new() -> Self {
        FileSystem { files: Vec::new() }
    }

    pub fn iter(&self) -> impl Iterator<Item = &File> {
        self.files.iter()
    }

    pub fn create_file(&mut self, name: String, content: &[u8]) {
        let file = File {
            name,
            content: Vec::from(content),
        };
        self.files.push(file);
    }

    pub fn read_file(&self, name: &str) -> Option<&[u8]> {
        self.files
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.content.as_slice())
    }
}
