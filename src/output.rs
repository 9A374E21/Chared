// src/output.rs

use crate::*;

pub fn 文件显示(内容片段: &[String], 光标: &mut 光标) -> io::Result<()> {
    // 隐藏光标，移到起点
    execute!(io::stdout(), Hide, 移到(0, 0))?;

    // 清除需要更新的行
    let 行数 = 内容片段.len() as u16;
    for i in 0..行数 {
        execute!(io::stdout(), 移到(0, i))?;
        execute!(io::stdout(), Clear(ClearType::CurrentLine))?;
    }

    // 写入
    for (i, 行) in 内容片段.iter().enumerate() {
        let line_str = 行.as_str(); // 转成 &str
        execute!(io::stdout(), 移到(0, i as u16))?;
        write!(io::stdout(), "{}", line_str)?;
        writeln!(io::stdout())?;
    }
    io::stdout().flush()?;

    // 光标重新定位
    let 最大行 = 行数.saturating_sub(1);
    // 限制光标行索引，防止越界
    let 有效行索引 = std::cmp::min(光标.行 as usize, 内容片段.len().saturating_sub(1));
    // 取对应行（若无内容则空字符串）
    let 当前行 = if 有效行索引 < 内容片段.len() {
        内容片段[有效行索引].as_str()
    } else {
        ""
    };
    let 最大列 = 当前行.len() as u16;

    execute!(
        io::stdout(),
        移到(
            std::cmp::min(光标.列, 最大列),
            std::cmp::min(光标.行, 最大行)
        ),
        Show
    )?;

    Ok(())
}

pub fn 输入显示(输入区: &str, 输入起始行: u16) -> io::Result<()> {
    // 移到指定行并清除当前行内容
    execute!(io::stdout(), 移到(0, 输入起始行))?;
    execute!(io::stdout(), Clear(ClearType::CurrentLine))?;

    write!(io::stdout(), "{}", 输入区)?;
    io::stdout().flush()?;

    Ok(())
}
