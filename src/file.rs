// src/file.rs

use std::fs::File;
use std::io::{self, Read};

/// 读取指定文件，返回内容。
pub fn 读取(path: &str) -> io::Result<String> {
    let mut 文件 = File::open(path)?;
    let mut 缓冲区 = Vec::new();
    文件.read_to_end(&mut 缓冲区)?;
    String::from_utf8(缓冲区).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}