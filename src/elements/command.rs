//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod simple;

use crate::Feeder;
use crate::ShellCore;
use self::simple::SimpleCommand;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("COMMAND").finish()
    }
}

pub trait Command {
    fn exec(&mut self, core: &mut ShellCore);
    fn get_text(&self) -> String;
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Command>> {
    if let Some(a) = SimpleCommand::parse(feeder, core){
        Some(Box::new(a))
    }else{
        None
    }
}
