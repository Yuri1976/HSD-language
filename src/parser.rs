// ============================================================
//  HSD — Hic Sunt Dracones
//  the PARSER (expressions)
//
//  Turns a list of tokens into an AST. This file handles
//  EXPRESSIONS, using a "Pratt parser" to get operator
//  precedence right. Statements and top-level items come next.
// ============================================================

#![allow(dead_code)]

use crate::ast::*;
use crate::lexer::{Token, TokenSpan};

// ---------- The Parser ----------

pub struct Parser {
    tokens: Vec<TokenSpan>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenSpan>) -> Parser {
        Parser { tokens, pos: 0 }
    }

    // --- small helpers ---

    // The current token (without consuming it).
    fn peek(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    // The current token with its position info.
    fn peek_span(&self) -> &TokenSpan {
        &self.tokens[self.pos]
    }

    // Consume the current token and move forward.
    // We never advance past the final Eof token.
    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].token.clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    // Is the current token equal to `t`?
    fn check(&self, t: &Token) -> bool {
        self.peek() == t
    }

    // Consume `t` if it is there, otherwise produce an error.
    fn expect(&mut self, t: &Token, what: &str) -> Result<(), String> {
        if self.check(t) {
            self.advance();
            Ok(())
        } else {
            let span = self.peek_span();
            Err(format!(
                "Line {}: expected {}, found {:?}",
                span.line, what, span.token
            ))
        }
    }

    // Consume an identifier and return its name.
    fn parse_ident(&mut self, what: &str) -> Result<String, String> {
        match self.peek().clone() {
            Token::Ident(name) => {
                self.advance();
                Ok(name)
            }
            other => {
                let span = self.peek_span();
                Err(format!(
                    "Line {}: expected {}, found {:?}",
                    span.line, what, other
                ))
            }
        }
    }

    // ========================================================
    //  EXPRESSIONS — the Pratt parser
    // ========================================================
    //
    //  The idea: every infix operator has a "binding power"
    //  (a precedence number). `parse_expr_bp(min_bp)` parses
    //  an expression made only of operators whose binding
    //  power is at least `min_bp`. Recursion with a higher
    //  min_bp is what makes `*` bind tighter than `+`.

    // Public entry point: parse a full expression.
    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, String> {
        // First, the left-hand side (a prefix expression).
        let mut left = self.parse_prefix()?;

        // Then, as long as the next token is an infix operator
        // strong enough (binding power >= min_bp), absorb it.
        loop {
            let (lbp, op) = match self.peek() {
                Token::Vel     => (10, BinOp::Or),
                Token::Et      => (20, BinOp::And),
                Token::EqEq    => (30, BinOp::Eq),
                Token::Neq     => (30, BinOp::Neq),
                Token::Lt      => (30, BinOp::Lt),
                Token::Gt      => (30, BinOp::Gt),
                Token::Le      => (30, BinOp::Le),
                Token::Ge      => (30, BinOp::Ge),
                Token::Plus    => (40, BinOp::Add),
                Token::Minus   => (40, BinOp::Sub),
                Token::Star    => (50, BinOp::Mul),
                Token::Slash   => (50, BinOp::Div),
                Token::Percent => (50, BinOp::Mod),
                _ => break, // not an operator: the expression ends here
            };
            if lbp < min_bp {
                break; // operator too weak for this level: stop
            }
            self.advance(); // consume the operator
            // Left-associative: the right side is parsed with a
            // slightly higher minimum (lbp + 1).
            let right = self.parse_expr_bp(lbp + 1)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // A prefix expression: an optional unary operator, then a
    // primary, then any postfix operators (call, index, field).
    fn parse_prefix(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_prefix()?;
                Ok(Expr::Unary { op: UnOp::Neg, operand: Box::new(operand) })
            }
            Token::Non => {
                self.advance();
                let operand = self.parse_prefix()?;
                Ok(Expr::Unary { op: UnOp::Not, operand: Box::new(operand) })
            }
            _ => {
                let primary = self.parse_primary()?;
                self.parse_postfix(primary)
            }
        }
    }

    // Postfix operators chain onto an expression:
    //   f(x)      call
    //   list[0]   index
    //   point.x   field access
    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    let args = self.parse_args(&Token::RParen)?;
                    self.expect(&Token::RParen, "')'")?;
                    expr = Expr::Call { callee: Box::new(expr), args };
                }
                Token::LBrack => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&Token::RBrack, "']'")?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Dot => {
                    self.advance();
                    let name = self.parse_ident("a field name after '.'")?;
                    expr = Expr::Field { object: Box::new(expr), name };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    // A primary expression: the atoms of the language.
    fn parse_primary(&mut self) -> Result<Expr, String> {
        let span = self.peek_span().clone();
        match self.peek().clone() {
            Token::IntLit(n)   => { self.advance(); Ok(Expr::IntLit(n)) }
            Token::FloatLit(f) => { self.advance(); Ok(Expr::FloatLit(f)) }
            Token::StrLit(s)   => { self.advance(); Ok(Expr::StrLit(s)) }
            Token::Verum       => { self.advance(); Ok(Expr::BoolLit(true)) }
            Token::Falsum      => { self.advance(); Ok(Expr::BoolLit(false)) }
            Token::Nihil       => { self.advance(); Ok(Expr::Nihil) }
            Token::Ipse        => { self.advance(); Ok(Expr::Ipse) }
            Token::Ident(name) => { self.advance(); Ok(Expr::Ident(name)) }

            // crea Name(args...) : spawn an actor
            Token::Crea => {
                self.advance();
                let name = self.parse_ident("an actor name after 'crea'")?;
                let mut args = Vec::new();
                if self.check(&Token::LParen) {
                    self.advance();
                    args = self.parse_args(&Token::RParen)?;
                    self.expect(&Token::RParen, "')'")?;
                }
                Ok(Expr::Create { name, args })
            }

            // [a, b, c] : a list literal
            Token::LBrack => {
                self.advance();
                let items = self.parse_args(&Token::RBrack)?;
                self.expect(&Token::RBrack, "']'")?;
                Ok(Expr::List(items))
            }

            // ( expr ) : parentheses for grouping
            Token::LParen => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(&Token::RParen, "')'")?;
                Ok(inner)
            }

            other => Err(format!(
                "Line {}: expected an expression, found {:?}",
                span.line, other
            )),
        }
    }

    // A comma-separated list of expressions, up to (but not
    // including) the `close` token. Handles the empty case.
    fn parse_args(&mut self, close: &Token) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.check(close) {
            return Ok(args); // empty: e.g. f()
        }
        loop {
            args.push(self.parse_expr()?);
            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    // ========================================================
    //  PROGRAM and ITEMS — recursive descent
    // ========================================================

    // A whole program: a sequence of top-level items.
    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        while !self.check(&Token::Eof) {
            // tolerate stray blank newlines between items
            if self.check(&Token::Newline) {
                self.advance();
                continue;
            }
            items.push(self.parse_item()?);
        }
        Ok(Program { items })
    }

    // Dispatch on the first token to the right item parser.
    fn parse_item(&mut self) -> Result<Item, String> {
        match self.peek() {
            Token::Affer   => self.parse_import(),
            Token::Munus   => Ok(Item::Function(self.parse_function()?)),
            Token::Genus   => Ok(Item::Genus(self.parse_genus()?)),
            Token::Actor   => Ok(Item::Actor(self.parse_actor()?)),
            Token::Nuntius => Ok(Item::Nuntius(self.parse_nuntius()?)),
            _ => Ok(Item::Statement(self.parse_stmt()?)),
        }
    }

    fn parse_import(&mut self) -> Result<Item, String> {
        self.expect(&Token::Affer, "'affer'")?;
        let module = match self.peek().clone() {
            Token::StrLit(s) => {
                self.advance();
                s
            }
            other => {
                return Err(format!(
                    "Line {}: expected a module name string, found {:?}",
                    self.peek_span().line, other
                ));
            }
        };
        self.expect(&Token::Newline, "a newline after the import")?;
        Ok(Item::Import(module))
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(&Token::Munus, "'munus'")?;
        let name = self.parse_ident("a function name")?;
        self.expect(&Token::LParen, "'(' after the function name")?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen, "')' after the parameters")?;
        // the return type is optional
        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Function { name, params, return_type, body })
    }

    fn parse_genus(&mut self) -> Result<GenusDef, String> {
        self.expect(&Token::Genus, "'genus'")?;
        let name = self.parse_ident("a genus name")?;
        self.expect(&Token::Newline, "a newline")?;
        self.expect(&Token::Indent, "an indented list of fields")?;
        let mut fields = Vec::new();
        while !self.check(&Token::Dedent) && !self.check(&Token::Eof) {
            fields.push(self.parse_field()?);
        }
        self.expect(&Token::Dedent, "the end of the genus")?;
        Ok(GenusDef { name, fields })
    }

    // A genus field: `name: type` on its own line.
    fn parse_field(&mut self) -> Result<Field, String> {
        let name = self.parse_ident("a field name")?;
        self.expect(&Token::Colon, "':' after the field name")?;
        let ty = self.parse_type()?;
        self.expect(&Token::Newline, "a newline after the field")?;
        Ok(Field { name, ty })
    }

    fn parse_actor(&mut self) -> Result<ActorDef, String> {
        self.expect(&Token::Actor, "'actor'")?;
        let name = self.parse_ident("an actor name")?;
        self.expect(&Token::Newline, "a newline")?;
        self.expect(&Token::Indent, "an indented actor body")?;
        let mut state = Vec::new();
        let mut handlers = Vec::new();
        while !self.check(&Token::Dedent) && !self.check(&Token::Eof) {
            match self.peek() {
                Token::Accipe => handlers.push(self.parse_handler()?),
                Token::Sit | Token::Fixum => state.push(self.parse_let()?),
                _ => {
                    let span = self.peek_span();
                    return Err(format!(
                        "Line {}: an actor body allows only state ('sit'/'fixum') \
                         and handlers ('accipe'), found {:?}",
                        span.line, span.token
                    ));
                }
            }
        }
        self.expect(&Token::Dedent, "the end of the actor")?;
        Ok(ActorDef { name, state, handlers })
    }

    // A message handler: `accipe Message(params) <block>`
    fn parse_handler(&mut self) -> Result<Handler, String> {
        self.expect(&Token::Accipe, "'accipe'")?;
        let message = self.parse_ident("a message name")?;
        let mut params = Vec::new();
        if self.check(&Token::LParen) {
            self.advance();
            params = self.parse_params()?;
            self.expect(&Token::RParen, "')'")?;
        }
        let body = self.parse_block()?;
        Ok(Handler { message, params, body })
    }

    fn parse_nuntius(&mut self) -> Result<NuntiusDef, String> {
        self.expect(&Token::Nuntius, "'nuntius'")?;
        let name = self.parse_ident("a message name")?;
        let mut params = Vec::new();
        if self.check(&Token::LParen) {
            self.advance();
            params = self.parse_params()?;
            self.expect(&Token::RParen, "')'")?;
        }
        self.expect(&Token::Newline, "a newline")?;
        Ok(NuntiusDef { name, params })
    }

    // Comma-separated parameters: `name: type, name: type, ...`
    fn parse_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) {
            return Ok(params); // empty parameter list
        }
        loop {
            let name = self.parse_ident("a parameter name")?;
            self.expect(&Token::Colon, "':' after the parameter name")?;
            let ty = self.parse_type()?;
            params.push(Param { name, ty });
            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    // A type: `name` or `name[innertype]`
    fn parse_type(&mut self) -> Result<Type, String> {
        // 'nihil' is a keyword, but it is also a valid type
        // (the "void" type used as a return type).
        let name = if self.check(&Token::Nihil) {
            self.advance();
            "nihil".to_string()
        } else {
            self.parse_ident("a type name")?
        };
        if self.check(&Token::LBrack) {
            self.advance();
            let inner = self.parse_type()?;
            self.expect(&Token::RBrack, "']'")?;
            Ok(Type::Generic(name, Box::new(inner)))
        } else {
            Ok(Type::Named(name))
        }
    }

    // ========================================================
    //  STATEMENTS — recursive descent
    // ========================================================

    // An indented block: NEWLINE INDENT statements... DEDENT
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(&Token::Newline, "a newline before the block")?;
        self.expect(&Token::Indent, "an indented block")?;
        let mut stmts = Vec::new();
        while !self.check(&Token::Dedent) && !self.check(&Token::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::Dedent, "the end of the block")?;
        Ok(stmts)
    }

    // Dispatch on the first token to the right statement parser.
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Token::Sit | Token::Fixum => self.parse_let(),
            Token::Si      => self.parse_if(),
            Token::Dum     => self.parse_while(),
            Token::Per     => self.parse_for(),
            Token::Redde   => self.parse_return(),
            Token::Mitte   => self.parse_send(),
            Token::Nativum => self.parse_nativum(),
            Token::Frange => {
                self.advance();
                self.expect(&Token::Newline, "a newline after 'frange'")?;
                Ok(Stmt::Break)
            }
            Token::Perge => {
                self.advance();
                self.expect(&Token::Newline, "a newline after 'perge'")?;
                Ok(Stmt::Continue)
            }
            _ => self.parse_expr_or_assign(),
        }
    }

    // sit / fixum : a name binding.
    fn parse_let(&mut self) -> Result<Stmt, String> {
        let mutable = self.check(&Token::Sit); // sit = mutable, fixum = not
        self.advance(); // consume sit / fixum
        let name = self.parse_ident("a variable name")?;
        // the type annotation is optional (it can be inferred)
        let ty = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&Token::Eq, "'=' in the declaration")?;
        let value = self.parse_expr()?;
        self.expect(&Token::Newline, "a newline after the declaration")?;
        Ok(Stmt::Let { mutable, name, ty, value })
    }

    // si / aliter si / aliter
    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(&Token::Si, "'si'")?;
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let mut elif = Vec::new();
        let mut else_block = None;
        while self.check(&Token::Aliter) {
            self.advance(); // consume 'aliter'
            if self.check(&Token::Si) {
                self.advance(); // 'aliter si' : another condition
                let c = self.parse_expr()?;
                let b = self.parse_block()?;
                elif.push((c, b));
            } else {
                // a bare 'aliter' : the final else, ends the chain
                else_block = Some(self.parse_block()?);
                break;
            }
        }
        Ok(Stmt::If { cond, then_block, elif, else_block })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(&Token::Dum, "'dum'")?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(&Token::Per, "'per'")?;
        let var = self.parse_ident("a loop variable name")?;
        self.expect(&Token::In, "'in' after the loop variable")?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For { var, iter, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.expect(&Token::Redde, "'redde'")?;
        // the expression is optional: `redde` alone is allowed
        let value = if self.check(&Token::Newline) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(&Token::Newline, "a newline after 'redde'")?;
        Ok(Stmt::Return(value))
    }

    // mitte <message> ad <target>
    fn parse_send(&mut self) -> Result<Stmt, String> {
        self.expect(&Token::Mitte, "'mitte'")?;
        let message = self.parse_expr()?;
        self.expect(&Token::Ad, "'ad' between the message and the target")?;
        let target = self.parse_expr()?;
        self.expect(&Token::Newline, "a newline after the send")?;
        Ok(Stmt::Send { message, target })
    }

    fn parse_nativum(&mut self) -> Result<Stmt, String> {
        self.expect(&Token::Nativum, "'nativum'")?;
        let body = self.parse_block()?;
        Ok(Stmt::Nativum(body))
    }

    // An expression statement, or an assignment if '=' follows.
    // We parse an expression first, then look at the next token
    // to decide which of the two it is.
    fn parse_expr_or_assign(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;
        if self.check(&Token::Eq) {
            self.advance(); // consume '='
            let value = self.parse_expr()?;
            self.expect(&Token::Newline, "a newline after the assignment")?;
            Ok(Stmt::Assign { target: expr, value })
        } else {
            self.expect(&Token::Newline, "a newline after the statement")?;
            Ok(Stmt::Expr(expr))
        }
    }
}
