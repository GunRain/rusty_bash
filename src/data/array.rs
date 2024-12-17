//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

#[derive(Debug, Clone, Default)]
pub struct ArrayData {
    pub data: Vec<String>,
}

impl From<Vec<String>> for ArrayData {
    fn from(v: Vec<String>) -> Self {
        Self {
            data: v,
            ..Default::default()
        }
    }
}

impl ArrayData {
    pub fn set(&mut self, pos: usize, val: &String) -> bool {
        if self.data.len() > pos {
            self.data[pos] = val.clone();
            true
        }else{
            false
        }
    }

    pub fn get(&self, pos: usize) -> Option<String> {
        match pos < self.data.len() {
            true  => Some(self.data[pos].clone()),
            false => None,
        }
    }

    pub fn get_all(&self) -> Vec<String> {
        self.data.clone()
    }

    pub fn join(&self, space: &str) -> String {
        self.data.join(space)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn print(&self, k: &str) {
        let mut formatted = String::new();
        formatted += "(";
        for i in 0..self.len() {
            let val = self.get(i).unwrap_or("".to_string());
            formatted += &format!("[{}]=\"{}\" ", i, val).clone();
        };
        if formatted.ends_with(" ") {
            formatted.pop();
        }
        formatted += ")";
        println!("{}={}", k.to_string(), formatted); 
    }
}
