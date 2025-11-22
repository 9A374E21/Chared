// src/output.rs

use crate::*;

pub fn render(
    输入区: &mut String, 行向量: &[String], 光标: &mut 光标, 保留行: u16
) -> io::Result<()> {
    // 先清除整个屏幕（不必每次全屏，简化实现）
    execute!(io::stdout(), Clear(ClearType::All))?;

    // 获取终端尺寸
    let (总列数, 总行数) = size()?;
    // 上方占 80% 行
    let 上部行数 = 总行数 - 保留行;
    // 下方保留行数（20%）

    // 显示上方区域：从光标的行索引开始，最多显示上部行数
    let start_idx = 光标.行索引 as usize;
    for (idx, line) in 行向量.iter().enumerate() {
        if idx < start_idx {
            continue;
        }
        if idx >= start_idx + 上部行数 as usize {
            break;
        }

        // 截断超出屏幕宽度的行
        let display_line = if line.len() > 总列数 as usize {
            &line[..总列数 as usize]
        } else {
            line
        };

        execute!(io::stdout(), 移到(0, (idx - start_idx) as u16))?;
        write!(io::stdout(), "{}", display_line)?;
    }

    // 下方保留区域：显示输入区并用箭头提示
    let input_y = 上部行数; // 输入区所在行号
    execute!(io::stdout(), 移到(0, input_y))?;
    write!(io::stdout(), ">>{}", 输入区)?;

    // 重新定位光标：限制在屏幕范围内
    let max_row = 上部行数; // 最后可显示的行号
    let cur_row = std::cmp::min(光标.行, max_row);
    let current_line = if cur_row < 行向量.len() as u16 {
        行向量[cur_row as usize].as_str()
    } else {
        ""
    };
    let max_col = current_line.len() as u16;
    execute!(
        io::stdout(),
        移到(std::cmp::min(光标.列, max_col), cur_row as u16),
        Show
    )?;

    io::stdout().flush()
}