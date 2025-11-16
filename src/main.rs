// src/main.rs

mod control;
mod input;
mod output;
pub use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode, size,
    },
};
pub use std::{
    fs::File,
    io::{self, Read, Write},
};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let content = if let Some(path) = args.get(1) {
        crate::input::读取(path)
    } else {
        // 没有参数，返回空字符串
        Ok(String::new())
    }?;

    //    初始化终端并进入交替屏幕
    execute!(io::stdout(), EnterAlternateScreen)?;
    execute!(io::stdout(), MoveTo(0, 0))?;

    let (_列数, 行数) = size()?;
    let mut 保留行 = ((行数 as f64) * 0.3).ceil() as u16;
    保留行 = std::cmp::max(3, 保留行);
    保留行 = std::cmp::min(行数, 保留行);

    let 行向量: Vec<&str> = content.lines().collect();
    /* 计算可显示行数，确保不超过文件实际行数 */
    let 最大行数: usize = std::cmp::min((行数 - 保留行) as usize, 行向量.len());

    /* ---------- end of merged logic ----------
    将结果直接赋值给变量 */
    let (行数, 保留行, 最大行数, 行向量) = (行数, 保留行, 最大行数, 行向量);

    // 其余代码保持不变
    let 输入起始行 = 行数 - 保留行 + 1;
    let mut 起始行索引 = 0; // 当前窗口起始行索引

    let mut 光标 = crate::control::光标 {
        行: 0,
        列: 0,
        列历史: 0,
    };

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
