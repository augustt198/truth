use std::io::File;

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

#[allow(dead_code)]
struct StringReader<'a> {
    pos:    uint,
    source: &'a str,
    col:    uint,
    line:   uint
}

impl<'a> StringReader<'a> {
    fn new(source: &'a str) -> StringReader<'a> {
        StringReader {
            pos: 0,
            line: 1,
            col: 0,
            source: source
        }
    }

    fn peak(&mut self) -> Option<char> {
        if self.pos + 1 < self.source.len() {
            Some(self.source.char_at(self.pos + 1))
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
    reader: StringReader<'static>
}

impl Lexer {
    fn tok(&self, token_type: Type) -> Token {
        Token { token_type: token_type, col: 0u, line: 0u }
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

            else if c == ' ' { continue }
            else { fail!("Unexpected character: {}", c) }
        }
        fail!() // hm, this code should never execute
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
struct Token {
    token_type: Type,
    col:        uint,
    line:       uint
}

fn main() {

}
