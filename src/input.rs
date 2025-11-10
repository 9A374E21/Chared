// src/input.rs

use crossterm::{
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{ClearType, LeaveAlternateScreen, size},
};
use std::io;

pub fn 输入(
    输入起始行: u16,
    保留行: u16,
    行数: u16,
    行向量: &[&str],
    最大行数: usize,
    起始行索引: &mut usize,
    光标: &mut crate::display::光标,
) -> io::Result<()> {
    let (列数, _总行数) = size()?;
    let mut 已输入 = false;
    let mut 输入列: u16 = 0;

    loop {
        let event = match crossterm::event::read() {
            Ok(ev) => ev,
            Err(e) => {
                execute!(io::stdout(), LeaveAlternateScreen)?;
                return Err(e);
            }
        };

        if let Event::Key(key_event) = event
            && key_event.kind == KeyEventKind::Press
        {
            // Ctrl+X 退出
            if key_event.code == KeyCode::Char('x')
                && key_event.modifiers.contains(KeyModifiers::CONTROL)
            {
                break Ok(());
            }

            // Esc 清空输入区
            if key_event.code == KeyCode::Esc {
                execute!(io::stdout(), crossterm::cursor::MoveTo(0, 输入起始行))?;
                execute!(
                    io::stdout(),
                    crossterm::terminal::Clear(ClearType::FromCursorDown)
                )?;
                已输入 = false;
                输入列 = 0;
                continue;
            }

            // Backspace
            if key_event.code == KeyCode::Backspace {
                if 已输入 && 输入列 > 0 {
                    execute!(
                        io::stdout(),
                        crossterm::cursor::MoveTo(输入列 - 1, 输入起始行)
                    )?;
                    execute!(
                        io::stdout(),
                        crossterm::terminal::Clear(ClearType::UntilNewLine)
                    )?;
                    输入列 -= 1;
                }
                if 输入列 == 0 {
                    已输入 = false;
                }
                continue;
            }

            if matches!(
                key_event.code,
                KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Home
                    | KeyCode::End
            ) {
                match key_event.code {
                    KeyCode::Left => {
                        if 光标.列 > 0 {
                            let 当前行索引 = *起始行索引 + 光标.行 as usize;
                            if 当前行索引 < 行向量.len() {
                                let 行 = 行向量[当前行索引];
                                // 找到前一个字符的宽度
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
                                // 若光标不在首字符，则可以左移
                                if prev_w > 0 {
                                    let new_列 = (光标.列 as usize) - prev_w;
                                    光标.列 = new_列.try_into().unwrap();
                                } else {
                                    光标.列 = 0;
                                }
                            }
                        }
                    }
                    KeyCode::Right => {
                        // 当前行索引
                        let 当前行索引 = *起始行索引 + 光标.行 as usize;
                        if 当前行索引 < 行向量.len() {
                            let 行 = 行向量[当前行索引];
                            // 只取当前字符宽度
                            let 符宽_w = crate::display::当前字符宽度(行, 光标.列);
                            // 计算新光标位置（下一个字符起始）
                            let new_列 = 光标.列 + 符宽_w as u16;
                            // 行长度（以字符宽度计）减 1，避免超出最后一列
                            let 行长 = unicode_width::UnicodeWidthStr::width(行);
                            if new_列 <= (行长 as u16) {
                                光标.列 = new_列;
                            }
                        }
                    }
                    KeyCode::Up => {
                        // let _最大行 = 行数 - 保留行;
                        if 光标.行 > 0 {
                            光标.行 -= 1;
                            crate::display::光标限位(光标, &行向量, *起始行索引, 列数)?;
                            // 调整列到字符边界
                            let 当前行 = 行向量[*起始行索引 + 光标.行 as usize];
                            光标.列 = crate::display::调整列到字符边界(当前行, 光标.列);
                        } else if *起始行索引 > 0 {
                            *起始行索引 -= 1;
                            crate::display::显示(
                                &行向量[*起始行索引..*起始行索引 + 最大行数],
                                光标,
                            )?;
                            crate::display::光标限位(光标, &行向量, *起始行索引, 列数)?;
                            // 调整列到字符边界
                            let 当前行 = 行向量[*起始行索引];
                            光标.列 = crate::display::调整列到字符边界(当前行, 光标.列);
                            光标.行 = 0;
                        }
                    }
                    KeyCode::Down => {
                        let 最大行 = 行数 - 保留行;
                        if 光标.行 == (最大行 - 1) && (*起始行索引 + 最大行 as usize) < 行向量.len()
                        {
                            *起始行索引 += 1;
                            crate::display::光标限位(光标, &行向量, *起始行索引, 列数)?;
                            crate::display::显示(
                                &行向量[*起始行索引..*起始行索引 + 最大行数],
                                光标,
                            )?;
                        } else if (*起始行索引 + 最大行 as usize) == 行向量.len() {
                            // 已到达文件最后一行，光标不再向下移动
                        } else {
                            光标.行 += 1;
                            crate::display::光标限位(光标, &行向量, *起始行索引, 列数)?;
                            // 调整列到字符边界
                            let 当前行 = 行向量[*起始行索引 + 光标.行 as usize];
                            光标.列 = crate::display::调整列到字符边界(当前行, 光标.列);
                        }
                    }
                    KeyCode::Home => {
                        crate::display::提前(光标);
                    }
                    KeyCode::End => {
                        crate::display::落后(光标, &行向量, *起始行索引, 列数)?;
                    }
                    _ => {}
                }
                execute!(io::stdout(), crossterm::cursor::MoveTo(光标.列, 光标.行))?;
                continue;
            }

            // 处理字符键
            if let KeyCode::Char(ch) = key_event.code {
                crate::display::处理字符键(ch, &mut 输入列, &mut 已输入, 输入起始行)?;
            }
        }
    }
}



