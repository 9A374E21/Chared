// src/main.rs
use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, KeyModifiers, KeyEventKind},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size},
};
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};

/// 读取指定路径的文本文件，并返回其内容。
fn 读取文件(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// 在终端打印字符串并等待用户按键后退出。
fn 编辑窗口(content: &str) -> io::Result<()> {
    // 初始化终端，清屏并进入交替屏幕
    execute!(io::stdout(), Clear(ClearType::All), EnterAlternateScreen)?;
    println!("{}", content);

    let (_cols, rows) = size()?;                    // 获取终端尺寸
    let mut reserved_rows = ((rows as f64) * 0.2).ceil() as u16;
    reserved_rows = std::cmp::max(2, reserved_rows);
    reserved_rows = std::cmp::min(rows, reserved_rows);

    let input_start_row = rows - reserved_rows + 1;   // 下方输入区起始行
    execute!(io::stdout(), MoveTo(0, input_start_row))?;

    let (cols, _total_rows) = size()?;
    let content_rows = rows - reserved_rows;          // 上方窗口可用行数
    let mut top_cursor_row: u16 = 0;
    let mut top_cursor_col: u16 = 0;

    let mut has_input = false;            // 是否已输入字符
    let mut cursor_col: u16 = 0;          // 当前光标列（下方输入区）

    // **辅助函数**：处理一个字符键，打印并保持光标在下方窗口
    fn handle_char(ch: char, cursor_col: &mut u16,
                   has_input: &mut bool,
                   input_start_row: u16) -> io::Result<()> {
        // 先定位到正确位置（防止第一次字符被覆盖）
        execute!(io::stdout(), MoveTo(*cursor_col, input_start_row))?;
        print!("{}", ch);
        io::stdout().flush()?;                 // 确保立即显示
        *cursor_col += 1;
        *has_input = true;
        // 保持光标在下方窗口
        execute!(io::stdout(), MoveTo(*cursor_col, input_start_row))?;
        Ok(())
    }

    loop {
        let event = match crossterm::event::read() {
            Ok(ev) => ev,
            Err(e) => { execute!(io::stdout(), LeaveAlternateScreen)?; return Err(e); }
        };

        if let Event::Key(key_event) = event
            && key_event.kind == KeyEventKind::Press
        {
            // Ctrl+X 退出
            if key_event.code == KeyCode::Char('x')
                && key_event.modifiers.contains(KeyModifiers::CONTROL)
            { break; }

            // Esc 清空输入区
            if key_event.code == KeyCode::Esc {
                execute!(io::stdout(), MoveTo(0, input_start_row))?;
                execute!(io::stdout(), Clear(ClearType::FromCursorDown))?;
                has_input = false;
                cursor_col = 0;
                continue;
            }

            // Backspace 删除上一个字符
            if key_event.code == KeyCode::Backspace {
                if has_input && cursor_col > 0 {
                    execute!(io::stdout(), MoveTo(cursor_col - 1, input_start_row))?;
                    execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
                    cursor_col -= 1;
                }
                if cursor_col == 0 { has_input = false; }
                continue;
            }

            // **无输入时**：方向键控制上方窗口光标
            if !has_input && matches!(key_event.code, KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down) {
                match key_event.code {
                    KeyCode::Left => { if top_cursor_col > 0 { top_cursor_col -= 1; } },
                    KeyCode::Right => { if top_cursor_col < cols - 1 { top_cursor_col += 1; } },
                    KeyCode::Up => { if top_cursor_row > 0 { top_cursor_row -= 1; } },
                    KeyCode::Down => { if top_cursor_row < content_rows - 1 { top_cursor_row += 1; } },
                    _ => {},
                }
                execute!(io::stdout(), MoveTo(top_cursor_col, top_cursor_row))?;
                continue;
            }

            // **字符键**：使用 `handle_char`
            if let KeyCode::Char(ch) = key_event.code {
                handle_char(ch, &mut cursor_col, &mut has_input, input_start_row)?;
            }
        }
    }

    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}



/// 主功能入口：根据命令行参数读取文件或显示空白，然后打印到终端。
fn 运行(args: &[String]) -> io::Result<()> {
    let content = if let Some(path) = args.get(1) {
        // 有路径参数，读取文件内容
        读取文件(path)
    } else {
        // 没有参数，返回空字符串
        Ok(String::new())
    }?;
    编辑窗口(&content)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match 运行(&args) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
