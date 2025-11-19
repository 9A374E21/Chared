// src/control.rs

use crate::*;
macro_rules! 句宽 {
    ($s:expr) => {
        unicode_width::UnicodeWidthStr::width($s)
    };
}
macro_rules! 字宽 {
    ($s:expr) => {
        unicode_width::UnicodeWidthChar::width($s)
    };
}

pub fn 下移(
    光标: &mut 光标, 行向量: &[&str], 最大行数: usize, 下: bool
) -> io::Result<()> {
    if 行向量.is_empty() {
        return Ok(());
    }
    // 记录旧的行号
    let 原行 = 光标.行;
    // 计算新行并限制范围
    let 新行 = ((光标.行 as i32 + if 下 { 1 } else { -1 }).clamp(0, (最大行数 - 1) as i32)) as u16;

    // 更新光标位置
    if 新行 != 光标.行 {
        光标.行 = 新行;
        let 当前行号 = 光标.行索引 + 光标.行 as usize;
        if 当前行号 < 行向量.len() {
            let 行长 = 句宽!(行向量[当前行号]);
            if 行长 == 0 {
                光标.列 = 0;
            } else {
                let 最宽列 = 行长 as u16;
                if 光标.列 > 最宽列 {
                    光标.列 = 最宽列;
                }
            }
        }
        调整光标列(行向量, 光标);
    }

    // 向下滚动（当 下 == true 且光标已处于底部边界）
    if 下 && 原行 == (最大行数 as u16 - 1) && (光标.行索引 + 最大行数) < 行向量.len()
    {
        光标.行索引 = 光标.行索引 + 1 as usize;
        output::文件显示(&行向量[光标.行索引..光标.行索引 + 最大行数], 光标)?;
        调整光标列(行向量, 光标);
    }
    // 向上滚动（当 下 == false 且光标已处于顶部边界）
    else if !下 && 原行 == 0 && (光标.行索引 > 0) {
        光标.行索引 = 光标.行索引 - 1 as usize;
        output::文件显示(&行向量[光标.行索引..光标.行索引 + 最大行数], 光标)?;
        调整光标列(行向量, 光标);
    }
    // 边界
    else if (光标.行索引 + 最大行数) == 行向量.len() || 光标.行索引 == 0 {
    }

    Ok(())
}

pub fn 下翻(
    光标: &mut 光标,
    行向量: &[&str],
    最大行数: usize,
    下翻: bool, // true 表示下翻，false 表示上翻
) -> io::Result<()> {
    // 若已在最上方或最底部则无操作
    if (!下翻 && 光标.行索引 == 0) || (下翻 && (光标.行索引 + 最大行数) >= 行向量.len())
    {
        return Ok(());
    }
    // 根据方向决定增减值
    let delta = std::cmp::max(1, 最大行数);
    let 新起始 = if 下翻 {
        光标.行索引.saturating_add(delta)
    } else {
        光标.行索引.saturating_sub(delta)
    };

    // 防止越界
    光标.行索引 = std::cmp::min(
        新起始,
        if 下翻 {
            行向量.len() - 最大行数
        } else {
            行向量.len()
        },
    );

    // 计算切片结束位置，保证不会超出长度
    let 切片结束 = std::cmp::min(光标.行索引 + 最大行数, 行向量.len());
    crate::output::文件显示(&行向量[光标.行索引..切片结束], 光标)?;

    调整光标列(行向量, 光标);

    Ok(())
}

pub fn 右移(光标: &mut 光标, 行向量: &[&str], 右: bool) {
    let 当前行号 = 光标.行索引 + 光标.行 as usize;
    if 当前行号 < 行向量.len() {
        let 行 = 行向量[当前行号];
        if 右 {
            let 字宽 = 当前字符宽度(行, 光标.列);
            let 新列 = 光标.列 + 字宽 as u16;
            let 行长 = 句宽!(行);
            if 新列 <= (行长 as u16) {
                光标.列 = 新列;
                光标.列历史 = 光标.列;
            }
        } else {
            if 光标.列 > 0 {
                let mut 字宽 = 0usize;
                let mut 前字宽 = 0usize;
                for 符 in 行.chars() {
                    let 宽 = 字宽!(符).unwrap_or(1);
                    if 字宽 + 宽 > 光标.列.into() {
                        break;
                    }
                    前字宽 = 宽;
                    字宽 += 宽;
                }
                if 前字宽 > 0 {
                    let 新列 = (光标.列 as usize) - 前字宽;
                    光标.列 = 新列.try_into().unwrap();
                    光标.列历史 = 光标.列;
                } else {
                    光标.列 = 0;
                    光标.列历史 = 0;
                }
            }
        }
    }
}

pub fn 提前(
    光标: &mut 光标, 行向量: &[&str], 列数: u16, 落后: bool
) -> io::Result<()> {
    let 行索引 = 光标.行索引 + 光标.行 as usize;
    if 落后 {
        if 行索引 < 行向量.len() {
            // 计算宽度
            let 行长 = 句宽!(行向量[行索引]);
            光标.列 = std::cmp::min(行长, (列数 as usize) - 1)
                .try_into()
                .unwrap_or_default();
            if 行长 != 0 {
                光标.列历史 = 光标.列;
            }
        }
    } else {
        // Home 键：移动到本行第一个字符所在的位置
        let line = 行向量.get(行索引).unwrap_or(&"");
        let mut 第一列: usize = 0;
        for ch in line.chars() {
            if !ch.is_whitespace() {
                break;
            }
            第一列 += 1; // 简化为宽度 1
        }
        光标.列 = std::cmp::min(第一列, (列数 as usize) - 1)
            .try_into()
            .unwrap_or_default();
        光标.列历史 = 光标.列;
    }
    execute!(io::stdout(), 移到(光标.列, 光标.行))?;
    Ok(())
}

// 返回当前所在字符的宽度
pub fn 当前字符宽度(行: &str, 列: u16) -> usize {
    let mut 字宽 = 0usize;
    for 符 in 行.chars() {
        let 宽 = 字宽!(符).unwrap_or(1);
        if 字宽 + 宽 > 列.into() {
            return 宽;
        }
        字宽 += 宽;
    }
    0
}

// 把光标定位到字符起始位置
pub fn 调整列到字符边界(行: &str, 列: u16) -> u16 {
    let mut 字宽 = 0usize;
    for 符 in 行.chars() {
        let 宽 = 字宽!(符).unwrap_or(1);
        if (字宽 + 宽 as usize) > 列.into() {
            // 当前列已超过字字宽度，返回上一个边界
            return (字宽 as u16).try_into().unwrap();
        }
        字宽 += 宽;
    }
    列
}

fn 调整光标列(行向量: &[&str], 光标: &mut 光标) {
    let 行内容 = 行向量[光标.行索引 + 光标.行 as usize];
    光标.列 = std::cmp::min(调整列到字符边界(行内容, 光标.列历史), 句宽!(行内容) as u16);
}

pub fn 字符输入(
    ch: char,
    输入起始行: u16,
    输入区: &mut String,
    光标: &mut 光标,
) -> io::Result<()> {
    // 将字符追加到输入
    输入区.push(ch);
    输入区.push_str(&(&光标.行索引 + &(光标.行 as usize)).to_string());
    output::输入显示(输入区.as_str(), 输入起始行)?;

    Ok(())
}

/// 计算光标位置在缓冲区中的索引（字节偏移）
pub fn 插入位置(光标: &光标, 行向量: Vec<&str>) -> usize {
    let mut 插入位置 = 0usize;
    for (idx, 行) in 行向量.iter().enumerate() {
        if idx < 光标.行索引 + 光标.行 as usize {
            插入位置 += 行.len() + 1; // 包括换行符
        } else if idx == 光标.行索引 + 光标.行 as usize {
            let mut width = 0usize;
            let mut chinese_count = 0usize;
            for ch in 行.chars() {
                let ch_width = if ch.is_ascii() { 1 } else { 2 };
                if width >= 光标.列 as usize {
                    break;
                }
                if !ch.is_ascii() {
                    chinese_count += 1;
                }
                width += ch_width;
            }
            插入位置 += 光标.列 as usize + chinese_count;
            break;
        }
    }
    插入位置
}
