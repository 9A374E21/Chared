// src/main.rs

mod control;
mod input;
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
    pub 列: u16,
    pub 行: u16,
    pub 列历史: u16,
    pub 行索引: usize,
}
struct 缓冲区 {
    原始: Vec<u8>,
    可读: String,
    行向量: Vec<String>,
}

pub fn 读取(path: &str) -> io::Result<String> {
    // 打开文件并一次性读取全部内容，保留完整原始
    let mut 文件 = File::open(path)?;
    let mut 缓冲区 = Vec::new();
    文件.read_to_end(&mut 缓冲区)?;
    // 将可能出现的 Windows 换行符 \r\n 转为 Unix 换行符 \n
    String::from_utf8(缓冲区)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .and_then(|s| Ok(s.replace("\r\n", "\n")))
}

fn main() -> io::Result<()> {
    let 参数: Vec<String> = std::env::args().collect();

    // 初始化缓冲区
    let mut 缓冲区 = 缓冲区 {
        原始: if let Some(path) = 参数.get(1) {
            读取(path)?.into()
        } else {
            Vec::new()
        },
        可读: String::new(),
        行向量: Vec::new(),
    };
    execute!(标准输出(), 进入副屏)?;
    execute!(标准输出(), 移到(0, 0))?;

    let (_列数, 行数) = size()?;
    let 保留行 = ((行数 as f64) * 0.25).ceil() as u16;

    缓冲区.可读 = String::from_utf8(缓冲区.原始.clone()).unwrap();
    缓冲区.行向量 = 缓冲区.可读.lines().map(|s| s.to_string()).collect(); // 生成行向量
    let 最大行数: usize = std::cmp::min((行数 - 保留行) as usize, 缓冲区.行向量.len());

    let mut 输入区 = String::new();

    let mut 光标 = 光标 {
        行: 0,
        列: 0,
        行索引: 0,
        列历史: 0,
    };

    let 文件路径 = 参数.get(1).cloned();

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

        output::render(&mut 输入区, &mut 缓冲区.行向量, &mut 光标, 保留行)?;

        // 移回光标
        if let Event::Key(事件) = event
            && 事件.kind == 按键种类::Release
        {
            execute!(标准输出(), 移到(光标.列, 光标.行))?;
            continue;
        }

        if let Event::Key(按键) = event
            && 按键.kind == 按键种类::Press
        {
            // Ctrl+S 保存
            if let Event::Key(按键) = event
                && 按键.kind == 按键种类::Press
                && 按键.code == 键::Char('s')
                && 按键.modifiers.contains(KeyModifiers::CONTROL)
            {
                if let Some(ref path) = 文件路径 {
                    // 将原始写入文件
                    match File::create(path) {
                        Ok(mut f) => {
                            if let Err(e) = f.write_all(&*缓冲区.原始) {
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
            if 按键.code == 键::Char('x') && 按键.modifiers.contains(KeyModifiers::CONTROL) {
                execute!(标准输出(), 离开副屏)?;
                转义模式()?;
                break;
            }

            // Esc 清空
            if 按键.code == 键::Esc {
                输入区 = "".to_string(); // 清空输入区
            }

            input::按键处理(
                &event,
                &mut 输入区,
                &mut 缓冲区.原始,
                &mut 光标,
                &mut 缓冲区.行向量,
                &mut 缓冲区.可读,
                最大行数,
                列数,
            )?;

            // 字符输入
            if let 键::Char(ch) = 按键.code {
                if ch == ' ' {
                    if 输入区.is_empty() {
                        // 在原始插入空格
                        缓冲区.原始.insert(input::定位(&光标, &缓冲区.行向量), b' ');
                        光标.列 += 1; // 光标往右移动一列
                    } else {
                        // 在输入区写空格并将输入区写入缓冲区
                        input::字符输入(' ', &mut 输入区, &mut 光标)?;
                        // 将输入区内容插入原始（使用 splice 插入字节 slice）
                        let 定位 = input::定位(&光标, &缓冲区.行向量);
                        缓冲区
                            .原始
                            .splice(定位..定位, 输入区.as_bytes().iter().cloned());
                        光标.列 += 输入区.len() as u16;
                        输入区.clear();
                    }
                    // 同步行向量
                    缓冲区.可读 = String::from_utf8_lossy(&缓冲区.原始).to_string();
                    缓冲区.行向量 = 缓冲区.可读.lines().map(|s| s.to_string()).collect();
                } else {
                    // 普通字符输入
                    input::字符输入(ch, &mut 输入区, &mut 光标)?;
                }
            }
        }
    }

    转义模式()?;
    execute!(标准输出(), 离开副屏)?;
    Ok(())
}

