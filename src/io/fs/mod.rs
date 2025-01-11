use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// File node, can be a file or a directory
///
/// 文件节点，可以是文件或目录
pub enum FsNode {
    File(File),
    Directory(Directory),
}

/// File structure
///
/// 文件结构
pub struct File {
    name: String,
    data: Vec<u8>,
}

/// 表示写入模式
pub enum WriteMode {
    Overwrite,
    Append,
}

impl File {
    /// Create a new file
    ///
    /// 创建一个新文件
    pub fn new(name: &str, content: &[u8]) -> Self {
        File {
            name: name.to_string(),
            data: content.to_vec(),
        }
    }

    /// Get the file name
    ///
    /// 获取文件名
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Read the file content
    ///
    /// 读取文件内容
    pub fn read(&self) -> &[u8] {
        &self.data
    }

    /// Write the file content (overwrite)
    ///
    /// 写入文件内容（覆盖或追加）
    pub fn write(&mut self, content: &[u8], mode: WriteMode) {
        match mode {
            WriteMode::Overwrite => {
                self.data.clear();
                self.data.extend_from_slice(content);
            }
            WriteMode::Append => {
                self.data.extend_from_slice(content);
            }
        }
    }
}

/// 目录结构
pub struct Directory {
    name: String,
    children: BTreeMap<String, FsNode>,
}

impl Directory {
    /// Create a new directory
    ///
    /// 创建一个新目录
    pub fn new(name: &str) -> Self {
        Directory {
            name: name.to_string(),
            children: BTreeMap::new(),
        }
    }

    /// Get the directory name
    ///
    /// 获取目录名
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the child node
    ///
    /// 获取子节点
    pub fn get_child(&self, name: &str) -> Option<&FsNode> {
        self.children.get(name)
    }

    /// Get a mutable reference to the child node
    ///
    /// 获取子节点的可变引用
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut FsNode> {
        self.children.get_mut(name)
    }

    /// Add a child node
    ///
    /// 添加子节点
    pub fn add_child(&mut self, node: FsNode) {
        match &node {
            FsNode::File(f) => self.children.insert(f.name().to_string(), node),
            FsNode::Directory(d) => self.children.insert(d.name().to_string(), node),
        };
    }
}

/// File system structure
///
/// 文件系统结构
pub struct FileSystem {
    root: Directory,
}

impl FileSystem {
    /// Create a new file system
    ///
    /// 创建一个新的文件系统
    pub fn new() -> Self {
        FileSystem {
            root: Directory::new("root"),
        }
    }

    /// Create a file in the root directory
    ///
    /// 在根目录下创建文件
    pub fn create_file(&mut self, name: &str, content: &[u8]) {
        let file = File::new(name, content);
        self.root.add_child(FsNode::File(file));
    }

    /// Create a directory in the root directory
    ///
    /// 读取文件内容
    pub fn read_file(&self, name: &str) -> Option<&[u8]> {
        if let Some(FsNode::File(f)) = self.root.get_child(name) {
            Some(f.read())
        } else {
            None
        }
    }

    /// Write file content
    ///
    /// 写入文件内容
    /// 如果文件不存在，则返回false
    /// 默认为覆盖模式写入
    pub fn write_file(&mut self, name: &str, content: &[u8]) -> bool {
        if let Some(FsNode::File(f)) = self.root.get_child_mut(name) {
            f.write(content, WriteMode::Overwrite);
            true
        } else {
            false
        }
    }

    /// List all files in the root directory
    ///
    /// 列出根目录下的所有文件/目录
    pub fn list_root(&self) -> Vec<&str> {
        self.root.children.keys().map(|k| k.as_str()).collect()
    }

    /// Get the root directory
    ///
    /// 获取根目录
    pub fn root(&self) -> &Directory {
        &self.root
    }

    /// Iterate over all files in the file system
    ///
    /// 遍历文件系统中的所有文件
    pub fn iter(&self) -> impl Iterator<Item = &File> {
        self.root.children.values().filter_map(|node| match node {
            FsNode::File(f) => Some(f),
            _ => None,
        })
    }
}
