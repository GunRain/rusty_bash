//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{exit, ShellCore, signal};
use crate::utils::error;
use nix::unistd;
use nix::sys::{resource, wait};
use nix::sys::resource::UsageWho;
use nix::sys::signal::Signal;
use nix::sys::wait::{WaitPidFlag, WaitStatus};
use nix::time::{clock_gettime, ClockId};
use nix::unistd::Pid;
use std::sync::atomic::Ordering::Relaxed;
use crate::core::database::data::DataType;

pub fn wait_pipeline(core: &mut ShellCore, pids: Vec<Option<Pid>>,
                     exclamation: bool, time: bool) -> Vec<WaitStatus> {
    if pids.len() == 1 && pids[0] == None {
        if time {
            show_time(core);
        }
        if exclamation {
            core.flip_exit_status();
        }
        exit::check_e_option(core);
        return vec![];
    }

    let mut pipestatus = vec![];
    let mut ans = vec![];
    for pid in &pids {
        let ws = wait_process(core, pid.expect("SUSHI INTERNAL ERROR (no pid)"));
        ans.push(ws);

        pipestatus.push(core.db.get_param("?"));
    }

    if time {
        show_time(core);
    }
    set_foreground(core);
    //core.db.set_layer_array("PIPESTATUS", &pipestatus, 0);
    core.db.set_layer("PIPESTATUS", DataType::from(pipestatus.clone()), 0);

    if core.options.query("pipefail") {
        pipestatus.retain(|e| e != "0");

        if pipestatus.len() != 0 {
            core.db.set_param("?", &pipestatus.last().unwrap());
        }
    }

    if exclamation {
        core.flip_exit_status();
    }

    exit::check_e_option(core);

    ans
}

fn wait_process(core: &mut ShellCore, child: Pid) -> WaitStatus {
    let waitflags = match core.is_subshell {
        true  => None,
        false => Some(WaitPidFlag::WUNTRACED | WaitPidFlag::WCONTINUED)
    };

    let ws = wait::waitpid(child, waitflags);

    let exit_status = match ws {
        Ok(WaitStatus::Exited(_pid, status)) => status,
        Ok(WaitStatus::Signaled(pid, signal, coredump)) => error::signaled(pid, signal, coredump),
        Ok(WaitStatus::Stopped(pid, signal)) => {
            eprintln!("Stopped Pid: {:?}, Signal: {:?}", pid, signal);
            148
        },
        Ok(unsupported) => {
            let msg = format!("Unsupported wait status: {:?}", unsupported);
            error::print(&msg, core);
            1
        },
        Err(err) => {
            let msg = format!("Error: {:?}", err);
            exit::internal(&msg);
        },
    };

    if exit_status == 130 {
        core.sigint.store(true, Relaxed);
    }
    core.db.set_layer_param("?", &exit_status.to_string(), 0); //追加
    ws.expect("SUSH INTERNAL ERROR: no wait status")
}

pub fn set_foreground(core: &ShellCore) {
    let fd = match core.tty_fd.as_ref() {
        Some(fd) => fd,
        _        => return,
    };

    let pgid = unistd::getpgid(Some(Pid::from_raw(0)))
               .expect(&error::internal("cannot get pgid"));

    if unistd::tcgetpgrp(fd) == Ok(pgid) {
        return;
    }

    signal::ignore(Signal::SIGTTOU); //SIGTTOUを無視
    unistd::tcsetpgrp(fd, pgid)
        .expect(&error::internal("cannot get the terminal"));
    signal::restore(Signal::SIGTTOU); //SIGTTOUを受け付け
}

pub fn set_pgid(core :&ShellCore, pid: Pid, pgid: Pid) {
    let _ = unistd::setpgid(pid, pgid);
    if pid.as_raw() == 0 && pgid.as_raw() == 0 { //以下3行追加
        set_foreground(core);
    }
}

fn show_time(core: &ShellCore) {
     let real_end_time = clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();

     let core_usage = resource::getrusage(UsageWho::RUSAGE_SELF).unwrap();
     let children_usage = resource::getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

     let real_diff = real_end_time - core.measured_time.real;
     eprintln!("\nreal\t{}m{}.{:06}s", real_diff.tv_sec()/60,
               real_diff.tv_sec()%60, real_diff.tv_nsec()/1000);
     let user_diff = core_usage.user_time() + children_usage.user_time() - core.measured_time.user;
     eprintln!("user\t{}m{}.{:06}s", user_diff.tv_sec()/60,
               user_diff.tv_sec()%60, user_diff.tv_usec());
     let sys_diff = core_usage.system_time() + children_usage.system_time() - core.measured_time.sys;
     eprintln!("sys \t{}m{}.{:06}s", sys_diff.tv_sec()/60,
               sys_diff.tv_sec()%60, sys_diff.tv_usec());
}
