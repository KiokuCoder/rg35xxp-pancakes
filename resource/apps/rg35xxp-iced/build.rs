use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 告诉 Cargo 如果这个文件发生变化，重新运行构建脚本
    println!("cargo:rerun-if-changed=assets/material-design-icons/MaterialSymbols.codepoints");

    // 读取源文件
    let content = fs::read_to_string("assets/material-design-icons/MaterialSymbols.codepoints")
        .expect("Failed to read MaterialSymbols.codepoints");

    // 准备生成的 Rust 代码
    let mut generated_code = String::new();
    generated_code.push_str("pub fn get_icon_codepoint(name: &str) -> Option<char> {\n");
    generated_code.push_str("    match name {\n");

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // 解析每一行：例如 "10k e951"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            let name = parts[0];
            let hex = parts[1];

            // 生成 match 分支，将 hex 转换为 Rust 的 Unicode 字符字面量
            // 注意：Rust 的 format! 宏里，{{ 和 }} 用来转义大括号
            generated_code.push_str(&format!("        \"{}\" => Some('\\u{{{}}}'),\n", name, hex));
        }
    }

    generated_code.push_str("        _ => None,\n");
    generated_code.push_str("    }\n");
    generated_code.push_str("}\n");

    // 将生成的代码写入到 OUT_DIR 目录中
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("material_symbols_match.rs");
    fs::write(&dest_path, generated_code).unwrap();
}