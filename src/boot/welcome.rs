use crate::{print, println};
use core::fmt::Write;

const BANNER: &str = r#"
  _       _                    ___    ____
 | |__   | |   ___     __ _   / _ \  / ___|
 | '_ \  | |  / _ \   / _` | | | | | \___ \
 | |_) | | | | (_) | | (_| | | |_| |  ___) |
 |_.__/  |_|  \___/   \__, |  \___/  |____/
                      |___/
"#;

const WELCOME_TEXT: &str = r#"
Welcome to blogOS!
Type 'help' for a list of commands.
"#;

/// 显示欢迎界面
pub fn show_welcome() {
    // 打印彩色 Logo
    for line in BANNER.lines() {
        println!("{}", line);
    }

    // 打字机效果显示欢迎文本
    typewriter_print(WELCOME_TEXT, 50);
}

/// 打字机效果打印文本
fn typewriter_print(text: &str, delay_ms: u64) {
    for c in text.chars() {
        print!("{}", c);
        // spin_sleep(delay_ms);
    }
}

/// 自旋等待
fn spin_sleep(ms: u64) {
    let end = crate::int::time::current_time_millis() + ms;
    while crate::int::time::current_time_millis() < end {
        core::hint::spin_loop();
    }
}
