// src/main.rs

mod file;
mod display;
mod input;

use std::io;
use crate::file::读取;
use crate::display::编辑窗口;

fn 运行(args: &[String]) -> io::Result<()> {
    let content = if let Some(path) = args.get(1) {
        读取(path)
    } else {
        // 没有参数，返回空字符串
        Ok(String::new())
    }?;
    编辑窗口(&content)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match 运行(&args) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}