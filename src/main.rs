// src/main.rs

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size},
};
use std::fs::File;
use std::io::{self, Read, Write};

// 光标位置
struct 光标 {
    行: u16,
    列: u16,
}

/// 读取指定文件,返回内容。
fn 读取(path: &str) -> io::Result<String> {
    let mut 文件 = File::open(path)?;
    let mut 缓冲区 = Vec::new();
    文件.read_to_end(&mut 缓冲区)?;
    String::from_utf8(缓冲区).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn 编辑窗口(content: &str) -> io::Result<()> {
    // 初始化终端
    execute!(io::stdout(), Clear(ClearType::All), EnterAlternateScreen)?;
    execute!(io::stdout(), MoveTo(0, 0))?;

    // 终端尺寸
    let (_列数, 行数) = size()?;
    let mut 保留行 = ((行数 as f64) * 0.2).ceil() as u16;
    保留行 = std::cmp::max(2, 保留行);
    保留行 = std::cmp::min(行数, 保留行);

    let 输入起始行 = 行数 - 保留行 + 1;

    // 拆分为行
    let 行向量: Vec<&str> = content.lines().collect();
    // 行数转换为 usize
    let 最大行数: usize = (行数 - 保留行) as usize;
    let mut 起始行索引 = 0; // 当前窗口起始行索引

    // 初始化光标
    let mut 光标 = 光标 { 行: 0, 列: 0 };

    // 显示
    显示(&行向量[起始行索引..起始行索引 + 最大行数], &mut 光标)?;

    输入(
        输入起始行,
        保留行,
        行数,
        &行向量,
        最大行数,
        &mut 起始行索引,
        &mut 光标,
    )?;

    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn 落后(
    光标: &mut 光标, 行向量: &[&str], 起始行索引: usize, 列数: u16
) -> io::Result<()> {
    let 当前行索引 = 起始行索引 + 光标.行 as usize;
    if 当前行索引 < 行向量.len() {
        let 行长 = 行向量[当前行索引].chars().count();
        if 行长 == 0 {
            // 空行，光标放到最左侧
            光标.列 = 0;
        } else {
            let 列数_usize = 列数 as usize;
            let col_pos = if 行长 < 列数_usize {
                行长
            } else {
                列数_usize - 1
            };
            光标.列 = col_pos as u16;
        }
    } else {
        // 超出范围时保持最右边
        光标.列 = 列数 - 1;
    }
    execute!(io::stdout(), MoveTo(光标.列, 光标.行))?;
    Ok(())
}

fn 光标限位(
    光标: &mut 光标, 行向量: &[&str], 起始行索引: usize, 列数: u16
) -> io::Result<()> {
    let 当前行索引 = 起始行索引 + 光标.行 as usize;
    if 当前行索引 < 行向量.len() {
        let 行长 = 行向量[当前行索引].chars().count();
        if 行长 == 0 {
            光标.列 = 0;
        } else {
            let 最宽列 = (行长 as u16) - 1;
            if 光标.列 > 最宽列 + 1 {
                光标.列 = 最宽列 + 1;
            }
        }
    } else {
        // 超出文件范围，保持最右侧
        光标.列 = 列数 - 1;
    }
    Ok(())
}

fn 显示(content_slice: &[&str], 光标: &mut 光标) -> io::Result<()> {
    // 隐藏光标
    execute!(io::stdout(), Hide)?;
    execute!(io::stdout(), MoveTo(0, 0))?;

    // 写入
    for 行 in content_slice {
        // 写当前行
        write!(io::stdout(), "{}", 行)?;
        // 如果旧行更长，清除剩余字符（直到换行）
        execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
        // 换行
        writeln!(io::stdout())?;
    }

    // 刷新显示
    io::stdout().flush()?;

    // 保持光标
    execute!(io::stdout(), MoveTo(光标.列, 光标.行))?;
    execute!(io::stdout(), Show)?;
    Ok(())
}

fn 输入(
    输入起始行: u16,
    保留行: u16,
    行数: u16,
    行向量: &[&str],
    最大行数: usize,
    起始行索引: &mut usize,
    光标: &mut 光标,
) -> io::Result<()> {
    let (列数, _总行数) = size()?;
    let mut 已输入 = false; // 是否已输入字符
    let mut 输入列: u16 = 0; // 当前光标列（下方输入区）

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
                execute!(io::stdout(), MoveTo(0, 输入起始行))?;
                execute!(io::stdout(), Clear(ClearType::FromCursorDown))?;
                已输入 = false;
                输入列 = 0;
                continue;
            }

            // Backspace 删除上一个字符
            if key_event.code == KeyCode::Backspace {
                if 已输入 && 输入列 > 0 {
                    execute!(io::stdout(), MoveTo(输入列 - 1, 输入起始行))?;
                    execute!(io::stdout(), Clear(ClearType::UntilNewLine))?;
                    输入列 -= 1;
                }
                if 输入列 == 0 {
                    已输入 = false;
                }
                continue;
            }

            // **无输入时**：方向键控制上方光标
            if !已输入
                && matches!(
                    key_event.code,
                    KeyCode::Left
                        | KeyCode::Right
                        | KeyCode::Up
                        | KeyCode::Down
                        | KeyCode::Home
                        | KeyCode::End // 新增 Home/End
                )
            {
                match key_event.code {
                    KeyCode::Left => {
                        if 光标.列 > 0 {
                            光标.列 -= 1;
                        }
                    }
                    KeyCode::Right => {
                        // **新增：禁止右移时已在最右侧字符**
                        let 当前行索引 = *起始行索引 + 光标.行 as usize;
                        if 当前行索引 < 行向量.len() {
                            let 行长 = 行向量[当前行索引].chars().count();
                            // 只允许光标在行长度以内（不超出一格）
                            if 光标.列 < (行长 as u16) {
                                光标.列 += 1;
                            }
                        } else {
                            // 超出文件范围，保持当前位置
                        }
                    }
                    KeyCode::Up => {
                        // 计算在当前视窗内可以显示的最大行数
                        let _最大行 = 行数 - 保留行;
                        if 光标.行 > 0 {
                            // 当光标不在第一行时，直接往上移动一行
                            光标.行 -= 1;
                            // **立即调整光标列**，避免跳跃到同一列后再回到末尾
                            光标限位(光标, &行向量, *起始行索引, 列数)?;
                        } else if *起始行索引 > 0 {
                            // 如果光标已位于视窗最顶端，但整个文件还有上一段内容，
                            // 则需要滚动视窗向上并保持光标在视窗顶部
                            *起始行索引 -= 1;
                            显示(&行向量[*起始行索引..*起始行索引 + 最大行数], 光标)?;
                            // **立即调整光标列**，避免跳跃到同一列后再回到末尾
                            光标限位(光标, &行向量, *起始行索引, 列数)?;
                            // 确保光标仍然停留在视窗的第一行（即文件中的上一行）
                            光标.行 = 0;
                        }
                    }
                    KeyCode::Down => {
                        // Calculate the maximum visible line index within the window
                        let 最大行 = 行数 - 保留行;
                        // If cursor is at the last visible line and there are more lines below,
                        // we need to scroll the view down.
                        if 光标.行 == 最大行 - 1 && *起始行索引 + 最大行数 < 行向量.len()
                        {
                            // Move the window start index forward by one line
                            *起始行索引 += 1;

                            // **立即调整光标列**，避免跳跃到同一列后再回到末尾
                            光标限位(光标, &行向量, *起始行索引, 列数)?;

                            // 刷新显示：重新绘制窗口内容
                            显示(&行向量[*起始行索引..*起始行索引 + 最大行数], 光标)?;
                        } else if *起始行索引 + 最大行数 == 行向量.len() {
                            // 已到达文件最后一行，光标不再向下移动
                        } else {
                            // 单行内移动：只改变光标的行位置
                            光标.行 += 1;
                            // **立即调整光标列**，避免跳跃到同一列后再回到末尾
                            光标限位(光标, &行向量, *起始行索引, 列数)?;
                        }
                    }

                    KeyCode::Home => {
                        // Home 键：移至最左列
                        光标.列 = 0;
                    }
                    KeyCode::End => {
                        // 调用公用函数
                        落后(光标, &行向量, *起始行索引, 列数)?;
                    }
                    _ => {}
                }
                execute!(io::stdout(), MoveTo(光标.列, 光标.行))?;
                continue;
            }

            //处理字符键
            if let KeyCode::Char(ch) = key_event.code {
                处理字符键(ch, &mut 输入列, &mut 已输入, 输入起始行)?;
            }
        }
    }
}

/// 处理一个字符键，打印并保持光标在下方窗口。
fn 处理字符键(
    ch: char,
    光标列: &mut u16,
    已输入: &mut bool,
    输入起始行: u16,
) -> io::Result<()> {
    // 先定位到正确位置（防止第一次字符被覆盖）
    execute!(io::stdout(), MoveTo(*光标列, 输入起始行))?;
    print!("{}", ch);
    io::stdout().flush()?; // 确保立即显示
    *光标列 += 1;
    *已输入 = true;
    // 保持光标在下方窗口
    execute!(io::stdout(), MoveTo(*光标列, 输入起始行))?;
    Ok(())
}

fn 运行(args: &[String]) -> io::Result<()> {
    let content = if let Some(path) = args.get(1) {
        读取(path)
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
