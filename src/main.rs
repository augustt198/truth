fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

#[allow(dead_code)]
struct StringReader {
    pos:    uint,
    source: String,
    col:    uint,
    line:   uint
}

impl StringReader {
    fn new(source: String) -> StringReader {
        StringReader {
            pos: 0,
            line: 1,
            col: 0,
            source: source
        }
    }

    fn peak(&mut self) -> Option<char> {
        if self.pos < self.source.len() {
            Some(self.source.as_slice().char_at(self.pos))
        } else {
            None
        }
    }

    fn read(&mut self) -> Option<char> {
        let next = self.peak();
        if next.is_some()  {
            if next.unwrap() == '\n' {
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

struct Lexer {
    reader: StringReader
}

impl Lexer {
    fn tok(&self, token_type: Type) -> Token {
        Token { token_type: token_type, col: self.reader.col, line: self.reader.line }
    }

    fn next_token(&mut self) -> Token {
        loop {
            let c = match self.reader.read() {
                Some(c) => c,
                None => return self.tok(EOF)
            };

            if      c == '(' { return self.tok(LParen) }
            else if c == ')' { return self.tok(RParen) }
            else if c == '&' { return self.tok(And) }
            else if c == '|' { return self.tok(Or) }
            else if c == '!' { return self.tok(Not) }
            else if c == '^' { return self.tok(Xor) }

            else if is_alpha(c) { return self.next_ident(c)}

            else if c == ' ' || c == '\n' { continue }
            else { fail!("Unexpected character: {}", c) }
        }
    }

    fn next_ident(&mut self, current: char) -> Token {
        let mut string = String::new();
        string.push(current);

        loop {
            let peak = self.reader.peak();
            if peak.is_some() && is_alpha(peak.unwrap()) {
                string.push(peak.unwrap());
                self.reader.read();
            } else {
                break
            }
        }
        
        self.tok(Ident(string))
    }
}

#[allow(dead_code)]
#[deriving(Show)]
#[deriving(Clone)]
enum Type {
    LParen,
    RParen,

    Ident(String),

    And,
    Or,
    Not,
    Xor,

    EOF
}

#[allow(dead_code)]
#[deriving(Clone)]
#[deriving(Show)]
struct Token {
    token_type: Type,
    col:        uint,
    line:       uint
}

#[deriving(Show)]
struct Operation {
    components: Vec<Component>,
    ops: Vec<Token>
}

#[deriving(Show)]
enum VarOrExpr {
    Var(String),
    Expr(Operation)
}

#[deriving(Show)]
struct Component {
    value: VarOrExpr,
    negated: bool
}

struct Parser {
    tokens: Vec<Token>,
    pos: uint
}

impl Parser {
    fn new(lexer: &mut Lexer) -> Parser {
        let mut tokens = vec!();
        let mut token;
        loop {
            token = lexer.next_token();
            match token.token_type {
                EOF => {
                    tokens.push(token);
                    break
                },
                _   => {
                    tokens.push(token);
                }
            }
        }
        Parser { tokens: tokens, pos: 0 }
    }
    
    fn next(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }
    
    fn back(&mut self) { self.pos -= 1; }
    
    fn parse(&mut self) -> Operation {
        let mut op = Operation { components: vec!(), ops: vec!() };
        
        let component = op.components.push(self.component());
        let mut token = self.next();
        println!("(P1) token is {}", token);

        loop {
            match token.token_type {
                Or | Xor | And => {
                    op.ops.push(token.clone());
                    op.components.push(self.component());
                },
                _ => {
                    break;
                }
            };
            token = self.next();
            println!("(PL) token is {}", token);
        }
        self.back();
        
        op
    }
    
    fn component(&mut self) -> Component {
        let mut token = self.next();
        let mut neg = false;
        let mut val: VarOrExpr;
        
        println!("(C1) token is {}", token);

        loop {
            match token.token_type {
                Not => neg = !neg,
                LParen => {
                    val = Expr(self.parse());
                    let check = self.next();
                    match check.token_type {
                        RParen => {},
                        _ => { fail!("Unexpected token: {}", check); }
                    };
                    break;
                },
                Ident(name) => {
                    val = Var(name);
                    break;
                },
                _ => { fail!("Unexpected token"); }
            }
            token = self.next();
            println!("(CL) token is {}", token);
        }

        Component { value: val, negated: neg }
    }
    
}

fn main() {
    for line in std::io::stdin().lines() {
        if line.is_ok() {
            let mut lexer = Lexer { reader: StringReader::new(line.unwrap()) };
            let mut parser = Parser::new(&mut lexer);
            println!("Parsed:\n{}", parser.parse());
        }
    }
}
