// src/main.rs

mod control;
mod output;

pub use crossterm::{
    cursor::{Hide, MoveTo as 移到, Show},
    event::{self, Event, KeyCode as 键, KeyEventKind as 按键种类, KeyModifiers},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen as 进入副屏, LeaveAlternateScreen as 离开副屏,
        disable_raw_mode as 转义模式, enable_raw_mode as 原始模式, size,
    },
};
pub use std::{
    fs::File,
    io::{self, Read, Write, stdout as 标准输出},
};

pub struct 光标 {
    pub 行: u16,
    pub 列: u16,
    pub 行编号: usize,
    pub 列历史: u16,
}

pub fn 读取(path: &str) -> io::Result<String> {
    // 打开文件并一次性读取全部内容，保留完整缓冲区
    let mut 文件 = File::open(path)?;
    let mut 缓冲区 = Vec::new();
    文件.read_to_end(&mut 缓冲区)?;
    String::from_utf8(缓冲区).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut 缓冲区: Vec<u8> = if let Some(path) = args.get(1) {
        读取(path)?.into()
    } else {
        Vec::new() // 空缓冲区
    };

    execute!(标准输出(), 进入副屏)?;
    execute!(标准输出(), 移到(0, 0))?;

    let (_列数, 行数) = size()?;
    let mut 保留行 = ((行数 as f64) * 0.25).ceil() as u16;
    保留行 = std::cmp::max(4, 保留行);
    保留行 = std::cmp::min(行数, 保留行);

    let binding = String::from_utf8(缓冲区.clone()).unwrap();
    let 行向量: Vec<&str> = binding.lines().collect();
    let mut 最大行数: usize = std::cmp::min((行数 - 保留行) as usize, 行向量.len());

    let mut 输入区 = String::new();
    let mut 输入起始行 = 行数 - 保留行 + 1;

    let mut 行索引 = 0;

    let mut 光标 = 光标 {
        行: 0,
        列: 0,
        行编号: 0,
        列历史: 0,
    };
    let mut 原高宽 = size()?;

    output::文件显示(
        &行向量[行索引..std::cmp::min(行索引 + 最大行数, 行向量.len())],
        &mut 光标,
    )?;

    let 文件路径 = args.get(1).cloned();

    原始模式()?;
    let (列数, _总行数) = size()?;

    loop {
        let event = match event::read() {
            Ok(ev) => ev,
            Err(e) => {
                execute!(标准输出(), 离开副屏)?;
                return Err(e);
            }
        };

        let (新列数, 新行数) = size()?; // 当前高宽
        let new_size = (新列数, 新行数); // 组合成元组用于比较
        // 每次高宽变化时更新
        if new_size != 原高宽 {
            // 重新计算保留行
            let mut 保留行 = ((新行数 as f64) * 0.25).ceil() as u16;
            保留行 = std::cmp::max(4, 保留行);
            保留行 = std::cmp::min(新行数, 保留行);

            // 更新最大行数
            最大行数 = std::cmp::min((新行数 - 保留行) as usize, 行向量.len());
            输入起始行 = new_size.1 - 保留行 + 1;

            if 新行数 < 原高宽.1 {
                // 清除刷新前残留
                execute!(标准输出(), 移到(0, 0))?;
                execute!(标准输出(), Clear(ClearType::FromCursorDown))?;
            }

            // 刷新文件显示
            output::文件显示(
                &行向量[行索引..std::cmp::min(行索引 + 最大行数, 行向量.len())],
                &mut 光标,
            )?;

            output::输入显示(&输入区, 输入起始行)?;

            原高宽 = (新列数, 新行数);
        }
        // 移回光标
        if let Event::Key(事件) = event
            && 事件.kind == 按键种类::Release
        {
            execute!(标准输出(), 移到(光标.列, 光标.行))?;
            continue;
        }

        if let Event::Key(事件) = event
            && 事件.kind == 按键种类::Press
        {
            // Ctrl+S 保存
            if let Event::Key(事件) = event
                && 事件.kind == 按键种类::Press
                && 事件.code == 键::Char('s')
                && 事件.modifiers.contains(KeyModifiers::CONTROL)
            {
                if let Some(ref path) = 文件路径 {
                    // 将缓冲区写入文件
                    match File::create(path) {
                        Ok(mut f) => {
                            if let Err(e) = f.write_all(&*缓冲区) {
                                execute!(标准输出(), 离开副屏)?;
                                return Err(e);
                            }
                        }
                        Err(e) => {
                            execute!(标准输出(), 离开副屏)?;
                            return Err(e);
                        }
                    }
                }
                // 保存后跳过字符处理
                continue;
            }

            // C-x 退出
            if 事件.code == 键::Char('x') && 事件.modifiers.contains(KeyModifiers::CONTROL) {
                execute!(标准输出(), 离开副屏)?;
                转义模式()?;
                break;
            }

            // Esc 清空
            if 事件.code == 键::Esc {
                输入区 = "".to_string(); // 清空输入区
                output::输入显示(&输入区, 输入起始行)?;
            }
            // 退格
            if 事件.code == 键::Backspace {
                输入区.pop();  
                output::输入显示(&输入区, 输入起始行)?;
            }

            // 光标移动
            if matches!(
                事件.code,
                键::Home
                    | 键::Left
                    | 键::Right
                    | 键::Up
                    | 键::Down
                    | 键::End
                    | 键::PageUp
                    | 键::PageDown
            ) {
                match 事件.code {
                    键::Home => {
                        control::提前(&mut 光标, &行向量, 行索引, 列数, false)?;
                    }
                    键::Left => control::右移(&mut 光标, &行向量, &mut 行索引, true),
                    键::Right => control::右移(&mut 光标, &行向量, &mut 行索引, false),
                    键::Up => {
                        control::下移(
                            &mut 光标,
                            &行向量,
                            &mut 行索引,
                            最大行数,
                            列数,
                            true, // 上移
                        )?;
                    }
                    键::Down => {
                        control::下移(
                            &mut 光标,
                            &行向量,
                            &mut 行索引,
                            最大行数,
                            列数,
                            false, // 下移
                        )?;
                    }
                    键::PageUp => {
                        control::下翻(&mut 光标, &行向量, &mut 行索引, 最大行数, true)?;
                    }
                    键::PageDown => {
                        control::下翻(&mut 光标, &行向量, &mut 行索引, 最大行数, false)?;
                    }
                    键::End => {
                        control::提前(&mut 光标, &行向量, 行索引, 列数, true)?;
                    }

                    _ => {}
                }
                execute!(标准输出(), 移到(光标.列, 光标.行))?;
                continue;
            }

            // 字符输入
            if let 键::Char(ch) = 事件.code {
                if ch == ' ' {
                    // 插入空格逻辑（已修正）
                    let mut 插入位置: usize = 0;

                    for _ in 0..光标.行 as usize {
                        // 跳过当前行的所有字符
                        while 插入位置 < 缓冲区.len() && 缓冲区[插入位置] != b'\n' {
                            插入位置 += 1;
                        }
                        // 若已到达换行符，跳过它（\n）
                        if 插入位置 < 缓冲区.len() {
                            插入位置 += 1; // 跳过 '\n'
                        }
                    }

                    // 在当前行内定位光标列
                    插入位置 += 光标.列 as usize;

                    let 当前行 = 行向量.get(行索引 + 光标.行 as usize).unwrap_or(&"");
                    光标.列 = control::调整列到字符边界(当前行, 光标.列);

                    // 计算插入位置对应的字符宽度（多字节）
                    let 字宽 = control::当前字符宽度(当前行, 光标.列);
                    if 字宽 == 0 || (光标.列 + 1) > 当前行.len() as u16 {
                        return Ok(());
                    }

                    // 在缓冲区插入空格
                    缓冲区.insert(插入位置, b' ');
                    光标.列 += 1;

                    // 同步行向量
                    let binding = String::from_utf8_lossy(&缓冲区).to_string();
                    let 行: Vec<&str> = binding.lines().collect();

                    output::文件显示(
                        &行[行索引..std::cmp::min(行索引 + 最大行数, 行.len())],
                        &mut 光标,
                    )?;
                } else {
                    control::字符输入(ch, 输入起始行, &mut 输入区)?;
                }
            }
        }
    }

    转义模式()?;
    execute!(标准输出(), 离开副屏)?;
    Ok(())
}
