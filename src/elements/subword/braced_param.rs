//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod alternative;
mod offset;
mod remove;
mod replace;

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::utils;
use super::simple::SimpleSubword;

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    pub text: String,
    pub name: String,
    unknown: String,
    is_array: bool,
    subscript: Option<Subscript>,
    has_alternative: bool,
    alternative_symbol: Option<String>,
    alternative_value: Option<Word>,
    num: bool,
    indirect: bool,
    has_offset: bool,
    offset: Option<ArithmeticExpr>,
    has_length: bool,
    length: Option<ArithmeticExpr>,
    has_remove_pattern: bool,
    remove_symbol: String,
    remove_pattern: Option<Word>,
    has_replace: bool,
    replace_from: Option<Word>,
    has_replace_to: bool,
    replace_to: Option<Word>,
    all_replace: bool,
    head_only_replace: bool,
    tail_only_replace: bool,
    array: Vec<String>,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        if ! self.check() {
            return false;
        }

        if self.indirect {
            let value = core.db.get_param(&self.name).unwrap_or_default();
            if utils::is_param(&value) {
                self.name = value;
            }else{
                eprintln!("sush: {}: invalid name", &value);
                return false;
            }
        }

        if self.subscript.is_some() {
            if self.name == "@" {
                eprintln!("sush: {}: bad substitution", &self.text);
                return false;
            }
            return self.subscript_operation(core);
        }

        if self.name == "@" && self.has_offset {
            return offset::set_partial_position_params(self, core);
        }

        let value = core.db.get_param(&self.name).unwrap_or_default();
        self.text = match self.num {
            true  => value.chars().count().to_string(),
            false => value.to_string(),
        };

        self.optional_operation(core)
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn get_alternative_subwords(&self) -> Vec<Box<dyn Subword>> {
        match self.alternative_value.as_ref() {
            Some(w) => w.subwords.to_vec(),
            None    => vec![],
        }
    }

    fn is_array(&self) -> bool {self.is_array}
    fn get_array_elem(&self) -> Vec<String> {self.array.clone()}
}

impl BracedParam {
    fn check(&mut self) -> bool {
        if self.name.is_empty() || ! utils::is_param(&self.name) {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }
        if self.unknown.len() > 0 
        && ! self.unknown.starts_with(",") {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }
        true
    }

    fn subscript_operation(&mut self, core: &mut ShellCore) -> bool {
        let index = match self.subscript.clone().unwrap().eval(core, &self.name) {
            Some(s) => s,
            None => return false,
        };

        if core.db.is_assoc(&self.name) {
            return self.subscript_operation_assoc(core, &index);
        }

        if index.as_str() == "@" {
            self.array = core.db.get_array_all(&self.name);
        }

        self.text = match (self.num, index.as_str()) {
            (true, "@") => core.db.len(&self.name).to_string(),
            (true, _)   => core.db.get_array_elem(&self.name, &index).unwrap().chars().count().to_string(),
            (false, _)  => core.db.get_array_elem(&self.name, &index).unwrap(),
        };
        self.optional_operation(core)
    }

    fn subscript_operation_assoc(&mut self, core: &mut ShellCore, index: &str) -> bool {
        if let Ok(s) = core.db.get_array_elem(&self.name, index) {
            self.text = s;
            return true;
        }
        false
    }

    fn optional_operation(&mut self, core: &mut ShellCore) -> bool {
        if self.has_offset {
            offset::set(self, core)
        }else if self.has_alternative {
            alternative::set(self, core)
        }else if self.has_remove_pattern {
            remove::set(self, core)
        }else if self.has_replace {
            replace::set(self, core)
        }else {
            true
        }
    }

    fn eat_subscript(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(s) = Subscript::parse(feeder, core) {
            ans.text += &s.text;
            if s.text.contains('@') {
                ans.is_array = true;
            }
            ans.subscript = Some(s);
            return true;
        }

        false
    }

    fn eat_offset(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with(":") {
            return false;
        }
        ans.text += &feeder.consume(1);
        ans.has_offset = true;
        ans.offset = match ArithmeticExpr::parse(feeder, core, true) {
            Some(a) => {
                ans.text += &a.text.clone();
                Self::eat_length(feeder, ans, core);
                Some(a)
            },
            None => None,
        };
        true
    }

    fn eat_remove_pattern(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_parameter_remove_symbol();
        if len == 0 {
            return false;
        }

        ans.remove_symbol = feeder.consume(len);
        ans.text += &ans.remove_symbol.clone();
        ans.has_remove_pattern = true;

        ans.remove_pattern = Some(Self::eat_subwords(feeder, ans, vec!["}"], core));
        true
    }

    fn eat_replace(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with("/") {
            return false;
        }

        ans.text += &feeder.consume(1);
        ans.has_replace = true;
        if feeder.starts_with("/") {
            ans.text += &feeder.consume(1);
            ans.all_replace = true;
        }else if feeder.starts_with("#") {
            ans.text += &feeder.consume(1);
            ans.head_only_replace = true;
        }else if feeder.starts_with("%") {
            ans.text += &feeder.consume(1);
            ans.tail_only_replace = true;
        }

        ans.replace_from = Some(Self::eat_subwords(feeder, ans, vec!["}", "/"], core));

        if ! feeder.starts_with("/") {
            return true;
        }
        ans.text += &feeder.consume(1);
        ans.has_replace_to = true;
        ans.replace_to = Some(Self::eat_subwords(feeder, ans, vec!["}"], core));

        true
    }

    fn eat_length(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        if ! feeder.starts_with(":") {
            return;
        }
        ans.text += &feeder.consume(1);
        ans.has_length = true;
        ans.length = match ArithmeticExpr::parse(feeder, core, true) {
            Some(a) => {
                ans.text += &a.text.clone();
                Some(a)
            },
            None => None,
        };
    }

    fn eat_alternative_value(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_parameter_alternative_symbol();
        if num == 0 {
            return false;
        }

        ans.has_alternative = true;
        let symbol = feeder.consume(num);
        ans.alternative_symbol = Some(symbol.clone());
        ans.text += &symbol;

        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        ans.alternative_value = Some(Self::eat_subwords(feeder, ans, vec!["}"], core));
        true
    }

    /*
    fn eat_blank(len: usize, feeder: &mut Feeder, ans: &mut Self, word: &mut Word) {
        let blank = feeder.consume(len);
        let sw = Box::new(SimpleSubword{ text: blank.clone() });
        word.subwords.push(sw);
        ans.text += &blank.clone();
    }*/

    fn eat_subwords(feeder: &mut Feeder, ans: &mut Self, ends: Vec<&str>, core: &mut ShellCore) -> Word {
        let mut word = Word::default();
        while ! ends.iter().any(|e| feeder.starts_with(e)) {
            if let Some(sw) = subword::parse(feeder, core) {
                ans.text += sw.get_text();
                word.text += sw.get_text();
                word.subwords.push(sw);
            }else{
                let c = feeder.consume(1);
                ans.text += &c;
                word.text += &c;
                word.subwords.push(Box::new(SimpleSubword{text: c}) );
            }

            if feeder.len() == 0 {
                if ! feeder.feed_additional_line(core) {
                    return word;
                }
            }
        }

        word
    }

    fn eat_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len != 0 {
            ans.name = feeder.consume(len);
            ans.text += &ans.name;
            return true;
        }

        let len = feeder.scanner_special_and_positional_param();
        if len != 0 {
            ans.name = feeder.consume(len);
            ans.is_array = ans.name == "@";
            ans.text += &ans.name;
            return true;
        }

        feeder.starts_with("}")
    }

    fn eat_unknown(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        if feeder.len() == 0 {
            feeder.feed_additional_line(core);
        }

        let unknown = match feeder.starts_with("\\}") {
            true  => feeder.consume(2),
            false => {
                let len = feeder.nth(0).unwrap().len_utf8();
                feeder.consume(len)
            },
        };

        ans.unknown += &unknown.clone();
        ans.text += &unknown;
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BracedParam> {
        if ! feeder.starts_with("${") {
            return None;
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        if feeder.starts_with("#") && ! feeder.starts_with("#}") {
            ans.num = true;
            ans.text += &feeder.consume(1);
        }else if feeder.starts_with("!") {
            ans.indirect = true;
            ans.text += &feeder.consume(1);
        }

        if Self::eat_param(feeder, &mut ans, core) {
            Self::eat_subscript(feeder, &mut ans, core);
            let _ = Self::eat_alternative_value(feeder, &mut ans, core) 
                 || Self::eat_offset(feeder, &mut ans, core)
                 || Self::eat_remove_pattern(feeder, &mut ans, core)
                 || Self::eat_replace(feeder, &mut ans, core);
        }

        while ! feeder.starts_with("}") {
            Self::eat_unknown(feeder, &mut ans, core);
        }

        ans.text += &feeder.consume(1);
        Some(ans)
    }
}
