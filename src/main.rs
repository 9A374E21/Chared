use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size},
};
use std::fs::File;
use std::io::{self, Read, Write};

struct WindowCursor {
    row: u16,
    col: u16,
}

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
    execute!(io::stdout(), MoveTo(0, 0))?;

    // 获取终端尺寸
    let (_cols, rows) = size()?;
    let mut reserved_rows = ((rows as f64) * 0.2).ceil() as u16;
    reserved_rows = std::cmp::max(2, reserved_rows);
    reserved_rows = std::cmp::min(rows, reserved_rows);

    // 下方输入区起始行
    let input_start_row = rows - reserved_rows + 1;

    // **拆分完整内容为行**
    let lines_vec: Vec<&str> = content.lines().collect();
    // 行数转换为 usize
    let max_display_lines: usize = (rows - reserved_rows) as usize;
    let mut start_line_idx = 0; // 当前窗口起始行索引

    // 显示窗口（上方区域）
    display_window(&lines_vec[start_line_idx..start_line_idx + max_display_lines])?;

    // 输入循环（下方区域）
    input_loop(
        input_start_row,
        reserved_rows,
        rows,
        &lines_vec,
        max_display_lines,
        &mut start_line_idx,
    )?;

    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// 打印内容到终端上方窗口。
fn display_window(content_slice: &[&str]) -> io::Result<()> {
    // 1️⃣ 移动光标到上方窗口左上角 (0,0)
    execute!(io::stdout(), MoveTo(0, 0))?;

    // 2️⃣ 遍历每行并写入
    for line in content_slice {
        // 写当前行
        write!(io::stdout(), "{}", line)?;
        // 如果旧行更长，清除剩余字符（直到换行）
        execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        // 换行
        writeln!(io::stdout())?;
    }

    // 3️⃣ 刷新输出确保立即显示
    io::stdout().flush()?;

    // 4️⃣ 光标保持在窗口顶部，避免影响下方输入区
    execute!(io::stdout(), MoveTo(0, 0))?;

    Ok(())
}

/// 处理下方输入区的交互逻辑。
fn input_loop(
    input_start_row: u16,
    reserved_rows: u16,
    rows: u16,
    lines_vec: &[&str],
    max_display_lines: usize,
    start_line_idx: &mut usize,
) -> io::Result<()> {
    let (cols, _total_rows) = size()?;
    // ① 使用 WindowCursor 替代两变量
    let mut cursor = WindowCursor { row: 0, col: 0 };

    let mut has_input = false; // 是否已输入字符
    let mut input_col: u16 = 0; // 当前光标列（下方输入区）

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
                execute!(io::stdout(), MoveTo(0, input_start_row))?;
                execute!(io::stdout(), Clear(ClearType::FromCursorDown))?;
                has_input = false;
                input_col = 0;
                continue;
            }

            // Backspace 删除上一个字符
            if key_event.code == KeyCode::Backspace {
                if has_input && input_col > 0 {
                    execute!(io::stdout(), MoveTo(input_col - 1, input_start_row))?;
                    execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
                    input_col -= 1;
                }
                if input_col == 0 {
                    has_input = false;
                }
                continue;
            }

            // **无输入时**：方向键控制上方窗口光标
            if !has_input
                && matches!(
                    key_event.code,
                    KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down
                )
            {
                match key_event.code {
                    KeyCode::Left => {
                        if cursor.col > 0 {
                            cursor.col -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if cursor.col < cols - 1 {
                            cursor.col += 1;
                        }
                    }
                    KeyCode::Up => {
                        let _max_rows = rows - reserved_rows;
                        if cursor.row > 0 {
                            cursor.row -= 1;
                        } else if *start_line_idx > 0 {
                            *start_line_idx -= 1;
                            display_window(
                                &lines_vec[*start_line_idx..*start_line_idx + max_display_lines],
                            )?;
                            // 光标保持在最顶行
                            cursor.row = 0;
                        }
                    }
                    KeyCode::Down => {
                        let max_rows = rows - reserved_rows;
                        if cursor.row == max_rows - 1
                            && *start_line_idx + max_display_lines < lines_vec.len()
                        {
                            *start_line_idx += 1; // 移动起始行索引
                            display_window(
                                &lines_vec[*start_line_idx..*start_line_idx + max_display_lines],
                            )?;
                        } else if *start_line_idx + max_display_lines == lines_vec.len() {
                            // 已到达文件最后一行，光标不再向下移动
                        } else {
                            cursor.row += 1;
                        }
                    }
                    _ => {} // 通配符处理未使用的键码
                }
                execute!(io::stdout(), MoveTo(cursor.col, cursor.row))?;
                continue;
            }

            // **字符键**：使用 `handle_char`
            if let KeyCode::Char(ch) = key_event.code {
                handle_char(ch, &mut input_col, &mut has_input, input_start_row)?;
            }
        }
    }
}

/// 处理一个字符键，打印并保持光标在下方窗口。
fn handle_char(
    ch: char,
    cursor_col: &mut u16,
    has_input: &mut bool,
    input_start_row: u16,
) -> io::Result<()> {
    // 先定位到正确位置（防止第一次字符被覆盖）
    execute!(io::stdout(), MoveTo(*cursor_col, input_start_row))?;
    print!("{}", ch);
    io::stdout().flush()?; // 确保立即显示
    *cursor_col += 1;
    *has_input = true;
    // 保持光标在下方窗口
    execute!(io::stdout(), MoveTo(*cursor_col, input_start_row))?;
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
