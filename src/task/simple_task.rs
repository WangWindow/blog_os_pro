use crate::{println, time};

pub async fn print_task() {
    let mut i = 0;
    loop {
        println!("print_task: {}", i);
        i += 1;
        time::sleep(1);
    }
}
