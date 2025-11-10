// src/display.rs

pub struct 光标 {
    pub 行: u16,
    pub 列: u16,
}

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size},
};
use std::io;
use std::io::Write;

pub fn 显示(content_slice: &[&str], 光标: &mut 光标) -> io::Result<()> {
    // 隐藏光标
    execute!(io::stdout(), Hide)?;
    execute!(io::stdout(), MoveTo(0, 0))?;

    // 写入
    for 行 in content_slice {
        write!(io::stdout(), "{}", 行)?;
        execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        writeln!(io::stdout())?;
    }

    io::stdout().flush()?;

    // 保持光标
    execute!(io::stdout(), MoveTo(光标.列, 光标.行))?;
    execute!(io::stdout(), Show)?;
    Ok(())
}

pub fn 落后(
    光标: &mut 光标, 行向量: &[&str], 起始行索引: usize, 列数: u16
) -> io::Result<()> {
    let 当前行索引 = 起始行索引 + 光标.行 as usize;
    if 当前行索引 < 行向量.len() {
        // 计算实际显示宽度
        let 行长 = unicode_width::UnicodeWidthStr::width(行向量[当前行索引]);
        if 行长 == 0 {
            光标.列 = 0;
        } else {
            let 列数_usize = 列数 as usize;
            let col_pos = if 行长 < 列数_usize {
                行长
            } else {
                列数_usize - 1
            };
            光标.列 = col_pos as u16;
        }
    } else {
        光标.列 = 列数 - 1;
    }
    execute!(io::stdout(), MoveTo(光标.列, 光标.行))?;
    Ok(())
}

pub fn 提前(光标: &mut crate::display::光标) {
    // 将光标列置为 0（即行首位置）
    光标.列 = 0;
}

// 辅助函数：给定字符串和当前位置，返回当前所在字符的宽度
pub fn 当前字符宽度(行: &str, 列: u16) -> usize {
    let mut 符宽 = 0usize;
    for c in 行.chars() {
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
        if 符宽 + w > 列.into() {
            return w;
        }
        符宽 += w;
    }
    0
}

// 新增：把光标列定位到最近的字符边界（即字符起始位置）
pub fn 调整列到字符边界(行: &str, 列: u16) -> u16 {
    let mut 符宽 = 0usize;
    for c in 行.chars() {
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
        if (符宽 + w as usize) > 列.into() {
            // 当前列已超过字符宽度，返回上一个边界
            return (符宽 as u16).try_into().unwrap();
        }
        符宽 += w;
    }
    列
}

pub fn 处理字符键(
    ch: char,
    光标列: &mut u16,
    已输入: &mut bool,
    输入起始行: u16,
) -> io::Result<()> {
    execute!(io::stdout(), crossterm::cursor::MoveTo(*光标列, 输入起始行))?;
    print!("{}", ch);
    io::stdout().flush()?;
    *光标列 += 1;
    *已输入 = true;
    execute!(io::stdout(), crossterm::cursor::MoveTo(*光标列, 输入起始行))?;
    Ok(())
}

pub fn 光标限位(
    光标: &mut 光标,
    行向量: &[&str],
    起始行索引: usize,
    列数: u16,
) -> io::Result<()> {
    let 当前行索引 = 起始行索引 + 光标.行 as usize;
    if 当前行索引 < 行向量.len() {
        // 计算实际显示宽度
        let 行长 = unicode_width::UnicodeWidthStr::width(行向量[当前行索引]);
        if 行长 == 0 {
            光标.列 = 0;
        } else {
            let 最宽列 = (行长 as u16) - 1;
            if 光标.列 > 最宽列 + 1 {
                光标.列 = 最宽列 + 1;
            }
        }
    } else {
        光标.列 = 列数 - 1;
    }
    Ok(())
}
pub fn 编辑窗口(content: &str) -> io::Result<()> {
    execute!(io::stdout(), Clear(ClearType::All), EnterAlternateScreen)?;
    execute!(io::stdout(), MoveTo(0, 0))?;

    let (_列数, 行数) = size()?;
    let mut 保留行 = ((行数 as f64) * 0.3).ceil() as u16;
    保留行 = std::cmp::max(3, 保留行);
    保留行 = std::cmp::min(行数, 保留行);

    let 输入起始行 = 行数 - 保留行 + 1;

    let 行向量: Vec<&str> = content.lines().collect();
    let 最大行数: usize = (行数 - 保留行) as usize;
    let mut 起始行索引 = 0; // 当前窗口起始行索引

    let mut 光标 = 光标 { 行: 0, 列: 0 };

    显示(&行向量[起始行索引..起始行索引 + 最大行数], &mut 光标)?;
    crate::input::输入(
        输入起始行,
        保留行,
        行数,
        &行向量,
        最大行数,
        &mut 起始行索引,
        &mut 光标,
    )?;

    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
