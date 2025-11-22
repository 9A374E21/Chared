// src/control.rs

use crate::*;

macro_rules! 句宽 {
    ($s:expr) => {
        unicode_width::UnicodeWidthStr::width($s)
    };
}
macro_rules! 符宽 {
    ($s:expr) => {
        unicode_width::UnicodeWidthChar::width($s)
    };
}

pub fn 下移(
    光标: &mut 光标,
    行向量: &[String],
    最大行数: usize,
    下: bool,
    归零: Option<bool>,
) -> io::Result<()> {
    // 记录旧的行号
    let 原行 = 光标.行;
    // 计算新行并限制范围
    let 新行 = ((光标.行 as i32 + if 下 { 1 } else { -1 }).clamp(0, (最大行数 - 1) as i32)) as u16;

    // 更新光标位置
    if 新行 != 光标.行 {
        光标.行 = 新行;
        let 当前行号 = 光标.行索引 + 光标.行 as usize;
        if 当前行号 < 行向量.len() {
            let 行长 = 句宽!(行向量[当前行号].as_str());
            if 行长 == 0 {
                光标.列 = 0;
            } else {
                let 最宽列 = 行长 as u16;
                if 光标.列 > 最宽列 {
                    光标.列 = 最宽列;
                }
            }
        }
        调整光标列(行向量, 光标, 归零);
    }
    // 抽离滚动逻辑
    if (下 && 原行 == (最大行数 as u16 - 1)) || (!下 && 原行 == 0) {
        滚动(光标, 行向量, 最大行数, 下, 归零)?;
    }

    Ok(())
}

// 新增函数：处理滚动（上下）
pub fn 滚动(
    光标: &mut 光标,
    行向量: &[String],
    最大行数: usize,
    下: bool,
    归零: Option<bool>,
) -> io::Result<()> {
    // 向下滚动（当 下 == true 且光标已处于底部边界）
    if 下 && (光标.行索引 + 最大行数) < 行向量.len() {
        光标.行索引 += 1;

        调整光标列(行向量, 光标, 归零);
    }
    // 向上滚动（当 下 == false 且光标已处于顶部边界）
    else if !下 && (光标.行索引 > 0) {
        光标.行索引 -= 1;

        调整光标列(行向量, 光标, 归零);
    }
    // 边界（不需要额外处理）
    Ok(())
}

pub fn 下翻(
    光标: &mut 光标,
    行向量: &[String],
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

    调整光标列(行向量, 光标, None);

    Ok(())
}

pub fn 右移(光标: &mut 光标, 行向量: &[String], 最大行数: usize, 右: bool) {
    let 当前行号 = 光标.行索引 + 光标.行 as usize;
    if 当前行号 < 行向量.len() {
        if 右 {
            let 符宽 = 当前字符宽度(行向量[当前行号].as_str(), 光标.列);
            let 行长 = 句宽!(行向量[当前行号].as_str());
            if 光标.列 < (行长 as u16) {
                光标.列 = 光标.列 + 符宽 as u16;
                光标.列历史 = 光标.列;
            } else {
                // 已到最右侧，继续按右键时往下移
                下移(光标, &行向量, 最大行数, true, Some(true)).ok();
            }
        } else {
            if 光标.列 > 0 {
                let mut 符宽 = 0usize;
                let mut 前符宽 = 0usize;
                for 符 in 行向量[当前行号].as_str().chars() {
                    let 宽 = 符宽!(符).unwrap_or(1);
                    if 符宽 + 宽 > 光标.列.into() {
                        break;
                    }
                    前符宽 = 宽;
                    符宽 += 宽;
                }
                if 前符宽 > 0 {
                    let 新列 = (光标.列 as usize) - 前符宽;
                    光标.列 = 新列.try_into().unwrap();
                    光标.列历史 = 光标.列;
                }
            } else {
                // 已到最左侧，继续按左键时往上移
                下移(光标, &行向量, 最大行数, false, Some(false)).ok();
            }
        }
    }
}

pub fn 提前(
    光标: &mut 光标, 行向量: &[String], 列数: u16, 落后: bool
) -> io::Result<()> {
    let 行索引 = 光标.行索引 + 光标.行 as usize;
    if 落后 {
        调整光标列(行向量, 光标, Some(false));
    } else {
        // Home 键：移动到本行第一个字符所在的位置
        let line = 行向量.get(行索引).map(|s| s.as_str()).unwrap_or("");
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
    let mut 符宽 = 0usize;
    for 符 in 行.chars() {
        let 宽 = 符宽!(符).unwrap_or(1);
        if 符宽 + 宽 > 列.into() {
            return 宽;
        }
        符宽 += 宽;
    }
    0
}

// 把光标定位到字符起始位置
fn 调整光标列(行向量: &[String], 光标: &mut 光标, 归零: Option<bool>) {
    // 确保索引不越界
    let 行索引 = std::cmp::min(
        光标.行索引.saturating_add(光标.行 as usize),
        行向量.len().saturating_sub(1),
    );
    // 处理空行向量：若为空则直接返回或使用空字符串
    let 行内容 = match 行向量.get(行索引) {
        Some(s) => s.as_str(),
        _ => "", // 空字符串作为默认值，避免越界
    };

    match 归零 {
        Some(true) => {
            // 归零：将光标列置为 0
            光标.列 = 0;
            光标.列历史 = 光标.列;
        }
        Some(false) => {
            // 归零为 false：定位到本行最后一个字符所在位置
            let 行长 = 句宽!(行内容);
            光标.列 = 行长.try_into().unwrap_or_default();
            if 行长 != 0 {
                光标.列历史 = 光标.列;
            }
        }
        _ => {
            // 未传递归零参数，保持原逻辑
            let mut 符宽 = 0usize;
            for 符 in 行内容.chars() {
                let 宽 = 符宽!(符).unwrap_or(1);
                if (符宽 + 宽 as usize) > 光标.列历史.into() {
                    // 当前列已超过字符宽度，返回上一个边界
                    光标.列 = (符宽 as u16).try_into().unwrap();
                    break;
                }
                符宽 += 宽;
            }
            // 若没有找到更靠前的边界，则保持当前列历史
            光标.列 = std::cmp::min(光标.列, 句宽!(行内容) as u16);
        }
    }
}
