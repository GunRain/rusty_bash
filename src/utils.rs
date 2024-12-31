//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod directory;
pub mod clock;
pub mod error;
pub mod exit;
pub mod file;
pub mod file_check;
pub mod glob;
pub mod arg;
pub mod random;

use crate::{Feeder, ShellCore};

pub fn reserved(w: &str) -> bool {
    match w {
        "[[" | "]]" | "{" | "}" | "while" | "for" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi" | "case" => true,
        _ => false,
    }
}

pub fn split_words(s: &str) -> Vec<String> {
    let mut ans = vec![];

    let mut in_quote = false;
    let mut escaped = false;
    let mut quote = ' ';

    let mut tmp = String::new();
    for c in s.chars() {
        if escaped || c == '\\' {
            escaped = ! escaped;
            tmp.push(c);
            continue;
        }

        if c == '\'' || c == '"' {
            if c == quote {
                in_quote = ! in_quote;
                quote = ' ';
            }else if quote == ' ' {
                in_quote = ! in_quote;
                quote = c;
            }
            tmp.push(c);
            continue;
        }

        if in_quote {
            tmp.push(c);
            continue;
        }

        if ! in_quote && ( c == ' ' || c == '\t') {
            if tmp.len() != 0 {
                ans.push(tmp.clone());
                tmp.clear();
            }
        }else{
            tmp.push(c);
        }
    }

    if tmp.len() != 0 {
        ans.push(tmp);
    }

    ans
}

pub fn is_wsl() -> bool {
    if let Ok(info) = nix::sys::utsname::uname() {
        let release = info.release().to_string_lossy().to_string();
        return release.find("WSL").is_some();
    };

    false
}

pub fn is_name(s: &str, core: &mut ShellCore) -> bool {
    let mut f = Feeder::new(s);
    s.len() > 0 && f.scanner_name(core) == s.len()
}

pub fn is_param(s :&String) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_ch = s.chars().nth(0).unwrap();
    if s.len() == 1 { //special or position param
        if "$?*@#-!_0123456789".find(first_ch) != None {
            return true;
        }
    }
    /* variable */
    if '0' <= first_ch && first_ch <= '9' {
        return s.chars().position(|c| c < '0' || '9' < c) == None;
    }

    let name_c = |c| ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z')
                     || ('0' <= c && c <= '9') || '_' == c;
    s.chars().position(|c| !name_c(c)) == None
}
