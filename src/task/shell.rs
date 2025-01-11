use super::BUFFER_WIDTH;
use crate::io::vga_buffer::WRITER;
use crate::{print, println, task, time};
use alloc::{string::String, vec::Vec};
use core::fmt::Write;
use pc_keyboard::{DecodedKey, KeyCode};

/// Shell 结构体
pub struct Shell {
    input_buffer: String,   // 输入缓冲区
    prompt: &'static str,   // 提示符
    cursor_position: usize, // 光标位置
}

impl Shell {
    /// 创建一个新的 shell
    pub fn new() -> Self {
        Shell {
            input_buffer: String::new(),
            prompt: "blog_os> ",
            cursor_position: 0,
        }
    }

    /// 处理按键
    pub fn handle_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(key) => match key {
                KeyCode::ArrowLeft => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        self.redraw_line();
                    }
                }
                KeyCode::ArrowRight => {
                    if self.cursor_position < self.input_buffer.len() {
                        self.cursor_position += 1;
                        self.redraw_line();
                    }
                }
                KeyCode::Backspace => {
                    if self.cursor_position > 0 {
                        self.input_buffer.remove(self.cursor_position - 1);
                        self.cursor_position -= 1;
                        self.redraw_line();
                    }
                }
                KeyCode::Delete => {
                    if self.cursor_position < self.input_buffer.len() {
                        self.input_buffer.remove(self.cursor_position);
                        self.redraw_line();
                    }
                }
                _ => {}
            },
            DecodedKey::Unicode(c) => {
                if c == '\n' {
                    self.execute_command();
                    self.input_buffer.clear();
                    self.cursor_position = 0;
                    print!("{}", self.prompt);
                } else {
                    // 在光标位置插入字符
                    self.input_buffer.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    self.redraw_line();
                }
            }
        }
    }

    /// 重绘当前行
    fn redraw_line(&self) {
        let mut writer = WRITER.lock();
        let row = writer.get_row();
        writer.set_column(0);

        // 清除当前行
        for _ in 0..BUFFER_WIDTH {
            write!(writer, " ").unwrap();
        }
        writer.set_column(0);

        // 重绘提示符和输入内容
        write!(writer, "{}{}", self.prompt, self.input_buffer).unwrap();

        // 计算并设置光标位置
        let prompt_len = self.prompt.len();
        let cursor_column = prompt_len + self.cursor_position;
        writer.set_column(cursor_column);

        // 在光标位置显示空格
        write!(writer, " ").unwrap();

        // 恢复光标位置
        writer.set_column(cursor_column);
    }

    /// 执行输入缓冲区中的命令
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
            "run" => {
                let mut executor = task::executor::Executor::new();
                executor.spawn(task::Task::new(task::simple_task::print_task()));
                executor.run();
            }
            // "cat" => self.cmd_cat(&args[1..]),
            // "ls" => self.cmd_ls(),
            _ => println!("Unknown command: {}", args[0]),
        }
    }

    /// 打印帮助信息
    fn cmd_help(&self) {
        println!("------------------------------------");
        println!("|Available commands:");
        println!("|  help: Print this help message");
        println!("|  echo <string>: Print the string to the screen");
        println!("|  clear: Clear the screen");
        // println!("|  cat <filename>: Print the contents of the file");
        // println!("|  ls: List files in the filesystem");
        println!("------------------------------------");
    }

    /// 打印给定的参数
    fn cmd_echo(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: echo <string>");
            return;
        }

        println!("{}", args.join(" "));
    }

    /// 清屏
    fn cmd_clear(&self) {
        for _ in 0..super::BUFFER_HEIGHT {
            println!();
        }
    }

    // /// 打印给定名称的文件的内容
    // fn cmd_cat(&self, args: &[&str]) {
    //     if args.is_empty() {
    //         println!("Usage: cat <filename>");
    //         return;
    //     }

    //     let fs = crate::FILESYSTEM.lock();
    //     match fs.read_file(args[0]) {
    //         Some(content) => {
    //             // 将内容转换为字符串并打印
    //             if let Ok(s) = core::str::from_utf8(content) {
    //                 print!("{}", s);
    //             }
    //         }
    //         None => println!("File not found: {}", args[0]),
    //     }
    // }

    // /// 列出文件系统中的文件
    // fn cmd_ls(&self) {
    //     let fs = crate::FILESYSTEM.lock();
    //     // 打印所有文件名
    //     for file in fs.iter() {
    //         println!("{}", file.name());
    //     }
    // }
}
