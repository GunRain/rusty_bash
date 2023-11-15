//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, Script};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd::Pid;

#[derive(Debug)]
pub struct WhileCommand {
    pub text: String,
    pub condition: Option<Script>,
    pub inner: Option<Script>,
    pub redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for WhileCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        None
    }

    fn get_text(&self) -> String { self.text.clone() }

    fn set_force_fork(&mut self) {
        self.force_fork = true;
    }
}

impl WhileCommand {
    fn new() -> WhileCommand {
        WhileCommand {
            text: String::new(),
            condition: None,
            inner: None,
            redirects: vec![],
            force_fork: false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<WhileCommand> {
        let mut ans = Self::new();
        if command::eat_inner_script(feeder, core, "while", &mut ans.condition) {
            ans.text = "while".to_string() + &ans.condition.as_mut().unwrap().text.clone();
            dbg!("{:?}", &ans);
            dbg!("{:?}", &feeder);
            Some(ans)
        }else{
            None
        }
    }
}