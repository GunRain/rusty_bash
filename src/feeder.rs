//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod term;
mod scanner;

use std::io;
use crate::ShellCore;
use std::sync::atomic::Ordering::Relaxed;

pub enum InputError {
    Interrupt,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Feeder {
    remaining: String,
    backup: Vec<String>,
    pub nest: Vec<(String, Vec<String>)>,
}

impl Feeder {
    pub fn new() -> Feeder {
        Feeder {
            remaining: "".to_string(),
            backup: vec![],
            nest: vec![("".to_string(), vec![])],
        }
    }

    pub fn consume(&mut self, cutpos: usize) -> String {
        let cut = self.remaining[0..cutpos].to_string();
        self.remaining = self.remaining[cutpos..].to_string();

        cut
    }

    pub fn set_backup(&mut self) {
        self.backup.push(self.remaining.clone());
    }

    pub fn pop_backup(&mut self) {
        self.backup.pop().expect("SUSHI INTERNAL ERROR (backup error)");
    }

    pub fn add_backup(&mut self, line: &str) {
        for b in self.backup.iter_mut() {
            if b.ends_with("\\\n") {
                b.pop();
                b.pop();
            }
            *b += &line;
        }
    }

    pub fn rewind(&mut self) {
        self.remaining = self.backup.pop().expect("SUSHI INTERNAL ERROR (backup error)");
    }   

    fn read_line_stdin(core: &mut ShellCore) -> Result<String, InputError> {
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0)  => Err(InputError::Eof), 
            Ok(_)  => Ok(line), 
            Err(e) => {
                eprintln!("sush: error reading input file: {}", &e);
                core.set_param("?", "2");
                core.exit()
            },
        }

        /*
        if len == 0 {
            Err(InputError::Eof)
        }else{
            Ok(line)
        }*/
    }

    fn feed_additional_line_core(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        if core.sigint.load(Relaxed) {
            return Err(InputError::Interrupt);
        }

        let line = if core.has_flag('i') {
            let len_prompt = term::prompt_additional();
            term::read_line_terminal(len_prompt, core)
        }else{
            Self::read_line_stdin(core)
        };

        match line { 
            Ok(ln) => {
                self.add_line(ln.clone());
                self.add_backup(&ln);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub fn feed_additional_line(&mut self, core: &mut ShellCore) -> bool {
        match self.feed_additional_line_core(core) {
            Ok(()) => true,
            Err(InputError::Eof) => {
                eprintln!("sush: syntax error: unexpected end of file");
                core.set_param("?", "2");
                core.exit();
            },
            Err(InputError::Interrupt) => {
                core.set_param("?", "130");
                false
            },
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        let line = if core.has_flag('i') {
            let len_prompt = term::prompt_normal(core);
            term::read_line_terminal(len_prompt, core)
        }else{ 
            Self::read_line_stdin(core)
        };

        match line {
            Ok(ln) => {
                self.add_line(ln);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    fn add_line(&mut self, line: String) {
        match self.remaining.len() {
            0 => self.remaining = line,
            _ => self.remaining += &line,
        };
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.remaining.starts_with(s)
    }

    pub fn len(&self) -> usize {
        self.remaining.len()
    }
}
