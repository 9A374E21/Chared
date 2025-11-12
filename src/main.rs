// src/main.rs

mod control;
mod input;
mod output;

use crate::input::读取;
use crossterm::{execute, terminal::LeaveAlternateScreen};
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let content = if let Some(path) = args.get(1) {
        读取(path)
    } else {
        // 没有参数，返回空字符串
        Ok(String::new())
    }?;

    // 调用新函数获取视图相关数据
    let (行数, 保留行, 最大行数, 行向量) = crate::output::设置视图(&content)?;

    let 输入起始行 = 行数 - 保留行 + 1;
    let mut 起始行索引 = 0; // 当前窗口起始行索引

    let mut 光标 = crate::control::光标 { 行: 0, 列: 0,列历史: 0 };

    crate::output::显示(
        &行向量[起始行索引..std::cmp::min(起始行索引 + 最大行数, 行向量.len())],
        &mut 光标,
    )?;
    crate::input::输入(
        输入起始行,
        保留行,
        行数,
        &行向量,
        最大行数,
        &mut 起始行索引,
        &mut 光标,
    )?;

    // 退出交替屏幕
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
