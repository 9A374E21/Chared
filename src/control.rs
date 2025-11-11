// src/control.rs

use crossterm::{cursor::MoveTo, execute};

use std::io;
use std::io::Write;

pub fn 上移(
    光标: &mut crate::光标,
    行向量: &[&str],
    起始行索引: &mut usize,
    最大行数: usize,
    列数: u16,
) -> io::Result<()> {
    // 新增：文件行数小于窗口时直接返回
    if 行向量.len() <= 最大行数 {
        if 光标.行 == 0 {
            return Ok(());
        }
        光标.行 -= 1;
        光标限位(光标, &行向量, *起始行索引, 列数)?;
        let 当前行 = 行向量[*起始行索引 + 光标.行 as usize];
        光标.列 = 调整列到字符边界(当前行, 光标.列);
        return Ok(());
    }

    // 原逻辑（窗口大于文件时）
    if 光标.行 > 0 {
        光标.行 -= 1;
        光标限位(光标, &行向量, *起始行索引, 列数)?;
        let 当前行 = 行向量[*起始行索引 + 光标.行 as usize];
        光标.列 = 调整列到字符边界(当前行, 光标.列);
    } else if *起始行索引 > 0 {
        *起始行索引 -= 1;
        crate::display::显示(&行向量[*起始行索引..*起始行索引 + 最大行数], 光标)?;
        光标限位(光标, &行向量, *起始行索引, 列数)?;
        let 当前行 = 行向量[*起始行索引];
        光标.列 = 调整列到字符边界(当前行, 光标.列);
        光标.行 = 0;
    }
    Ok(())
}

pub fn 下移(
    光标: &mut crate::光标,
    行向量: &[&str],
    起始行索引: &mut usize,
    最大行数: usize,
    列数: u16,
    保留行: u16,
    行数: u16,
) -> io::Result<()> {
    if 行向量.is_empty() {
        return Ok(());
    }

    let 当前行索引 = *起始行索引 + 光标.行 as usize;

    // 新增：文件行数小于窗口时直接返回
    if 行向量.len() <= 最大行数 {
        if 当前行索引 == 行向量.len() - 1 {
            return Ok(());
        }
        光标.行 += 1;
        光标限位(光标, &行向量, *起始行索引, 列数)?;
        let 当前行 = 行向量[*起始行索引 + 光标.行 as usize];
        光标.列 = 调整列到字符边界(当前行, 光标.列);
        return Ok(());
    }

    // 原逻辑（窗口大于文件时）
    let 页面宽度 = (行数 - 保留行) as usize; // 转为 usize
    if 光标.行 == (行数 - 保留行 - 1) && (*起始行索引 + 页面宽度) < 行向量.len()
    {
        *起始行索引 += 1;
        crate::display::显示(&行向量[*起始行索引..*起始行索引 + 最大行数], 光标)?;
        光标限位(光标, &行向量, *起始行索引, 列数)?;
    } else if (*起始行索引 + 页面宽度) == 行向量.len() {
        // 已到文件末尾，光标不再移动
    } else {
        光标.行 += 1;
        光标限位(光标, &行向量, *起始行索引, 列数)?;
        let 当前行 = 行向量[*起始行索引 + 光标.行 as usize];
        光标.列 = 调整列到字符边界(当前行, 光标.列);
    }
    Ok(())
}

pub fn 左移(光标: &mut crate::光标, 行向量: &[&str], 起始行索引: &usize) {
    if 光标.列 > 0 {
        let 当前行索引 = *起始行索引 + 光标.行 as usize;
        if 当前行索引 < 行向量.len() {
            let 行 = 行向量[当前行索引];
            // 找前字符宽
            let mut 符宽 = 0usize;
            let mut prev_w = 0usize;
            for c in 行.chars() {
                let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
                if 符宽 + w > 光标.列.into() {
                    break;
                }
                prev_w = w;
                符宽 += w;
            }
            // 若光标不顶格，左移
            if prev_w > 0 {
                let 新列 = (光标.列 as usize) - prev_w;
                光标.列 = 新列.try_into().unwrap();
            } else {
                光标.列 = 0;
            }
        }
    }
}

pub fn 右移(光标: &mut crate::光标, 行向量: &[&str], 起始行索引: &usize) {
    let 当前行索引 = *起始行索引 + 光标.行 as usize;
    if 当前行索引 < 行向量.len() {
        let 行 = 行向量[当前行索引];
        // 只取当前符宽
        let 符宽_w = crate::control::当前字符宽度(行, 光标.列);
        // 计算新光标位
        let 新列 = 光标.列 + 符宽_w as u16;
        // 行长减 1，免超限位
        let 行长 = unicode_width::UnicodeWidthStr::width(行);
        if 新列 <= (行长 as u16) {
            光标.列 = 新列;
        }
    }
}

pub fn 落后(
    光标: &mut crate::光标,
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

pub fn 提前(光标: &mut crate::光标) {
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
    光标: &mut crate::光标,
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
