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
    光标: &mut 光标,
    行向量: &[&str],
    行索引: &mut usize,
    最大行数: usize,
    列数: u16,
    上: bool,
) -> io::Result<()> {
    if 行向量.is_empty() {
        return Ok(());
    }
    // 计算新行并限制范围
    let 新行 = ((光标.行 as i32 + if 上 { -1 } else { 1 }).clamp(0, (最大行数 - 1) as i32)) as u16;

    // 更新光标位置
    if 新行 != 光标.行 {
        光标.行 = 新行;
        let 当前行索引 = *行索引 + 光标.行 as usize;
        if 当前行索引 < 行向量.len() {
            let 行长 = 句宽!(行向量[当前行索引]);
            if 行长 == 0 {
                光标.列 = 0;
            } else {
                let 最宽列 = (行长 as u16) - 1;
                if 光标.列 > 最宽列 {
                    光标.列 = 最宽列;
                }
            }
        } else {
            光标.列 = 列数 - 1;
        }
        let 当前行 = 行向量[*行索引 + 光标.行 as usize];
        let 目标列 = 光标.列历史;
        let 新列 = std::cmp::min(调整列到字符边界(当前行, 目标列), 句宽!(当前行) as u16);
        光标.列 = 新列;
    }

    // 向上滚动（当 上 == true）
    if 上 && 光标.行 == 0 && (*行索引 > 0) {
        *行索引 -= 1;
        output::文件显示(&行向量[*行索引..*行索引 + 最大行数], 光标)?;
        // 同样重新定位列
        let 当前行 = 行向量[*行索引 + 光标.行 as usize];
        let 目标列 = 光标.列历史;
        let 新列 = std::cmp::min(调整列到字符边界(当前行, 目标列), 句宽!(当前行) as u16);
        光标.列 = 新列;
        光标.行编号 -= 1;
    }
    // 向下滚动（当 上 == false）
    else if !上 && 光标.行 == (最大行数 as u16 - 1) && (*行索引 + 最大行数) < 行向量.len()
    {
        *行索引 += 1;
        output::文件显示(&行向量[*行索引..*行索引 + 最大行数], 光标)?;
        // 重新定位列（使用列历史）
        let 当前行 = 行向量[*行索引 + 光标.行 as usize];
        let 目标列 = 光标.列历史;
        let 新列 = std::cmp::min(调整列到字符边界(当前行, 目标列), 句宽!(当前行) as u16);
        光标.列 = 新列;
        光标.行编号 += 1;
    }
    // 边界
    else if (*行索引 + 最大行数) == 行向量.len() || *行索引 == 0 {
    }

    Ok(())
}

pub fn 下翻(
    光标: &mut 光标,
    行向量: &[&str],
    行索引: &mut usize,
    最大行数: usize,
    上翻: bool, // true 表示上翻，false 表示下翻
) -> io::Result<()> {
    // 若已在最上方或最底部则无操作
    if (!上翻 && *行索引 == 0) || (上翻 && (*行索引 + 最大行数) >= 行向量.len()) {
        return Ok(());
    }

    // 根据方向决定增减值
    let 变化量 = std::cmp::max(1, 最大行数);
    let 新起始 = if 上翻 {
        (*行索引).saturating_sub(变化量)
    } else {
        (*行索引).saturating_add(变化量)
    };

    // 防止越界
    *行索引 = std::cmp::min(
        新起始,
        if 上翻 {
            光标.行编号 -= 最大行数;
            行向量.len()
        } else {
            光标.行编号 += 最大行数;
            行向量.len() - 最大行数
        },
    );

    // 计算切片结束位置，保证不会超出长度
    let 切片结束 = std::cmp::min(*行索引 + 最大行数, 行向量.len());
    output::文件显示(&行向量[*行索引..切片结束], 光标)?;

    // 重新定位列（使用列历史）
    let 当前行 = 行向量[*行索引 + 光标.行 as usize];
    let 目标列 = 光标.列历史;
    let 新列 = std::cmp::min(调整列到字符边界(当前行, 目标列), 句宽!(当前行) as u16);
    光标.列 = 新列;

    Ok(())
}

pub fn 右移(光标: &mut 光标, 行向量: &[&str], 行索引: &usize, 左: bool) {
    let 当前行索引 = *行索引 + 光标.行 as usize;
    if 当前行索引 < 行向量.len() {
        let 行 = 行向量[当前行索引];
        if 左 {
            // 左移
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
        } else {
            // 右移
            let 字宽 = 当前字符宽度(行, 光标.列);
            let 新列 = 光标.列 + 字宽 as u16;
            let 行长 = 句宽!(行);
            if 新列 <= (行长 as u16) {
                光标.列 = 新列;
                光标.列历史 = 光标.列;
            }
        }
    }
}

pub fn 提前(
    //home&end
    光标: &mut 光标,
    行向量: &[&str],
    行索引: usize,
    列数: u16,
    落后: bool,
) -> io::Result<()> {
    let 当前行索引 = 行索引 + 光标.行 as usize;
    if 落后 {
        if 当前行索引 < 行向量.len() {
            // 计算宽度
            let 行长 = 句宽!(行向量[当前行索引]);
            光标.列 = std::cmp::min(行长, (列数 as usize) - 1)
                .try_into()
                .unwrap_or_default();
            if 行长 != 0 {
                光标.列历史 = 光标.列;
            }
        }
    } else {
        // Home 键：移动到本行第一个字符所在的位置
        let line = 行向量.get(当前行索引).unwrap_or(&"");
        // 找出第一非空格字符的宽度（假设每个字符宽 1）
        let mut first_col: usize = 0;
        for ch in line.chars() {
            if !ch.is_whitespace() {
                break;
            }
            first_col += 1; // 简化为宽度 1
        }
        光标.列 = std::cmp::min(first_col, (列数 as usize) - 1)
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

pub fn 字符输入(
    ch: char,
    输入起始行: u16,
    输入区: &mut String,
) -> io::Result<()> {
    // 将字符追加到输入
    输入区.push(ch);
    output::输入显示(输入区.as_str(), 输入起始行)?;

    Ok(())
}
