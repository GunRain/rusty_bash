//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::ArithElem;
use crate::elements::subscript::Subscript;

pub fn to_operand(name: &String, sub: &mut Subscript, pre_increment: i64, post_increment: i64,
                   core: &mut ShellCore) -> Result<ArithElem, String> {
    let key = match sub.eval(core, name) {
        Some(s) => s, 
        None => return Err(format!("{}: wrong substitution", &name)),
    };

    let mut value_str = core.db.get_array(name, &key);
    if value_str == "" {
        value_str = "0".to_string();
    }

    let mut value_num = match value_str.parse::<i64>() {
        Ok(n) => n,
        Err(_) => return Err(format!("{}: not an interger", &name)),
    };

    if pre_increment != 0 {
        value_num += pre_increment;
        match set_value(name, &key, value_num, core) {
            Ok(()) => {},
            Err(e) => return Err(e),
        }
    }

    let ans = Ok( ArithElem::Integer(value_num) );

    if post_increment != 0 {
        value_num += post_increment;
        match set_value(name, &key, value_num, core) {
            Ok(()) => {},
            Err(e) => return Err(e),
        }
    }
    ans
}

fn set_value(name: &String, key: &String, new_value: i64,
                     core: &mut ShellCore) -> Result<(), String> {
    let res = match key.parse::<i64>() {
        Ok(n) => {
            if n >= 0 {
                core.db.set_array_elem(name, &(new_value.to_string()), n as usize)
            }else{
                return Err("negative index".to_string());
            }
        },
        Err(_) => core.db.set_assoc_elem(name, &(new_value.to_string()), key),
    };

    if ! res {
        return Err("readonly array".to_string());
    }
    Ok(())
}

