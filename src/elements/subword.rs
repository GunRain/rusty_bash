//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod other;
mod single_quoted;
mod braced_param;
mod command;
mod escaped_char;
mod double_quoted;
pub mod parameter;

use crate::{ShellCore, Feeder};
use self::other::OtherSubword;
use self::braced_param::BracedParam;
use self::command::CommandSubstitution;
use self::escaped_char::EscapedChar;
use self::double_quoted::DoubleQuoted;
use self::single_quoted::SingleQuoted;
use self::parameter::Parameter;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub enum SubwordType {
    /* related dollar substitution */
    BracedParameter,
    CommandSubstitution,
    Parameter,
    VarName,
    /* other subwords */
    SingleQuoted,
    DoubleQuoted,
    EscapedChar,
    Other,
}


impl Debug for dyn Subword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}

impl Clone for Box::<dyn Subword> {
    fn clone(&self) -> Box<dyn Subword> {
        self.boxed_clone()
    }
}

fn split_str(s: &str) -> Vec<&str> {
    let mut esc = false;
    let mut from = 0;
    let mut pos = 0;
    let mut ans = vec![];

    for c in s.chars() {
        pos += c.len_utf8();
        if esc || c == '\\' {
            esc = ! esc;
            continue;
        }

        if c == ' ' || c == '\t' || c == '\n' {
            ans.push(&s[from..pos-1]);
            from = pos;
        }
    }

    ans.push(&s[from..]);
    ans
}

pub trait Subword {
    fn get_text(&self) -> &str;
    fn boxed_clone(&self) -> Box<dyn Subword>;
    fn merge(&mut self, _right: &Box<dyn Subword>) {}
    fn substitute(&mut self, _: &mut ShellCore) -> bool {true}

    fn split(&self, _core: &mut ShellCore) -> Vec<Box<dyn Subword>>{
        let splits = split_str(self.get_text());

        if splits.len() < 2 {
            return vec![self.boxed_clone()];
        }

        let mut tmp = OtherSubword::new("", SubwordType::Other);
        let mut copy = |text: &str| {
            tmp.text = text.to_string();
            tmp.boxed_clone()
        };

        splits.iter().map(|s| copy(s)).collect()
    }

    fn make_glob_string(&mut self) -> String {self.get_text().to_string()}
    fn make_unquoted_string(&mut self) -> String { self.get_text().to_string() }
    fn get_type(&self) -> SubwordType;
    fn clear(&mut self) {}
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a) = BracedParam::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = CommandSubstitution::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = SingleQuoted::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = DoubleQuoted::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = EscapedChar::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = Parameter::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = OtherSubword::parse(feeder, core){ Some(Box::new(a)) }
    else{ None }
}
