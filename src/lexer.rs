// ============================================================
//  HSD — Hic Sunt Dracones
//  Phase 1: the LEXER
//  Turns source text into a list of tokens.
// ============================================================

// ---------- Tokens ----------
// A Token is "one of" many possible things: a keyword, a
// literal, an operator, and so on. Variants with parentheses
// carry a payload (the number's value, the string text, ...).

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // --- Keywords (HSD keywords stay Latin: they ARE the language) ---
    Sit, Fixum, Munus, Redde,
    Si, Aliter, Dum, Per, In,
    Frange, Perge,
    Verum, Falsum, Nihil,
    Et, Vel, Non,
    Genus, Affer,
    Actor, Accipe, Nuntius, Mitte, Ad, Crea, Ipse,
    Nativum,

    // --- Literals (carry a value) ---
    IntLit(i64),       // 42      -> IntLit(42)
    FloatLit(f64),     // 3.14    -> FloatLit(3.14)
    StrLit(String),    // "ciao"  -> StrLit("ciao")
    Ident(String),     // myVar   -> Ident("myVar")

    // --- Operators ---
    Plus, Minus, Star, Slash, Percent,      //  + - * / %
    Eq, EqEq, Neq,                          //  =  ==  !=
    Lt, Gt, Le, Ge,                         //  <  >  <=  >=
    Arrow,                                  //  ->

    // --- Punctuation ---
    LParen, RParen, LBrack, RBrack,         //  ( ) [ ]
    Comma, Colon, Dot,                      //  , : .

    // --- Layout ---
    Newline, Indent, Dedent, Eof,
}

// Each token also knows WHERE it sits in the source: this is
// what makes clear error messages possible (a goal from the
// project sheet).
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSpan {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

// ---------- Keyword table ----------
// Given the text of an identifier, tells whether it is a
// keyword. Otherwise returns None (and it will be an Ident).

fn keyword(s: &str) -> Option<Token> {
    Some(match s {
        "sit"     => Token::Sit,
        "fixum"   => Token::Fixum,
        "munus"   => Token::Munus,
        "redde"   => Token::Redde,
        "si"      => Token::Si,
        "aliter"  => Token::Aliter,
        "dum"     => Token::Dum,
        "per"     => Token::Per,
        "in"      => Token::In,
        "frange"  => Token::Frange,
        "perge"   => Token::Perge,
        "verum"   => Token::Verum,
        "falsum"  => Token::Falsum,
        "nihil"   => Token::Nihil,
        "et"      => Token::Et,
        "vel"     => Token::Vel,
        "non"     => Token::Non,
        "genus"   => Token::Genus,
        "affer"   => Token::Affer,
        "actor"   => Token::Actor,
        "accipe"  => Token::Accipe,
        "nuntius" => Token::Nuntius,
        "mitte"   => Token::Mitte,
        "ad"      => Token::Ad,
        "crea"    => Token::Crea,
        "ipse"    => Token::Ipse,
        "nativum" => Token::Nativum,
        _ => return None,   // not a keyword
    })
}

// ---------- The Lexer ----------

pub struct Lexer {
    src: Vec<char>,       // the source, character by character
    pos: usize,           // how far we have read
    line: usize,
    column: usize,
    indents: Vec<usize>,  // stack of indentation levels
    out: Vec<TokenSpan>,  // tokens produced so far
}

impl Lexer {
    pub fn new(source: &str) -> Lexer {
        Lexer {
            src: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            indents: vec![0],   // start at level 0
            out: Vec::new(),
        }
    }

    // --- small helpers to look and advance ---

    fn peek(&self) -> Option<char> {
        self.src.get(self.pos).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.src.get(self.pos + n).copied()
    }

    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }

    fn emit(&mut self, token: Token) {
        self.out.push(TokenSpan {
            token,
            line: self.line,
            column: self.column,
        });
    }

    fn skip_comment(&mut self) {
        // a '#' comment runs to end of line (the newline stays)
        while let Some(c) = self.peek() {
            if c == '\n' { break; }
            self.advance();
        }
    }

    // --- the main function ---

    pub fn tokenize(&mut self) -> Result<Vec<TokenSpan>, String> {
        loop {
            // at the start of a line: handle indentation first
            self.handle_indentation()?;
            if self.at_end() { break; }
            // then tokenize the rest of the line
            self.tokenize_line()?;
        }
        // ensure a trailing Newline before closing
        if self.out.last().map_or(false, |t| t.token != Token::Newline) {
            self.emit(Token::Newline);
        }
        // close every block still open
        while self.indents.len() > 1 {
            self.indents.pop();
            self.emit(Token::Dedent);
        }
        self.emit(Token::Eof);
        Ok(self.out.clone())
    }

    // --- indentation handling (the delicate part) ---
    // Count the leading spaces of a line and compare with the
    // stack:
    //   more indented -> push, emit Indent
    //   less indented -> repeated pop, emit Dedent
    //   equal         -> nothing
    // Blank or comment-only lines are skipped.

    fn handle_indentation(&mut self) -> Result<(), String> {
        loop {
            // count the leading spaces
            let mut count = 0;
            while self.peek() == Some(' ') {
                self.advance();
                count += 1;
            }
            // tabs are converted to 4 spaces each
            while self.peek() == Some('\t') {
                self.advance();
                count += 4;
            }
            // blank or comment-only line: skip it, do not count
            match self.peek() {
                None => return Ok(()),                       // end of file
                Some('\n') => { self.advance(); continue; }  // blank line
                Some('#') => { self.skip_comment(); continue; }
                _ => {}
            }
            // a "real" line: compare with the indentation stack
            let current = *self.indents.last().unwrap();
            if count > current {
                self.indents.push(count);
                self.emit(Token::Indent);
            } else if count < current {
                while count < *self.indents.last().unwrap() {
                    self.indents.pop();
                    self.emit(Token::Dedent);
                }
                if count != *self.indents.last().unwrap() {
                    return Err(format!(
                        "Line {}: inconsistent indentation",
                        self.line
                    ));
                }
            }
            return Ok(());
        }
    }

    // --- tokenize one line up to the newline ---

    fn tokenize_line(&mut self) -> Result<(), String> {
        loop {
            // skip spaces inside the line
            while self.peek() == Some(' ') {
                self.advance();
            }
            match self.peek() {
                None => return Ok(()),               // end of file
                Some('\n') => {
                    self.emit(Token::Newline);
                    self.advance();
                    return Ok(());
                }
                Some('#') => self.skip_comment(),    // trailing comment
                Some(c) => self.tokenize_one(c)?,
            }
        }
    }

    // --- tokenize a single element ---

    fn tokenize_one(&mut self, c: char) -> Result<(), String> {
        if c.is_ascii_digit() {
            return self.tokenize_number();
        }
        if c.is_alphabetic() || c == '_' {
            return self.tokenize_ident();
        }
        if c == '"' {
            return self.tokenize_string();
        }

        // operators and punctuation
        let line = self.line;
        let col = self.column;
        self.advance(); // consume c
        let tok = match c {
            '+' => Token::Plus,
            '-' => {
                if self.peek() == Some('>') { self.advance(); Token::Arrow }
                else { Token::Minus }
            }
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') { self.advance(); Token::EqEq }
                else { Token::Eq }
            }
            '!' => {
                if self.peek() == Some('=') { self.advance(); Token::Neq }
                else {
                    return Err(format!("Line {}: unexpected character '!'", line));
                }
            }
            '<' => {
                if self.peek() == Some('=') { self.advance(); Token::Le }
                else { Token::Lt }
            }
            '>' => {
                if self.peek() == Some('=') { self.advance(); Token::Ge }
                else { Token::Gt }
            }
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBrack,
            ']' => Token::RBrack,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '.' => Token::Dot,
            other => {
                return Err(format!("Line {}: unexpected character '{}'", line, other));
            }
        };
        self.out.push(TokenSpan { token: tok, line, column: col });
        Ok(())
    }

    // --- numbers: integers (IntLit) and decimals (FloatLit) ---

    fn tokenize_number(&mut self) -> Result<(), String> {
        let line = self.line;
        let col = self.column;
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() { text.push(c); self.advance(); }
            else { break; }
        }
        // decimal part: a '.' followed by at least one digit
        // (otherwise the '.' is a Dot, e.g. field access)
        if self.peek() == Some('.')
            && self.peek_n(1).map_or(false, |c| c.is_ascii_digit())
        {
            text.push('.');
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() { text.push(c); self.advance(); }
                else { break; }
            }
            let value: f64 = text.parse().map_err(|_| {
                format!("Line {}: invalid number '{}'", line, text)
            })?;
            self.out.push(TokenSpan { token: Token::FloatLit(value), line, column: col });
        } else {
            let value: i64 = text.parse().map_err(|_| {
                format!("Line {}: invalid number '{}'", line, text)
            })?;
            self.out.push(TokenSpan { token: Token::IntLit(value), line, column: col });
        }
        Ok(())
    }

    // --- identifiers and keywords ---

    fn tokenize_ident(&mut self) -> Result<(), String> {
        let line = self.line;
        let col = self.column;
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' { text.push(c); self.advance(); }
            else { break; }
        }
        // a keyword if it matches the table, otherwise an Ident
        let token = keyword(&text)
            .unwrap_or_else(|| Token::Ident(text.clone()));
        self.out.push(TokenSpan { token, line, column: col });
        Ok(())
    }

    // --- strings ---

    fn tokenize_string(&mut self) -> Result<(), String> {
        let line = self.line;
        let col = self.column;
        self.advance(); // consume the opening "
        let mut text = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(format!("Line {}: unterminated string", line));
                }
                Some('"') => { self.advance(); break; }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n')  => { text.push('\n'); self.advance(); }
                        Some('t')  => { text.push('\t'); self.advance(); }
                        Some('"')  => { text.push('"');  self.advance(); }
                        Some('\\') => { text.push('\\'); self.advance(); }
                        Some(other) => {
                            return Err(format!(
                                "Line {}: invalid escape sequence '\\{}'", line, other
                            ));
                        }
                        None => {
                            return Err(format!("Line {}: unterminated string", line));
                        }
                    }
                }
                Some(c) => { text.push(c); self.advance(); }
            }
        }
        self.out.push(TokenSpan { token: Token::StrLit(text), line, column: col });
        Ok(())
    }
}
