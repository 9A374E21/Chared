// src/input.rs

use crossterm::{
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{ClearType, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size},
};
use std::io;

pub fn 输入(
    输入起始行: u16,
    保留行: u16,
    行数: u16,
    行向量: &[&str],
    最大行数: usize,
    起始行索引: &mut usize,
    光标: &mut crate::光标,
) -> io::Result<()> {
    // 原始模式，防字符转义
    enable_raw_mode()?;

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

        // 处理键释放事件：立即把光标定位到上方窗口
        if let Event::Key(key_event) = event
            && key_event.kind == KeyEventKind::Release
        {
            // 将光标移回到当前行/列位置（保持在显示区）
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
                // 恢复原始模式后正常返回
                disable_raw_mode()?;
                break Ok(());
            }
            // Esc 清空
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

            // 退格
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
                    KeyCode::Left => crate::control::左移(光标, 行向量, 起始行索引),
                    KeyCode::Right => crate::control::右移(光标, 行向量, 起始行索引),
                    KeyCode::Up => {
                        crate::control::上移(光标, &行向量, 起始行索引, 最大行数, 列数)?;
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
                        )?;
                    }
                    KeyCode::Home => {
                        crate::control::提前(光标);
                    }
                    KeyCode::End => {
                        crate::control::落后(光标, &行向量, *起始行索引, 列数)?;
                    }
                    _ => {}
                }
                execute!(io::stdout(), crossterm::cursor::MoveTo(光标.列, 光标.行))?;
                continue;
            }

            // 处理字符键
            if let KeyCode::Char(ch) = key_event.code {
                crate::control::处理字符键(ch, &mut 输入列, &mut 已输入, 输入起始行)?;
            }
        }
    }
}
