use {
    super::{
        error::Error,
        types::{KeyTy, Ty},
    },
    regex::Regex,
    std::collections::HashMap,
};

pub struct Reader<'a> {
    idx: usize,
    tokens: Vec<&'a str>,
    count: usize,
}

impl<'a> Reader<'a> {
    /// Converts a string to an AST
    ///
    /// Returns an error if the AST is not well formed
    pub fn read_str(string: &str) -> Result<Ty, Error> {
        let tokens = tokenize(string);
        let mut reader = Reader::new(tokens);
        reader.read_form()
    }

    /// Converts a single form to an AST
    fn read_form(&mut self) -> Result<Ty, Error> {
        if self.idx + 1 == self.tokens.len() && self.count != 0 {
            return Err(Error::FormEarlyEnd(self.tokens.len(), self.count));
        }
        match self.tokens[self.idx] {
            "(" => {
                self.idx += 1;
                self.count += 1;
                self.read_list()
            }
            "[" => {
                self.idx += 1;
                self.count += 1;
                self.read_list()
            }
            "{" => {
                self.idx += 1;
                self.count += 1;
                self.read_hashmap()
            }
            _ => self.read_atom(),
        }
    }

    /// Converts a Vector [] or List () into an AST
    fn read_list(&mut self) -> Result<Ty, Error> {
        let mut accum = vec![];
        loop {
            match self.tokens[self.idx] {
                "]" => {
                    self.idx += 1;
                    self.count -= 1;
                    return Ok(Ty::Vector(accum));
                }
                ")" => {
                    self.idx += 1;
                    self.count -= 1;
                    return Ok(Ty::List(accum));
                }
                _ => accum.push(self.read_form()?),
            }
        }
    }

    /// Converts a Hashmap {} into an AST
    fn read_hashmap(&mut self) -> Result<Ty, Error> {
        let mut accum = HashMap::new();
        let mut key: Option<KeyTy> = None;
        loop {
            match self.tokens[self.idx] {
                "}" => {
                    self.idx += 1;
                    self.count -= 1;
                    return Ok(Ty::HashMap(accum));
                }
                _ => match key {
                    Some(k) => {
                        key = None;
                        accum.insert(k, self.read_form()?);
                    }
                    None => {
                        key = Some(self.read_form()?.try_into()?);
                    }
                },
            }
        }
    }

    /// Makes an atom into an AST
    fn read_atom(&mut self) -> Result<Ty, Error> {
        let token = self.tokens[self.idx];
        self.idx += 1;

        if let Ok(i) = token.parse::<i32>() {
            return Ok(Ty::Number(i));
        }

        if token.starts_with('"') {
            let mut accum = String::new();
            let mut active_slash = false;
            for c in token.chars().skip(1) {
                if c == '"' && !active_slash {
                    return Ok(Ty::Str(accum));
                } else if c == '\\' && !active_slash {
                    active_slash = true;
                } else if c == 'n' && active_slash {
                    active_slash = false;
                    accum.push('\n')
                } else if c == '"' && active_slash {
                    active_slash = false;
                    accum.push(c)
                } else if c == '\\' && active_slash {
                    active_slash = false;
                    accum.push('\\');
                } else {
                    accum.push(c)
                }
            }
            return Err(Error::Unbalanced);
        }
        if let Some(stripped) = token.strip_prefix(':') {
            return Ok(Ty::Keyword(stripped.to_string()));
        }

        Ok(match token {
            "false" => Ty::False,
            "true" => Ty::True,
            e => Ty::Symbol(e.to_string()),
        })
    }

    fn new(tokens: Vec<&'a str>) -> Reader<'a> {
        Reader { count: 0, idx: 0, tokens }
    }

    /// Convert the AST ty to a string
    ///
    /// `pretty` controls if the string is formatted or not.
    pub fn to_string(ty: &Ty, pretty: bool) -> String {
        match ty {
            Ty::List(list) => {
                let mut accum = vec![];
                for elem in list {
                    accum.push(Self::to_string(elem, pretty));
                }
                format!("({})", accum.join(" "))
            }
            Ty::Vector(list) => {
                let mut accum = vec![];
                for elem in list {
                    accum.push(Self::to_string(elem, pretty));
                }
                format!("[{}]", accum.join(" "))
            }
            Ty::HashMap(map) => {
                let mut accum = vec![];
                for (key, val) in map.iter() {
                    match key {
                        KeyTy::Keyword(k) => accum.push(format!(":{}", k)),
                        KeyTy::Str(k) => accum.push(format!("\"{}\"", k)),
                    }
                    accum.push(Self::to_string(val, pretty));
                }
                format!("{{{}}}", accum.join(" "))
            }
            Ty::True => "true".to_string(),
            Ty::False => "false".to_string(),
            Ty::Number(n) => n.to_string(),
            Ty::Symbol(n) => n.to_string(),
            Ty::Keyword(s) => format!(":{}", s),
            Ty::Str(n) => {
                if pretty {
                    let mut accum = String::new();
                    for c in n.chars() {
                        if c == '"' {
                            accum.push_str("\\\"");
                        } else if c == '\\' {
                            accum.push_str("\\\\");
                        } else if c == '\n' {
                            accum.push_str("\\n");
                        } else {
                            accum.push(c)
                        }
                    }
                    format!("\"{}\"", accum)
                } else {
                    n.clone()
                }
            }
        }
    }
}

/// Split the text into a vector of token strings
fn tokenize(string: &str) -> Vec<&str> {
    let re = Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#)
        .unwrap();
    re.find_iter(string)
        .map(|c| {
            c.as_str()
                .trim_start_matches(|c| c == ',' || c == ' ' || c == '\n')
                .trim_end_matches(|c| c == ',' || c == ' ' || c == '\n')
        })
        .collect()
}
