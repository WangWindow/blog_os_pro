use crate::{print, println};
use alloc::{string::String, vec::Vec};
use pc_keyboard::DecodedKey;

pub struct Shell {
    input_buffer: String,
    prompt: &'static str,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            input_buffer: String::new(),
            prompt: "blog_os> ",
        }
    }

    pub fn handle_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::Unicode(c) => {
                if c == '\n' {
                    self.execute_command();
                    self.input_buffer.clear();
                    print!("{}", self.prompt);
                } else {
                    print!("{}", c);
                    self.input_buffer.push(c);
                }
            }
            _ => {}
        }
    }

    fn execute_command(&self) {
        println!();
        let args: Vec<&str> = self.input_buffer.split_whitespace().collect();
        if args.is_empty() {
            return;
        }

        match args[0] {
            "help" => self.cmd_help(),
            "echo" => self.cmd_echo(&args[1..]),
            "clear" => self.cmd_clear(),
            "cat" => self.cmd_cat(&args[1..]),
            "ls" => self.cmd_ls(),
            _ => println!("Unknown command: {}", args[0]),
        }
    }

    fn cmd_help(&self) {
        println!("Available commands:");
        println!("help: Print this help message");
        println!("echo <string>: Print the string to the screen");
        println!("clear: Clear the screen");
        println!("cat <filename>: Print the contents of the file");
        println!("ls: List files in the filesystem");
    }

    fn cmd_echo(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: echo <string>");
            return;
        }

        println!("{}", args.join(" "));
    }

    fn cmd_clear(&self) {
        for _ in 0..crate::BUFFER_HEIGHT {
            println!();
        }
    }

    fn cmd_cat(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: cat <filename>");
            return;
        }

        let fs = crate::FILESYSTEM.lock();
        match fs.read_file(args[0]) {
            Some(content) => {
                // 将内容转换为字符串并打印
                if let Ok(s) = core::str::from_utf8(content) {
                    print!("{}", s);
                }
            }
            None => println!("File not found: {}", args[0]),
        }
    }

    fn cmd_ls(&self) {
        let fs = crate::FILESYSTEM.lock();
        // 打印所有文件名
        for file in fs.iter() {
            println!("{}", file.name());
        }
    }
}
