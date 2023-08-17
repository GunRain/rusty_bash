//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::File;
use std::os::fd::IntoRawFd;
use crate::elements::io;
use crate::{Feeder, ShellCore};

#[derive(Debug)]
pub struct Redirect {
    pub text: String,
    pub symbol: String,
    pub right: String,
}

impl Redirect {
    fn input_file(&mut self, core: &mut ShellCore) -> bool {
        match File::open(&self.right) {
            Ok(f)  => io::replace(f.into_raw_fd(), 0),
            Err(e) => {
                eprintln!("bash: {}: {}", &self.right, &e);
                false
            },
        }
    }

    pub fn connect(&mut self, core: &mut ShellCore) -> bool {
        match self.symbol.as_ref() {
            "<" => self.input_file(core),
            ">" => {
                let fd = File::create(&self.right).unwrap().into_raw_fd();
                io::replace(fd, 1);
                true
            },
            _ => false,
        }
    }

    pub fn new() -> Redirect {
        Redirect {
            text: String::new(),
            symbol: String::new(),
            right: String::new(),
        }
    }

    fn eat_symbol(feeder: &mut Feeder, ans: &mut Self) -> bool {
        let len = feeder.scanner_redirect_symbol();
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();
        len != 0
    }

    fn eat_right(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let blank_len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(blank_len);

        let len = feeder.scanner_word(core);
        ans.right = feeder.consume(len);
        ans.text += &ans.right.clone();
        len != 0
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Redirect> {
        let mut ans = Self::new();

        if Self::eat_symbol(feeder, &mut ans) &&
           Self::eat_right(feeder, &mut ans, core) {
            Some(ans)
        }else{
            None
        }
    }
}
