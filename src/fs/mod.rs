use alloc::string::String;
use alloc::vec::Vec;

/// Struct representing a file
///
/// 文件结构体
pub struct File {
    name: String,
    content: Vec<u8>,
}

/// Struct representing a file system
///
/// 文件系统结构体
pub struct FileSystem {
    files: Vec<File>,
}

impl File {
    /// Create a new file with the given name and content
    ///
    /// 使用给定的名称和内容创建一个新文件
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl FileSystem {
    /// Create a new file system
    ///
    /// 创建一个新的文件系统
    pub fn new() -> Self {
        FileSystem { files: Vec::new() }
    }

    /// Get an iterator over all files in the file system
    ///
    /// 获取文件系统中所有文件的迭代器
    pub fn iter(&self) -> impl Iterator<Item = &File> {
        self.files.iter()
    }

    /// Create a new file with the given name and content
    ///
    /// 使用给定的名称和内容创建一个新文件
    pub fn create_file(&mut self, name: String, content: &[u8]) {
        let file = File {
            name,
            content: Vec::from(content),
        };
        self.files.push(file);
    }

    /// Read the content of the file with the given name
    ///
    /// 读取给定名称的文件的内容
    pub fn read_file(&self, name: &str) -> Option<&[u8]> {
        self.files
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.content.as_slice())
    }
}
