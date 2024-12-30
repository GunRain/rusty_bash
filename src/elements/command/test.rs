//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Redirect};
use crate::elements::command;
use crate::elements::expr::conditional::ConditionalExpr;
use crate::elements::expr::conditional::elem::CondElem;
use crate::utils::error;

#[derive(Debug, Clone, Default)]
pub struct TestCommand {
    text: String,
    cond: Option<ConditionalExpr>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for TestCommand {
    fn run(&mut self, core: &mut ShellCore, _: bool) {
        match self.cond.clone().unwrap().eval(core) {
            Ok(CondElem::Ans(true))  => core.db.set_param("?", "0"),
            Ok(CondElem::Ans(false)) => core.db.set_param("?", "1"),
            Err(err_msg)  => {
                error::print(&err_msg, core);
                core.db.set_param("?", "2")
            },
            _  => {
                error::print("unknown error", core);
                core.db.set_param("?", "2")
            },
        } ;
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl TestCommand {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("[[") {
            return None;
        }

        let mut ans = Self::default();
        ans.text = feeder.consume(2);

        match ConditionalExpr::parse(feeder, core) {
            Some(e) => {
                ans.text += &e.text.clone();
                ans.cond = Some(e);
            },
            None => return None,
        }

        if feeder.starts_with("]]") {
            ans.text += &feeder.consume(2);
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            return Some(ans);
        }
    
        None
    }
}
