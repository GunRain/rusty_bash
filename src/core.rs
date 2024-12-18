//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

pub mod builtins;
pub mod database;
pub mod history;
pub mod jobtable;
pub mod options;

use crate::{child, signal};
use crate::data::DataType;
use self::database::DataBase;
use self::options::Options;
use std::collections::HashMap;
use std::os::fd::{FromRawFd, OwnedFd};
use std::{io, env, path};
use nix::{fcntl, unistd};
use nix::sys::signal::Signal;
use nix::sys::time::{TimeSpec, TimeVal};
use nix::unistd::Pid;
use crate::utils::{error, exit};
use crate::core::jobtable::JobEntry;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct MeasuredTime {
    pub real: TimeSpec, 
    pub user: TimeVal, 
    pub sys: TimeVal, 
}

impl Default for MeasuredTime {
    fn default() -> Self {
        Self {
            real: TimeSpec::new(0,0),
            user: TimeVal::new(0,0),
            sys: TimeVal::new(0,0),
        }
    }
}

#[derive(Default)]
pub struct ShellCore {
    pub db: DataBase,
    rewritten_history: HashMap<usize, String>,
    pub history: Vec<String>,
    pub builtins: HashMap<String, fn(&mut ShellCore, &mut Vec<String>) -> i32>,
    pub sigint: Arc<AtomicBool>,
    pub read_stdin: bool,
    pub word_eval_error: bool,
    pub is_subshell: bool,
    pub source_function_level: i32,
    pub source_level: i32,
    pub eval_level: i32,
    pub loop_level: i32,
    pub break_counter: i32,
    pub continue_counter: i32,
    pub return_flag: bool,
    pub tty_fd: Option<OwnedFd>,
    pub job_table: Vec<JobEntry>,
    pub job_table_priority: Vec<usize>,
    current_dir: Option<path::PathBuf>, // the_current_working_directory
    pub completion_functions: HashMap<String, String>,
    pub completion_actions: HashMap<String, (String, HashMap<String, String>)>, //command, action,
                                                                            //options for compgen
    pub measured_time: MeasuredTime,
    pub options: Options,
    pub shopts: Options,
    pub suspend_e_option: bool,
    pub script_name: String,
}

impl ShellCore {
    pub fn new() -> ShellCore {
        let mut core = ShellCore{
            db: DataBase::new(),
            sigint: Arc::new(AtomicBool::new(false)),
            read_stdin: true,
            options: Options::new_as_basic_opts(),
            shopts: Options::new_as_shopts(),
            script_name: "-".to_string(),
            ..Default::default()
        };

        core.init_current_directory();
        core.set_initial_parameters();
        core.set_builtins();
        signal::ignore(Signal::SIGPIPE);
        signal::ignore(Signal::SIGTSTP);

        core.db.set_param2("PS4", "+ ");

        if unistd::isatty(0) == Ok(true) {
            core.db.flags += "i";
            core.read_stdin = false;
            core.db.set_param2("PS1", "🍣 ");
            core.db.set_param2("PS2", "> ");
            let fd = fcntl::fcntl(0, fcntl::F_DUPFD_CLOEXEC(255))
                .expect("sush(fatal): Can't allocate fd for tty FD");
            core.tty_fd = Some(unsafe{OwnedFd::from_raw_fd(fd)});
        }

        let home = core.db.get_param("HOME").to_string();
        core.db.set_param2("HISTFILE", &(home + "/.sush_history"));
        core.db.set_param2("HISTFILESIZE", "2000");

        core
    }

    fn set_initial_parameters(&mut self) {
        let version = env!("CARGO_PKG_VERSION");
        let profile = env!("CARGO_BUILD_PROFILE");
        let t_arch = env!("CARGO_CFG_TARGET_ARCH");
        let t_vendor = env!("CARGO_CFG_TARGET_VENDOR");
        let t_os = env!("CARGO_CFG_TARGET_OS");
        let machtype = format!("{}-{}-{}", t_arch, t_vendor, t_os);
        let symbol = "rusty_bash";
        let vparts = version.split('.').collect();
        let versinfo = vec![vparts, vec![symbol, profile, &machtype]].concat()
                       .iter().map(|e| e.to_string()).collect();

        self.db.set_param2("BASH_VERSION", &format!("{}({})-{}", version, symbol, profile));
        self.db.set_param2("MACHTYPE", &machtype);
        self.db.set_param2("HOSTTYPE", &t_arch);
        self.db.set_param2("OSTYPE", &t_os);
        self.db.set("BASH_VERSINFO", DataType::from(&versinfo));
    }

    pub fn flip_exit_status(&mut self) {
        match self.db.get_param("?").as_ref() {
            "0" => self.db.set_param2("?", "1"),
            _   => self.db.set_param2("?", "0"),
        };
    }

    pub fn run_builtin(&mut self, args: &mut Vec<String>, special_args: &mut Vec<String>) -> bool {
        if args.len() == 0 {
            exit::internal(" (no arg for builtins)");
        }

        if self.builtins.contains_key(&args[0]) {
            let func = self.builtins[&args[0]];
            args.append(special_args);
            let status = func(self, args);
            self.db.set_layer_param2("?", &status.to_string(), 0);
            return true;
        }

        false
    }

    fn set_subshell_parameters(&mut self) {
        let pid = nix::unistd::getpid();
        self.db.set_layer_param2("BASHPID", &pid.to_string(), 0);
        match self.db.get_param("BASH_SUBSHELL").parse::<usize>() {
            Ok(num) => self.db.set_layer_param2("BASH_SUBSHELL", &(num+1).to_string(), 0),
            Err(_) =>  self.db.set_layer_param2("BASH_SUBSHELL", "0", 0),
        };
    }

    pub fn initialize_as_subshell(&mut self, pid: Pid, pgid: Pid){
        signal::restore(Signal::SIGINT);
        signal::restore(Signal::SIGTSTP);
        signal::restore(Signal::SIGPIPE);

        self.is_subshell = true;
        child::set_pgid(self, pid, pgid);
        self.set_subshell_parameters();
        self.job_table.clear();
    }

    pub fn init_current_directory(&mut self) {
        match env::current_dir() {
            Ok(path) => self.current_dir = Some(path),
            Err(err) => {
                let msg = format!("pwd: error retrieving current directory: {:?}", err);
                error::print(&msg, self);
            },
        }
    }

    pub fn get_current_directory(&mut self) -> Option<path::PathBuf> {
        if self.current_dir.is_none() {
            self.init_current_directory();
        }
        self.current_dir.clone()
    }


    pub fn set_current_directory(&mut self, path: &path::PathBuf) -> Result<(), io::Error> {
        let res = env::set_current_dir(path);
        if res.is_ok() {
            self.current_dir = Some(path.clone());
        }
        res
    }

    pub fn get_ps4(&mut self) -> String {
        let ps4 = self.db.get_param("PS4").trim_end().to_string();
        let mut multi_ps4 = ps4.to_string();
        for _ in 0..(self.source_level + self.eval_level) {
            multi_ps4 += &ps4;
        }

        multi_ps4
    }
}
