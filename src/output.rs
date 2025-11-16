// src/output.rs

use crate::*;
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