// src/input.rs

use crate::*;
/// 读取指定文件，返回内容。
pub fn 读取(path: &str) -> io::Result<String> {
    // 打开文件并一次性读取全部内容，保留完整缓冲区
    let mut 文件 = File::open(path)?;
    let mut 缓冲区 = Vec::new();
    文件.read_to_end(&mut 缓冲区)?;
    String::from_utf8(缓冲区).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn 输入(
    输入起始行: u16,
    保留行: u16,
    行数: u16,
    行向量: &[&str],
    最大行数: usize,
    起始行索引: &mut usize,
    光标: &mut crate::control::光标,
) -> io::Result<()> {
    // 原始模式，防字符转义
    enable_raw_mode()?;

    let (列数, _总行数) = size()?;
    let mut 输入列: u16 = 0; // 去掉已输入的标记

    loop {
        let event = match crossterm::event::read() {
            Ok(ev) => ev,
            Err(e) => {
                execute!(io::stdout(), LeaveAlternateScreen)?;
                return Err(e);
            }
        };

        //移回光标
        if let Event::Key(key_event) = event
            && key_event.kind == KeyEventKind::Release
        {
            execute!(io::stdout(), crossterm::cursor::MoveTo(光标.列, 光标.行))?;
            continue;
        }

        if let Event::Key(key_event) = event
            && key_event.kind == KeyEventKind::Press
        {
            // C-x 退出
            if key_event.code == KeyCode::Char('x')
                && key_event.modifiers.contains(KeyModifiers::CONTROL)
            {
                disable_raw_mode()?;
                break;
            }
            // Esc 清空
            if key_event.code == KeyCode::Esc {
                execute!(io::stdout(), crossterm::cursor::MoveTo(0, 输入起始行))?;
                execute!(
                    io::stdout(),
                    crossterm::terminal::Clear(ClearType::FromCursorDown)
                )?;
                输入列 = 0; // 清空输入列
                continue;
            }
            // 退格
            if key_event.code == KeyCode::Backspace {
                if 输入列 > 0 {
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
                // 无需再判断已输入
                continue;
            }

            if matches!(
                key_event.code,
                KeyCode::Home
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::End
                    | KeyCode::PageUp
                    | KeyCode::PageDown
            ) {
                match key_event.code {
                    KeyCode::Home => {
                        crate::control::提前(光标, &行向量, *起始行索引, 列数, false)?;
                    }
                    KeyCode::Left => crate::control::右移(光标, 行向量, 起始行索引, true),
                    KeyCode::Right => crate::control::右移(光标, 行向量, 起始行索引, false),
                    KeyCode::Up => {
                        crate::control::下移(
                            光标,
                            &行向量,
                            起始行索引,
                            最大行数,
                            列数,
                            保留行,
                            行数,
                            true, // 上移
                        )?;
                    }
                    KeyCode::Down => {
                        crate::control::下移(
                            光标,
                            &行向量,
                            起始行索引,
                            最大行数,
                            列数,
                            保留行,
                            行数,
                            false, // 下移
                        )?;
                    }
                    KeyCode::PageUp => {
                        crate::control::翻页(光标, 行向量, 起始行索引, 最大行数,false)?;
                    }
                    KeyCode::PageDown => {
                        crate::control::翻页(光标, 行向量, 起始行索引, 最大行数,true)?;
                    }
                    KeyCode::End => {
                        crate::control::提前(光标, &行向量, *起始行索引, 列数, true)?;
                    }

                    _ => {}
                }
                execute!(io::stdout(), crossterm::cursor::MoveTo(光标.列, 光标.行))?;
                continue;
            }

            // 字符输入
            if let KeyCode::Char(ch) = key_event.code {
                if ch == ' ' {
                    // 空格不产生任何输出，直接跳过
                    continue;
                }
                crate::control::字符输入(ch, &mut 输入列, 输入起始行)?;
            }
        }
    }
    // 退出交替屏幕后返回缓冲区内容
    disable_raw_mode()?;
    Ok(())
}
