// src/output.rs
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{Clear, ClearType,EnterAlternateScreen, size},
};

use std::io;
use std::io::Write;

pub fn 设置视图(
    content: &String,
) -> io::Result<(
    u16,       // 行数
    u16,       // 保留行数
    usize,     // 最大可显示行数
    Vec<&str>, // 行向量
)> {
    // 初始化终端并进入交替屏幕
    execute!(io::stdout(), EnterAlternateScreen)?;
    execute!(io::stdout(), MoveTo(0, 0))?;

    let (_列数, 行数) = size()?;
    let mut 保留行 = ((行数 as f64) * 0.3).ceil() as u16;
    保留行 = std::cmp::max(3, 保留行);
    保留行 = std::cmp::min(行数, 保留行);

    let 行向量: Vec<&str> = content.lines().collect();
    // 计算可显示行数，确保不超过文件实际行数
    let 最大行数: usize = std::cmp::min((行数 - 保留行) as usize, 行向量.len());

    Ok((行数, 保留行, 最大行数, 行向量))
}

pub fn 显示(
    内容片段: &[&str],
    光标: &mut crate::control::光标,
) -> io::Result<()> {
    // 隐藏光标，移到起点
    execute!(io::stdout(), Hide, MoveTo(0, 0))?;

    // 清除需要更新的行
    let 行数 = 内容片段.len() as u16;
    for i in 0..行数 {
        execute!(io::stdout(), MoveTo(0, i))?;
        execute!(io::stdout(), Clear(ClearType::CurrentLine))?;
    }

    // 写入
    for (i, 行) in 内容片段.iter().enumerate() {
        execute!(io::stdout(), MoveTo(0, i as u16))?;
        write!(io::stdout(), "{}", 行)?;
        writeln!(io::stdout())?;
    }
    io::stdout().flush()?;

    // 光标重新定位
    let 最大行 = 行数.saturating_sub(1);
    let 最大列 = 内容片段[光标.行 as usize].len() as u16;
    execute!(
        io::stdout(),
        MoveTo(
            std::cmp::min(光标.列, 最大列),
            std::cmp::min(光标.行, 最大行)
        ),
        Show
    )?;

    Ok(())
}