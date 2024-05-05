//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::elements::command::Command;
use std::env;
use std::collections::HashMap;

pub struct Data {
    pub parameters: HashMap<String, String>,
    pub arrays: HashMap<String, Vec<String>>,
    pub position_parameters: Vec<String>,
    pub aliases: HashMap<String, String>,
    pub functions: HashMap<String, Box<dyn Command>>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            parameters: HashMap::new(),
            arrays: HashMap::new(),
            position_parameters: vec![],
            aliases: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_param_ref(&mut self, key: &str) -> &str {
        if let Some(n) = self.get_position_param_pos(key) {
            return &self.position_parameters[n];
        }

        if  self.parameters.get(key) == None {
            if let Ok(val) = env::var(key) {
                self.set_param(key, &val);
            }
        }

        match self.parameters.get(key) {
            Some(val) => val,
            None      => "",
        }
    }

    pub fn get_array(&mut self, key: &str, pos: String) -> String {
        if  self.arrays.get(key) == None {
            return "".to_string();
        }

        match self.arrays.get(key) {
            Some(a) => {
                if pos == "@" {
                    return a.join(" ");
                }

                match pos.parse::<usize>() {
                    Ok(n) => {
                        if n < a.len() {
                            return a[n].clone();
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }


        "".to_string()
    }

    fn get_position_param_pos(&self, key: &str) -> Option<usize> {
        if ! (key.len() == 1 && "0" <= key && key <= "9") {
            return None;
        }

        let n = key.parse::<usize>().unwrap();
        if n < self.position_parameters.len() {
            Some(n)
        }else{
            None
        }
    }

    pub fn set_param(&mut self, key: &str, val: &str) {
        match env::var(key) {
            Ok(_) => env::set_var(key, val),
            _     => {},
        }

        self.parameters.insert(key.to_string(), val.to_string());
    }

    pub fn set_array(&mut self, key: &str, vals: &Vec<String>) {
        self.arrays.insert(key.to_string(), vals.to_vec());
    }
}