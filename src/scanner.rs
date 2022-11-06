//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::element_list::ControlOperator;

pub fn scanner_until_escape(text: &Feeder, from: usize, to: &str) -> usize {
    let mut pos = from;
    let mut escaped = false;
    for ch in text.chars_after(from) {
        if escaped || ch == '\\' {
            escaped = !escaped;
        }else if let Some(_) = to.find(ch) {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_blank(text: &Feeder, from: usize) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if let Some(_) = " \t".find(ch) {
            pos += ch.len_utf8();
        }else{
            break;
        };
    }
    pos
}

pub fn scanner_until(text: &Feeder, from: usize, to: &str) -> usize {
    let mut pos = from;
    for ch in text.chars_after(from) {
        if let Some(_) = to.find(ch) {
            break;
        };
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_varname(text: &Feeder, from: usize) -> usize {
    if text.len() == from {
        return from;
    }else if "?*@$#!-:".chars().any(|c| c == text.nth(from)) {
        return from+1;
    };

    let mut pos = from;
    for ch in text.chars_after(from) {
        if !((ch >= '0' && ch <= '9') || (ch >= 'A' && ch <= 'Z') 
        || (ch >= 'a' && ch <= 'z') || ch == '_' || ch == '-') {
            break;
        }
        pos += ch.len_utf8();
    }
    pos
}

pub fn scanner_control_op(text: &Feeder, from: usize) -> (usize, Option<ControlOperator> ) {
    let mut op = None;
    let mut pos = from;

    if text.len() > from+2  {
        pos = from+3;
        op = if text.compare(from, ";;&") {
            Some(ControlOperator::SemiSemiAnd)
        }else{
            None
        };
    }

    if op == None && text.len() > from + 1  {
        pos = from+2;
        op = if text.compare(from, "||") {
            Some(ControlOperator::Or)
        }else if text.compare(from, "&&") {
            Some(ControlOperator::And)
        }else if text.compare(from, ";;") {
            Some(ControlOperator::DoubleSemicolon)
        }else if text.compare(from, ";&") {
            Some(ControlOperator::SemiAnd)
        }else if text.compare(from, "|&") {
            Some(ControlOperator::PipeAnd)
        }else{
            None
        };

    }

    if op == None && text.len() > from  {
        pos = from+1;
        if text.compare(from, "&") {
            if text.len() > from+1 && text.compare(from+1, ">") {
                return (0, None)
            }
            return (from + 1, Some(ControlOperator::BgAnd));
        } else if text.compare(from, "\n") {
            return (from + 1, Some(ControlOperator::NewLine));
        } else if text.compare(from, "|") {
            return (from + 1, Some(ControlOperator::Pipe));
        } else if text.compare(from, ";") {
            return (from + 1, Some(ControlOperator::Semicolon));
        } else if text.compare(from, "(") {
            return (from + 1, Some(ControlOperator::LeftParen));
        } else if text.compare(from, ")") {
            return (from + 1, Some(ControlOperator::RightParen));
        }
    }

    if op != None && text.len() > pos && text.compare(pos, "\n") {
        pos += 1;
    }

    if op != None{
        return (pos, op);
    }


    (from , None)
}

pub fn scanner_comment(text: &Feeder, from: usize) -> usize {
    if text.len() > from && text.nth_is(from, "#") {
        return scanner_until(text, from, "\n");
    }

    from
}

pub fn scanner_end_paren(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, ")") {
        return from+1;
    }
    return from;
}

/* TODO: these scanners should be summarized. */ 
pub fn scanner_start_paren(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, "(") {
        return from+1;
    }
    return from;
}

pub fn scanner_start_brace(text: &Feeder, from: usize) -> usize {
    if text.len() == 0 {
        return 0;
    }

    if text.nth_is(from, "{") {
        return from+1;
    }
    return from;
}

pub fn scanner_integer(text: &Feeder, from: usize) -> usize {
    if text.len() == from {
        return from;
    }

    let mut pos = from;
    if text.nth(from) == '-' {
        pos += 1;
    }

    for ch in text.chars_after(pos) {
        if ch < '0' || ch > '9' {
            break;
        }

        pos += 1;
    }

    if text.nth(from) == '-' && pos == from+1 {
        from
    }else{
        pos
    }
}
