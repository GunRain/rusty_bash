//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::Pid;
use crate::elements::command::Command;
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};

//[1]+  Running                 sleep 5 &
#[derive(Clone,Debug)]
pub struct Job {
    pub pids: Vec<Pid>,
    pub async_pids: Vec<Pid>, //maybe not required.
    pub text: String,
    pub status: char, // S: stopped, R: running, D: done, I: invalid, F: fg
    pub id: usize,
    pub priority: u32,
//    pub mark: char, // '+': current, '-': previous, ' ': others
}

impl Job {
    pub fn new(text: &String, commands: &Vec<Box<dyn Command>>, is_bg: bool) -> Job {
        let mut pids = vec![];
        for c in commands {
            if let Some(p) = c.get_pid() {
                pids.push(p);
            }
        }

        Job {
            pids: pids,
            async_pids: vec![],
            text: text.clone(),
            status: if is_bg {'R'}else{'F'},
            //is_bg: is_bg,
            //is_waited: false,
            id: 0,
            //mark: ' ',
            priority: 0, 
        }
    }

    pub fn check_of_finish(&mut self) -> bool {
        if self.status != 'R' {
            return true; 
        }

        let mut remain = vec![];

        while self.async_pids.len() > 0 {
            let p = self.async_pids.pop().unwrap();

            if ! Self::check_async_process(p){
                remain.push(p);
            }
        }

        if remain.len() == 0 {
            self.status = 'D';
        }

        self.async_pids = remain;

        self.async_pids.len() == 0 // true if finished
    }

    pub fn status_string(&self, first: usize, second: usize) -> String {
        let mark = if self.id == first {
            '+'
        }else if self.id == second {
            '-'
        }else{
            ' '
        };

        let status = match self.status {
            'D' => "Done",
            'S' => "Stopped",
            'R' => "Running",
            _   => "ERROR",
        };
        format!("[{}]{} {}\t\t{}", &self.id, mark, status, &self.text.trim_end())
    }

    pub fn print_status(&mut self, first: usize, second: usize) {
        if self.status == 'I' {
            return;
        }

        println!("{}", &self.status_string(first, second));
        if self.status == 'D' {
            self.status = 'I';
        }
    }

    pub fn check_async_process(pid: Pid) -> bool {
        match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) =>  false,
            Ok(_)                      => true, 
            _                          => {eprintln!("ERROR");true},
        }
    }
}
