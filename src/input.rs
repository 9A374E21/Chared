// src/input.rs

use crate::*;

pub fn 按键处理(
    event: &Event,
    输入区: &mut String,
    缓冲区: &mut Vec<u8>,
    光标: &mut 光标,
    行向量: &mut Vec<String>,
    可读缓冲区: &mut String,
    输入起始行: u16,
    最大行数: usize,
) -> Result<(), std::io::Error> {
    if let Event::Key(按键) = event
        && 按键.code == 键::Backspace
    {
        // 退格
        if !输入区.is_empty() {
            输入区.pop();
            output::输入显示(输入区.as_str(), 输入起始行)?;
        } else {
            // 当输入区为空时删除原始定位前一个字符
            let 定位 = input::定位(&光标, &行向量);
            if 定位 > 0 {
                缓冲区.remove(定位 - 1);
                control::右移(光标, 行向量, 最大行数, false);
                可读缓冲区.clear();
                可读缓冲区.push_str(&String::from_utf8_lossy(缓冲区).to_string());
                行向量.clear();
                行向量.extend(可读缓冲区.lines().map(|s| s.to_string()));
                output::文件显示(
                    &行向量[光标.行索引..std::cmp::min(光标.行索引 + 最大行数, 行向量.len())],
                    光标,
                )?;
            }
        }
    }
    if let Event::Key(按键) = event
        && 按键.code == 键::Delete
    {
        // 删除定位后一个字符
        let 定位 = input::定位(&光标, &行向量);
        if 定位 < 缓冲区.len() {
            缓冲区.remove(定位);
        } else {
            if 光标.列 == 0 {
                control::滚动(光标, 行向量, 最大行数, false, Some(false)).ok();
            } else {
                control::右移(光标, 行向量, 最大行数, false);
            }
        }
        // 同步行向量并刷新显示
        可读缓冲区.clear();
        可读缓冲区.push_str(&String::from_utf8_lossy(&缓冲区).to_string());
        行向量.clear();
        行向量.extend(可读缓冲区.lines().map(|s| s.to_string()));
        output::文件显示(
            &行向量[光标.行索引..std::cmp::min(光标.行索引 + 最大行数, 行向量.len())],
            光标,
        )?;
    }
    // 回车键输入
    if let Event::Key(按键) = event
        && 按键.code == 键::Enter
    {
        // 若输入区为空，插入换行符；否则插入输入内容
        let 插件字节: Vec<u8> = if 输入区.is_empty() {
            vec![b'\n'] // 换行符
        } else {
            输入区.as_bytes().iter().cloned().collect()
        };

        // 将输入区内容插入原始（使用 splice 插入字节 slice）
        let 定位 = input::定位(&光标, 行向量);
        缓冲区.splice(定位..定位, 插件字节);

        // 同步行向量
        可读缓冲区.clear();
        可读缓冲区.push_str(&String::from_utf8_lossy(缓冲区).to_string());
        行向量.clear();
        行向量.extend(可读缓冲区.lines().map(|s| s.to_string()));

        // 清空输入区并刷新显示
        if 输入区.is_empty() {
            control::下移(光标, &行向量, 最大行数, true, Some(true))?;
        } else {
            // 光标移动到新文本末尾
            光标.列 += 输入区.len() as u16;
            输入区.clear();
            output::输入显示(&输入区, 输入起始行)?;
        }
        output::文件显示(
            &行向量[光标.行索引..std::cmp::min(光标.行索引 + 最大行数, 行向量.len())],
            光标,
        )?;
    }
    Ok(())
}

pub fn 字符输入(
    符: char,
    输入起始行: u16,
    输入区: &mut String,
    _光标: &mut 光标,
) -> io::Result<()> {
    // 将字符追加到输入
    输入区.push(符);
    // 输入区.push_str(&(&光标.行索引 + &(光标.行 as usize)).to_string());
    output::输入显示(输入区.as_str(), 输入起始行)?;

    Ok(())
}

/// 计算光标位置在缓冲区中的索引（字节偏移）
pub fn 定位(光标: &光标, 行向量: &[String]) -> usize {
    let mut 定位 = 0usize;
    for (idx, 行) in 行向量.iter().enumerate() {
        if idx < 光标.行索引 + 光标.行 as usize {
            定位 += 行.len() + 1; // 包括换行符
        } else if idx == 光标.行索引 + 光标.行 as usize {
            let mut 符宽 = 0usize;
            let mut 字数 = 0usize;
            for 符 in 行.chars() {
                let 字宽 = if 符.is_ascii() { 1 } else { 2 };
                if 符宽 >= 光标.列 as usize {
                    break;
                }
                if !符.is_ascii() {
                    字数 += 1;
                }
                符宽 += 字宽;
            }
            定位 += 光标.列 as usize + 字数;
            break;
        }
    }
    定位
}
