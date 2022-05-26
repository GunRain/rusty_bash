//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::debuginfo::{DebugInfo};
use crate::elems_in_command::{Arg, Substitution};
use crate::elems_in_arg::{SubArg, SubArgBraced, ArgElem, SubArgSingleQuoted, SubArgDoubleQuoted, SubArgVariable, VarName};
use crate::parser::{arg_delimiter};
use crate::scanner::*;

// single quoted arg or double quoted arg or non quoted arg 
pub fn arg(text: &mut Feeder, expand_brace: bool) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: DebugInfo::init(text),
        subargs: vec!(),
    };

    let sub = if expand_brace{subarg}else{subvalue};

    while let Some(result) = sub(text) {
        ans.text += &(*result).text();
        ans.subargs.push(result);
    };

    Some(ans)
}

pub fn subarg(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = subarg_variable_braced(text)          {Some(Box::new(a))}
    else if let Some(a) = subarg_variable_non_braced(text) {Some(Box::new(a))}
    else if let Some(a) = subarg_braced(text)              {Some(Box::new(a))}
    else if let Some(a) = subarg_normal(text)              {Some(Box::new(a))}
    else if let Some(a) = subarg_single_qt(text)           {Some(Box::new(a))}
    else if let Some(a) = subarg_double_qt(text)           {Some(Box::new(a))}
    else                                                   {None}
}

pub fn subvalue(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = subarg_variable_braced(text)          {Some(Box::new(a))}
    else if let Some(a) = subarg_variable_non_braced(text) {Some(Box::new(a))}
    else if let Some(a) = subvalue_normal(text)            {Some(Box::new(a))}
    else if let Some(a) = subarg_single_qt(text)           {Some(Box::new(a))}
    else if let Some(a) = subarg_double_qt(text)           {Some(Box::new(a))}
    else                                                   {None}
}

pub fn arg_in_brace(text: &mut Feeder) -> Option<Arg> {
    let mut ans = Arg{
        text: "".to_string(),
        pos: DebugInfo::init(text),
        subargs: vec!(),
    };

    if text.match_at(0, ",}"){ // zero length arg
        let tmp = SubArg{
            text: "".to_string(),
            pos: DebugInfo::init(text),
        };
        ans.subargs.push(Box::new(tmp));
        return Some(ans);
    };

    while let Some(result) = subarg_in_brace(text) {
        ans.text += &(*result).text();
        ans.subargs.push(result);
    };

    Some(ans)
}

pub fn subarg_in_brace(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = subarg_variable_braced(text)         {Some(Box::new(a))}
    else if let Some(a) = subarg_variable_non_braced(text){Some(Box::new(a))}
    else if let Some(a) = subarg_braced(text)             {Some(Box::new(a))}
    else if let Some(a) = subarg_single_qt(text)          {Some(Box::new(a))}
    else if let Some(a) = subarg_double_qt(text)          {Some(Box::new(a))}
    else if let Some(a) = subarg_normal_in_brace(text)    {Some(Box::new(a))}
    else{None}
}

pub fn subvalue_normal(text: &mut Feeder) -> Option<SubArg> {
    let pos = scanner_escaped_string(text, 0, " \n\t\"';");
    if pos == 0{
        return None;
    };
    Some( SubArg{text: text.consume(pos), pos: DebugInfo::init(text) } )
}

pub fn subarg_normal(text: &mut Feeder) -> Option<SubArg> {
    let pos = scanner_escaped_string(text, 0, " \n\t\"';{}");
    if pos == 0 {
        return None;
    };
    Some( SubArg{text: text.consume(pos), pos: DebugInfo::init(text) } )
}

pub fn subarg_normal_in_brace(text: &mut Feeder) -> Option<SubArg> {
    if text.match_at(0, ",}"){
        return None;
    };
    
    let pos = scanner_escaped_string(text, 0, ",{}");
    Some( SubArg{ text: text.consume(pos), pos: DebugInfo::init(text) })
}

pub fn subarg_single_qt(text: &mut Feeder) -> Option<SubArgSingleQuoted> {
    if !text.match_at(0, "'"){
        return None;
    };

    let pos = scanner_string(text, 1, "'");
    Some(SubArgSingleQuoted{text: text.consume(pos+1), pos: DebugInfo::init(text)})
}

/* parser for a string such as "aaa${var}" */
pub fn subarg_double_qt(text: &mut Feeder) -> Option<SubArgDoubleQuoted> {
    let backup = text.clone();

    let mut ans = SubArgDoubleQuoted {
        text: "".to_string(),
        pos: DebugInfo::init(text),
        subargs: vec!(),
    };

    if scanner_string(text, 0, "\"") != 0 {
        return None;
    }
    text.consume(1);

    loop {
        if let Some(a) = subarg_variable_braced(text) {
            ans.subargs.push(Box::new(a));
        }else if let Some(a) = subarg_variable_non_braced(text) {
            ans.subargs.push(Box::new(a));
        }else if let Some(a) = string_in_double_qt(text) {
            ans.subargs.push(Box::new(a));
        }else{
            break;
        };
    }

    if scanner_string(text, 0, "\"") != 0 {
        text.rewind(backup);
        return None;
    }
    text.consume(1);

    let mut text = "\"".to_string();
    for a in &ans.subargs {
        text += &a.text();
    }
    ans.text = text + "\"";

    Some(ans)
}

pub fn string_in_double_qt(text: &mut Feeder) -> Option<SubArg> {
    if text.nth(0) == '"' {
        return None;
    };

    let pos = scanner_escaped_string(text, 0, "\"$");
    Some( SubArg{text: text.consume(pos), pos: DebugInfo::init(text)})
}

pub fn subarg_variable_non_braced(text: &mut Feeder) -> Option<SubArgVariable> {
    if !(text.nth(0) == '$') || text.nth(1) == '{' {
        return None;
    };

    let pos = scanner_varname(&text, 1);
    Some(
        SubArgVariable{
            text: text.consume(pos),
            pos: DebugInfo::init(text),
        })
}

pub fn subarg_variable_braced(text: &mut Feeder) -> Option<SubArgVariable> {
    if !(text.nth(0) == '$' && text.nth(1) == '{') {
        return None;
    }

    let pos = scanner_varname(&text, 2);
    if text.nth(pos) == '}' {
        Some( SubArgVariable{ text: text.consume(pos+1), pos: DebugInfo::init(text) })
    }else{
        None
    }
}

pub fn subarg_braced(text: &mut Feeder) -> Option<SubArgBraced> {
    let pos = scanner_string(text, 0, "{");
    if pos != 0 {
        return None;
    }
    text.consume(1);
    
    let mut ans = SubArgBraced {
        text: "{".to_string(),
        pos: DebugInfo::init(text),
        args: vec!(),
    };

    while let Some(arg) = arg_in_brace(text) {
        ans.text += &arg.text.clone();
        ans.args.push(arg); 

        if let Some(_) = arg_delimiter(text, ',') {
            ans.text += ",";
            continue;
        }else if scanner_string(text, 0, "}") == 0{
            text.consume(1);
            ans.text += "}";
            break;
        };
    };

    Some(ans)
}

pub fn substitution(text: &mut Feeder) -> Option<Substitution> {
    let varname_pos = scanner_varname(text, 0);
    let equal_pos = scanner_string(text, varname_pos, "=");
    if equal_pos != varname_pos {
        return None;
    }

    let backup = text.clone();
    let var_part = VarName::new(text, varname_pos);
    text.consume(1); // = 
    if let Some(value_part) = arg(text, false){
        Some(Substitution::new(text, var_part, value_part))
    }else{ // cases where the value goes the next line
        text.rewind(backup);
        None
    }
}
