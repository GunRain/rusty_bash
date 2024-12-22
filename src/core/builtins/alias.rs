//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn alias(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() == 1 {
        for (k, v) in &core.db.aliases {
            println!("alias {}='{}'", k, v);
        }
        return 0;
    }

    if args.len() == 2 && args[1].find("=") != None {
        let kv: Vec<String> = args[1].split("=").map(|t| t.to_string()).collect();
        core.db.aliases.insert(kv[0].clone(), kv[1..].join("="));
    }

    0
}

pub fn unalias(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        println!("unalias: usage: unalias [-a] name [name ...]");
    }

    if args.contains(&"-a".to_string()) {
        core.db.aliases.clear();
        return 0;
    }

    for alias in &mut args[1..] {
        dbg!("{:?}", &alias);
        core.db.aliases.remove_entry(alias);
    }

    /*
    if args.len() == 2 && args[1].find("=") != None {
        let kv: Vec<String> = args[1].split("=").map(|t| t.to_string()).collect();
        core.db.aliases.insert(kv[0].clone(), kv[1..].join("="));
    }*/

    0
}

