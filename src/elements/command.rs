//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

pub mod arithmetic;
pub mod case;
pub mod simple;
pub mod paren;
pub mod brace;
pub mod r#for;
pub mod test;
pub mod function_def;
pub mod r#while;
pub mod r#if;

use crate::{child, ShellCore, Feeder, Script};
use crate::utils::exit;
use self::arithmetic::ArithmeticCommand;
use self::case::CaseCommand;
use self::simple::SimpleCommand;
use self::paren::ParenCommand;
use self::brace::BraceCommand;
use self::function_def::FunctionDefinition;
use self::r#while::WhileCommand;
use self::r#for::ForCommand;
use self::r#if::IfCommand;
use self::test::TestCommand;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::os::fd::RawFd;
use super::{io, Pipe};
use super::io::redirect::Redirect;
use nix::unistd;
use nix::unistd::{ForkResult, Pid};
use std::os::fd::FromRawFd;

use std::thread;
use std::sync::Arc;

impl Debug for dyn Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("COMMAND").finish()
    }
}

impl Clone for Box::<dyn Command> {
    fn clone(&self) -> Box<dyn Command> {
        self.boxed_clone()
    }
}

pub trait Command {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.force_fork() || pipe.is_connected() {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core);
            None
        }
    }

    fn fork_exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                core.initialize_as_subshell(Pid::from_raw(0), pipe.pgid);
                io::connect(pipe, self.get_redirects(), core);

                self.set_herepipe();

                self.run(core, true);
                exit::normal(core)
            },
            Ok(ForkResult::Parent { child } ) => {
                child::set_pgid(core, child, pipe.pgid);
                pipe.parent_close();

                self.send_herepipe();

                Some(child)
            },
            Err(err) => panic!("sush(fatal): Failed to fork. {}", err),
        }
    }

    fn nofork_exec(&mut self, core: &mut ShellCore) {
        if self.get_redirects().iter_mut().all(|r| r.connect(true, core)){
            self.run(core, false);
        }else{
            core.db.set_param("?", "1");
        }
        self.get_redirects().iter_mut().rev().for_each(|r| r.restore());
    }

    fn run(&mut self, _: &mut ShellCore, fork: bool);
    fn get_text(&self) -> String;
    fn get_redirects(&mut self) -> &mut Vec<Redirect>;
    fn set_force_fork(&mut self);
    fn boxed_clone(&self) -> Box<dyn Command>;
    fn force_fork(&self) -> bool;

    fn set_herepipe(&mut self) {
        for re in self.get_redirects() {
            if re.herepipe.is_some() {
                dbg!("{:?}", &re);
                io::replace(re.herepipe.as_ref().unwrap().recv, 0);
            }
        }
    }

    fn send_herepipe(&mut self) {
        for re in self.get_redirects() {
            if re.herepipe.is_some() {

       //         let text = Arc::clone(&re.right.text); //追加
             
                //thread::spawn(move || {
                let mut f = unsafe { File::from_raw_fd(re.herepipe.as_ref().unwrap().send) };
                //    let mut f = unsafe { File::from_raw_fd(3) };
                    write!(&mut f, "{}\n", &re.right.text);
                    f.flush().unwrap();
                    //f.close();
                //});
                /*
                let f = unsafe { re.herepipe.as_ref().unwrap().send };
                let mut f = BufWriter::new(f);
                */
                //f.write(re.right.text);
                /*
                dbg!("{:?}", &re);
                io::replace(re.herepipe.as_ref().unwrap().send, 1);
                print!("{}", &re.right.text);
                */
            }
        }
        dbg!("END");
    }
    
}

pub fn eat_inner_script(feeder: &mut Feeder, core: &mut ShellCore,
           left: &str, right: Vec<&str>, ans: &mut Option<Script>, permit_empty: bool) -> bool {
    if ! feeder.starts_with(left) {
        return false;
    }
    feeder.nest.push( (left.to_string(), right.iter().map(|e| e.to_string()).collect()) );
    feeder.consume(left.len());
    *ans = Script::parse(feeder, core, permit_empty);
    feeder.nest.pop();
    ! ans.is_none()
}

pub fn eat_blank_with_comment(feeder: &mut Feeder, core: &mut ShellCore, ans_text: &mut String) -> bool {
    let blank_len = feeder.scanner_blank(core);
    if blank_len == 0 {
        return false;
    }
    *ans_text += &feeder.consume(blank_len);

    let comment_len = feeder.scanner_comment();
    *ans_text += &feeder.consume(comment_len);
    true
}

fn eat_redirect(feeder: &mut Feeder, core: &mut ShellCore,
                     ans: &mut Vec<Redirect>, ans_text: &mut String) -> bool {
    if let Some(r) = Redirect::parse(feeder, core) {
        *ans_text += &r.text.clone();
        ans.push(r);
        true
    }else{
        false
    }
}

pub fn eat_redirects(feeder: &mut Feeder, core: &mut ShellCore,
                     ans_redirects: &mut Vec<Redirect>, ans_text: &mut String) {
    loop {
        eat_blank_with_comment(feeder, core, ans_text);
        if ! eat_redirect(feeder, core, ans_redirects, ans_text){
            break;
        }
    }
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Command>> {
    if let Some(a) = FunctionDefinition::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = SimpleCommand::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = IfCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = ArithmeticCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = ParenCommand::parse(feeder, core, false) { Some(Box::new(a)) }
    else if let Some(a) = BraceCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = ForCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = WhileCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = CaseCommand::parse(feeder, core) { Some(Box::new(a)) }
    else if let Some(a) = TestCommand::parse(feeder, core) { Some(Box::new(a)) }
    else{ None }
}
