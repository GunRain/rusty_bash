//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod terminal;
mod scanner;

use std::{io, process};
use crate::ShellCore;
use crate::utils::exit;
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
    lineno: usize,
}

impl Feeder {
    pub fn new(s: &str) -> Feeder {
        Feeder {
            remaining: s.to_string(),
            backup: vec![],
            nest: vec![("".to_string(), vec![])],
            lineno: 0,
        }
    }

    pub fn consume(&mut self, cutpos: usize) -> String {
        let cut = self.remaining[0..cutpos].to_string();
        self.remaining = self.remaining[cutpos..].to_string();

        cut
    }

    pub fn refer(&mut self, cutpos: usize) -> &str {
        &self.remaining[0..cutpos]
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

        let len = match io::stdin().read_line(&mut line) {
            Ok(len)  => len,
            Err(why) => {
                eprintln!("sush: {}: {}", &core.script_name, why);
                process::exit(1)
            },
        };

        if len == 0 {
            Err(InputError::Eof)
        }else{
            Ok(line)
        }
    }

    fn feed_additional_line_core(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        if core.sigint.load(Relaxed) {
            return Err(InputError::Interrupt);
        }

        let line = match ! core.read_stdin {
            true  => terminal::read_line(core, "PS2"),
            false => Self::read_line_stdin(core),
        };

        match line { 
            Ok(ln) => {
                self.add_line(ln.clone(), core);
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
                core.data.set_param("?", "2");

                match core.data.flags.contains('S') { //S: on source command
                    true  => return false,
                    false => exit::normal(core),
                }
            },
            Err(InputError::Interrupt) => {
                core.data.set_param("?", "130");
                false
            },
        }
    }

    pub fn feed_line(&mut self, core: &mut ShellCore) -> Result<(), InputError> {
        let line = match ! core.read_stdin {
            true  => terminal::read_line(core, "PS1"),
            false => Self::read_line_stdin(core),
        };

        match line {
            Ok(ln) => {
                self.add_line(ln, core);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    pub fn add_line(&mut self, line: String, core: &mut ShellCore) {
        if core.data.flags.contains('v') {
            eprint!("{}", &line);
        }

        self.lineno += 1;
        core.data.set_param("LINENO", &self.lineno.to_string());
        match self.remaining.len() {
            0 => self.remaining = line,
            _ => self.remaining += &line,
        };
    }

    pub fn replace(&mut self, num: usize, to: &str) {
        self.consume(num);
        self.remaining = to.to_string() + &self.remaining;
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.remaining.starts_with(s)
    }

    pub fn len(&self) -> usize {
        self.remaining.len()
    }
}
