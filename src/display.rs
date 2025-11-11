// src/display.rs


use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{Clear, ClearType},
};
use std::io;
use std::io::Write;

pub fn 显示(content_slice: &[&str], 光标: &mut crate::光标) -> io::Result<()> {
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