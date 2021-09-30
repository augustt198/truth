#![feature(stdin_forwarders)]
#![allow(clippy::cast_possible_truncation)]

use std::collections::HashMap;
use std::convert::TryFrom;

type TruthTable = Vec<(HashMap<String, bool>, bool)>;

trait IsAlpha {
    fn is_alpha(&self) -> bool;
}

impl IsAlpha for char {
    fn is_alpha(&self) -> bool {
        ('a'..='z').contains(self) || ('A'..='Z').contains(self)
    }
}

trait RepeatChar {
    fn repeat(self, times: u32) -> String;
}

impl RepeatChar for char {
    fn repeat(self, times: u32) -> String {
        let mut string = String::new();
        let mut i = 0;
        while i < times {
            string.push(self);
            i += 1;
        }
        string
    }
}

struct StringReader {
    pos: u32,
    source: String,
    col: u32,
    line: u32,
}

impl StringReader {
    const fn new(source: String) -> Self {
        Self {
            pos: 0,
            line: 1,
            col: 0,
            source,
        }
    }

    fn peak(&mut self) -> Option<char> {
        if self.pos < u32::try_from(self.source.len()).unwrap() {
            self.source.chars().nth(self.pos as usize)
        } else {
            None
        }
    }

    fn read(&mut self) -> Option<char> {
        let next = self.peak();
        if let Some(next) = next {
            if next == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
        next
    }
}

struct ErrorPosition {
    msg: String,
    line: u32,
    col_range: (u32, u32),
}

impl ErrorPosition {
    const fn from_token(msg: String, tok: &Token) -> Self {
        Self {
            msg,
            line: tok.line,
            col_range: (tok.col, tok.col),
        }
    }
}

struct Lexer {
    reader: StringReader,
}

impl Lexer {
    const fn tok(&self, token_type: Type) -> Token {
        Token {
            token_type,
            col: self.reader.col,
            line: self.reader.line,
        }
    }

    fn next_token(&mut self) -> Result<Token, ErrorPosition> {
        loop {
            if let Some(c) = self.reader.read() {
                return match c {
                    '(' => Ok(self.tok(Type::LParen)),
                    ')' => Ok(self.tok(Type::RParen)),
                    '&' | '*' => Ok(self.tok(Type::And)),
                    '|' | '+' => Ok(self.tok(Type::Or)),
                    '!' | '~' => Ok(self.tok(Type::Not)),
                    '^' => Ok(self.tok(Type::Xor)),
                    ' ' | '\n' => continue,
                    _ => {
                        if c.is_alpha() {
                            Ok(self.next_ident(c))
                        } else {
                            Err(ErrorPosition {
                                msg: format!("Unexpected character: {}", c),
                                line: self.reader.line,
                                col_range: (self.reader.col, self.reader.col),
                            })
                        }
                    }
                };
            }
            return Ok(self.tok(Type::Eof));
        }
    }

    fn next_ident(&mut self, current: char) -> Token {
        let mut string = String::new();
        string.push(current);

        while let Some(peak) = self.reader.peak() {
            if peak.is_alpha() {
                string.push(peak);
                self.reader.read();
            } else {
                break;
            }
        }

        self.tok(Type::Ident(string))
    }
}

#[derive(Clone, Debug)]
enum Type {
    LParen,
    RParen,

    Ident(String),

    And,
    Or,
    Not,
    Xor,

    Eof,
}

#[derive(Clone, Debug)]
struct Token {
    token_type: Type,
    col: u32,
    line: u32,
}

#[derive(Debug)]
struct Operation {
    components: Vec<Component>,
    ops: Vec<Token>,
}

impl Operation {
    fn eval(&self, env: &Environment) -> Result<bool, ErrorPosition> {
        let mut value = if self.components.is_empty() {
            false
        } else {
            self.components[0].eval(env)?
        };

        for idx in 1..self.components.len() {
            let eval = self.components[idx].eval(env)?;
            //let token = self.ops[idx - 1];
            match self.ops[idx - 1].token_type {
                Type::And => value &= eval,
                Type::Or => value |= eval,
                Type::Xor => value ^= eval,
                ref other => {
                    return Err(ErrorPosition::from_token(
                        format!("Unexpected operation: {:#?}", other),
                        &self.ops[idx - 1],
                    ))
                }
            }
        }

        Ok(value)
    }

    fn get_variables(&self) -> Vec<String> {
        let mut vars: Vec<String> = Vec::new();

        for component in &self.components {
            match component.value {
                VarOrExpr::Var(ref var) => {
                    if !vars.contains(var) {
                        vars.push(var.clone());
                    }
                }
                VarOrExpr::Expr(ref op) => {
                    let other_vars = op.get_variables();
                    for var in &other_vars {
                        if !vars.contains(var) {
                            vars.push(var.clone());
                        }
                    }
                }
            }
        }

        vars
    }

    fn truth_table(&self) -> Result<TruthTable, ErrorPosition> {
        let mut result = Vec::new();

        let vars = self.get_variables();
        let tests = 2_i32.pow(vars.len() as u32);

        for num in 0..tests {
            let mut env = Environment {
                vars: HashMap::new(),
            };
            for pos in 0..vars.len() {
                env.vars.insert(
                    vars[pos].clone(),
                    ((num >> (vars.len() - 1 - pos)) & 1) == 1,
                );
            }
            result.push((env.vars.clone(), self.eval(&env)?));
        }

        Ok(result)
    }
}

#[derive(Debug)]
enum VarOrExpr {
    Var(String),
    Expr(Operation),
}

#[derive(Debug)]
struct Component {
    value: VarOrExpr,
    negated: bool,
}

impl Component {
    fn eval(&self, env: &Environment) -> Result<bool, ErrorPosition> {
        let mut val = match self.value {
            VarOrExpr::Var(ref name) => env.get_variable(name),
            VarOrExpr::Expr(ref op) => op.eval(env)?,
        };
        if self.negated {
            val = !val;
        };
        Ok(val)
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: u32,
}

impl Parser {
    fn new(lexer: &mut Lexer) -> Result<Self, ErrorPosition> {
        let mut tokens = vec![];
        let mut token;
        loop {
            token = lexer.next_token()?;
            match token.token_type {
                Type::Eof => {
                    tokens.push(token);
                    break;
                }
                _ => {
                    tokens.push(token);
                }
            }
        }
        Ok(Self { tokens, pos: 0 })
    }

    fn next(&mut self) -> Token {
        let tok = self.tokens[self.pos as usize].clone();
        self.pos += 1;
        tok
    }

    fn back(&mut self) {
        self.pos -= 1;
    }

    fn parse(&mut self) -> Result<Operation, ErrorPosition> {
        let mut op = Operation {
            components: vec![],
            ops: vec![],
        };

        op.components.push(self.component()?);
        let mut token = self.next();

        while let Type::Or | Type::Xor | Type::And = token.token_type {
            op.ops.push(token.clone());
            op.components.push(self.component()?);
            token = self.next();
        }
        self.back();

        Ok(op)
    }

    fn component(&mut self) -> Result<Component, ErrorPosition> {
        let mut token = self.next();
        let mut neg = false;
        let val: VarOrExpr;

        loop {
            match token.token_type {
                Type::Not => neg = !neg,
                Type::LParen => {
                    val = VarOrExpr::Expr(self.parse()?);
                    let next = self.next();
                    match next.token_type {
                        Type::RParen => {}
                        ref other => {
                            return Err(ErrorPosition::from_token(
                                format!("Unexpected token: {:#?}", other),
                                &next,
                            ))
                        }
                    };
                    break;
                }
                Type::Ident(name) => {
                    val = VarOrExpr::Var(name);
                    break;
                }
                ref other => {
                    return Err(ErrorPosition::from_token(
                        format!("Unexpected token: {:#?}", other),
                        &token,
                    ))
                }
            }
            token = self.next();
        }

        Ok(Component {
            value: val,
            negated: neg,
        })
    }
}

struct Environment {
    vars: HashMap<String, bool>,
}

impl Environment {
    fn get_variable(&self, name: &str) -> bool {
        match self.vars.get(name) {
            Some(var) => *var,
            None => false,
        }
    }
}

fn main() {
    println!("Enter an expression:");
    for line in std::io::stdin().lines().flatten() {
        let string = line.trim().to_string();
        let eval = parse_expr(string);
        if let Err(err) = eval {
            let rng = err.col_range;
            print!("{}", '~'.repeat(rng.0 - 1));
            println!("{}", '^'.repeat(rng.1 - rng.0 + 1));
            println!(
                "Error: \"{}\" at column {}, line {}",
                err.msg, rng.0, err.line
            );
        }
    }
}

fn parse_expr(src: String) -> Result<(), ErrorPosition> {
    let mut lexer = Lexer {
        reader: StringReader::new(src),
    };

    let mut parser = Parser::new(&mut lexer)?;
    let root = parser.parse()?;

    let table = root.truth_table()?;
    let mut vars = root.get_variables();

    vars.sort();

    println!("> Truth table:");
    for var in &vars {
        print!("{}    ", var);
    }
    print!("Result\n\n");

    for &(ref vars, ref res) in &table {
        let mut sorted = Vec::with_capacity(table.len());
        for pair in vars.iter() {
            sorted.push(pair);
        }
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        for &(name, val) in &sorted {
            print!("{}{}    ", *val as u8, ' '.repeat(name.len() as u32));
        }
        println!("{}", *res as u8);
    }

    println!("> Parsed tree:\n{:#?}", root);
    println!("> Variables: {:#?}", vars);
    Ok(())
}
