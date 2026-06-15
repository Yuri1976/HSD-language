// ============================================================
//  HSD — Hic Sunt Dracones
//  Phase 3: SEMANTIC ANALYSIS
//    3a — name resolution  (every name used must be declared)
//    3b — type checking    (every operation must be type-valid)
//
//  All errors are collected, then reported together.
// ============================================================

#![allow(dead_code)]

use std::collections::HashMap;
use crate::ast::*;

// ---------- Semantic types ----------
// `Ty` is the type of an expression as the analyzer sees it.
// (The AST `Type` is the *syntactic* type the user wrote;
//  `Ty` is the *resolved* type used for checking.)

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Num,             // numerus
    Real,            // realis
    Bool,            // veritas
    Str,             // verba
    Nihil,           // nihil (no value / void)
    List(Box<Ty>),   // series[T]
    Genus(String),   // a user-defined genus
    Actor(String),   // an actor type
    Unknown,         // error recovery: matches anything
}

impl Ty {
    // A human-readable name, for error messages.
    fn name(&self) -> String {
        match self {
            Ty::Num => "numerus".to_string(),
            Ty::Real => "realis".to_string(),
            Ty::Bool => "veritas".to_string(),
            Ty::Str => "verba".to_string(),
            Ty::Nihil => "nihil".to_string(),
            Ty::List(t) => format!("series[{}]", t.name()),
            Ty::Genus(n) => n.clone(),
            Ty::Actor(n) => n.clone(),
            Ty::Unknown => "?".to_string(),
        }
    }
}

// Two types are compatible if they are equal, or if either is
// Unknown. The Unknown case stops one error from cascading
// into many.
fn compatible(a: &Ty, b: &Ty) -> bool {
    *a == Ty::Unknown || *b == Ty::Unknown || a == b
}

fn is_number(t: &Ty) -> bool {
    matches!(t, Ty::Num | Ty::Real | Ty::Unknown)
}

// ---------- Symbols ----------

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable { mutable: bool, ty: Ty },
    Function { params: Vec<Ty>, ret: Ty },
    Parameter { ty: Ty },
    Genus { fields: Vec<(String, Ty)> },
    Actor,
    Nuntius { params: Vec<Ty> },
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
}

// ---------- The Symbol Table (stack of scopes) ----------

pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolInfo>>,
}

impl SymbolTable {
    fn new() -> SymbolTable {
        SymbolTable { scopes: vec![HashMap::new()] }
    }
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    // Declare in the current scope; false if the name exists there.
    fn declare(&mut self, name: &str, info: SymbolInfo) -> bool {
        let current = self.scopes.last_mut().unwrap();
        if current.contains_key(name) {
            false
        } else {
            current.insert(name.to_string(), info);
            true
        }
    }
    // Look up a name from the innermost scope outward.
    fn lookup(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }
}

// ---------- The Analyzer ----------

pub struct Analyzer {
    table: SymbolTable,
    errors: Vec<String>,
    current_return: Ty,            // return type of the function in progress
    current_actor: Option<String>, // the actor in progress (for 'ipse')
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            table: SymbolTable::new(),
            errors: Vec::new(),
            current_return: Ty::Nihil,
            current_actor: None,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<String>> {
        // built-in functions
        self.table.declare("scribe", SymbolInfo {
            kind: SymbolKind::Function { params: vec![], ret: Ty::Nihil },
        }); // 'scribe' is variadic: handled specially at call sites
        self.table.declare("numera", SymbolInfo {
            kind: SymbolKind::Function {
                params: vec![Ty::Num, Ty::Num],
                ret: Ty::List(Box::new(Ty::Num)),
            },
        });
        self.table.declare("lege", SymbolInfo {
            kind: SymbolKind::Function { params: vec![], ret: Ty::Str },
        }); // 'lege' accepts an optional prompt: handled specially at call sites
        self.table.declare("numerus_ex", SymbolInfo {
            kind: SymbolKind::Function { params: vec![Ty::Str], ret: Ty::Num },
        });
        self.table.declare("realis_ex", SymbolInfo {
            kind: SymbolKind::Function { params: vec![Ty::Str], ret: Ty::Real },
        });

        // Pass 1a: declare type names (genus, actor, nuntius)
        for item in &program.items {
            match item {
                Item::Genus(g) => {
                    let mut fields = Vec::new();
                    for f in &g.fields {
                        let fty = self.resolve_type(&f.ty);
                        fields.push((f.name.clone(), fty));
                    }
                    self.declare_named(&g.name, SymbolKind::Genus { fields });
                }
                Item::Actor(a) => self.declare_named(&a.name, SymbolKind::Actor),
                Item::Nuntius(n) => {
                    let mut params = Vec::new();
                    for p in &n.params {
                        params.push(self.resolve_type(&p.ty));
                    }
                    self.declare_named(&n.name, SymbolKind::Nuntius { params });
                }
                _ => {}
            }
        }

        // Pass 1b: declare function signatures
        for item in &program.items {
            if let Item::Function(f) = item {
                let mut params = Vec::new();
                for p in &f.params {
                    params.push(self.resolve_type(&p.ty));
                }
                let ret = match &f.return_type {
                    Some(t) => self.resolve_type(t),
                    None => Ty::Nihil,
                };
                self.declare_named(&f.name, SymbolKind::Function { params, ret });
            }
        }

        // Pass 1c: declare global constants
        for item in &program.items {
            if let Item::Statement(Stmt::Let { mutable, name, ty, value }) = item {
                if *mutable {
                    self.errors.push(format!(
                        "Global '{}' must be a constant: use 'fixum', not 'sit'", name
                    ));
                }
                let var_ty = self.infer_let_type(name, ty, value, "Global");
                self.declare_named(name, SymbolKind::Variable { mutable: *mutable, ty: var_ty });
            }
        }

        // Pass 2: check the bodies
        for item in &program.items {
            self.check_item(item);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn declare_named(&mut self, name: &str, kind: SymbolKind) {
        if !self.table.declare(name, SymbolInfo { kind }) {
            self.errors.push(format!("Name '{}' is declared more than once", name));
        }
    }

    // ---------- Resolving syntactic types into semantic ones ----------

    fn resolve_type(&mut self, t: &Type) -> Ty {
        match t {
            Type::Named(name) => self.resolve_named(name),
            Type::Generic(name, inner) => {
                let inner_ty = self.resolve_type(inner);
                if name == "series" {
                    Ty::List(Box::new(inner_ty))
                } else {
                    self.errors.push(format!("Unknown generic type '{}'", name));
                    Ty::Unknown
                }
            }
        }
    }

    fn resolve_named(&mut self, name: &str) -> Ty {
        match name {
            "numerus" => Ty::Num,
            "realis" => Ty::Real,
            "veritas" => Ty::Bool,
            "verba" => Ty::Str,
            "nihil" => Ty::Nihil,
            "series" => {
                self.errors.push(
                    "Type 'series' needs an element type, e.g. series[numerus]".to_string()
                );
                Ty::Unknown
            }
            other => {
                let kind = self.table.lookup(other).map(|i| i.kind.clone());
                match kind {
                    Some(SymbolKind::Genus { .. }) => Ty::Genus(other.to_string()),
                    Some(SymbolKind::Actor) => Ty::Actor(other.to_string()),
                    _ => {
                        self.errors.push(format!("Unknown type '{}'", other));
                        Ty::Unknown
                    }
                }
            }
        }
    }

    // Infer the type of a `sit`/`fixum` binding: from the value,
    // checking it against the annotation if one is present.
    fn infer_let_type(
        &mut self,
        name: &str,
        annotation: &Option<Type>,
        value: &Expr,
        what: &str,
    ) -> Ty {
        let value_ty = self.type_of(value);
        match annotation {
            Some(t) => {
                let declared = self.resolve_type(t);
                if !compatible(&declared, &value_ty) {
                    self.errors.push(format!(
                        "{} '{}': declared type {} but the value has type {}",
                        what, name, declared.name(), value_ty.name()
                    ));
                }
                declared
            }
            None => value_ty,
        }
    }

    // ---------- Pass 2: items ----------

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.check_function(f),
            Item::Actor(a) => self.check_actor(a),
            Item::Genus(_) | Item::Nuntius(_) | Item::Import(_) => {}
            Item::Statement(stmt) => match stmt {
                // global constants are fully handled in Pass 1c
                Stmt::Let { .. } => {}
                other => self.check_stmt(other),
            },
        }
    }

    fn check_function(&mut self, f: &Function) {
        // fetch the signature already resolved in Pass 1b
        let (param_types, ret) = match self.table.lookup(&f.name).map(|i| i.kind.clone()) {
            Some(SymbolKind::Function { params, ret }) => (params, ret),
            _ => (vec![], Ty::Nihil),
        };
        self.table.enter_scope();
        for (p, pty) in f.params.iter().zip(param_types) {
            if !self.table.declare(&p.name, SymbolInfo {
                kind: SymbolKind::Parameter { ty: pty },
            }) {
                self.errors.push(format!(
                    "Parameter '{}' is declared twice in function '{}'", p.name, f.name
                ));
            }
        }
        let saved = self.current_return.clone();
        self.current_return = ret;
        self.check_block(&f.body);
        self.current_return = saved;
        self.table.exit_scope();
    }

    fn check_actor(&mut self, a: &ActorDef) {
        self.table.enter_scope();
        let saved_actor = self.current_actor.clone();
        self.current_actor = Some(a.name.clone());

        // declare the actor's state (the sit/fixum entries)
        for stmt in &a.state {
            if let Stmt::Let { mutable, name, ty, value } = stmt {
                let var_ty = self.infer_let_type(name, ty, value, "State");
                self.table.declare(name, SymbolInfo {
                    kind: SymbolKind::Variable { mutable: *mutable, ty: var_ty },
                });
            }
        }

        // check each message handler
        for h in &a.handlers {
            let kind = self.table.lookup(&h.message).map(|i| i.kind.clone());
            match kind {
                None => self.errors.push(format!(
                    "Handler for undeclared message '{}'", h.message
                )),
                Some(SymbolKind::Nuntius { .. }) => {}
                Some(_) => self.errors.push(format!(
                    "'{}' is not a message type (nuntius)", h.message
                )),
            }
            self.table.enter_scope();
            for p in &h.params {
                let pty = self.resolve_type(&p.ty);
                self.table.declare(&p.name, SymbolInfo {
                    kind: SymbolKind::Parameter { ty: pty },
                });
            }
            let saved = self.current_return.clone();
            self.current_return = Ty::Nihil; // handlers return nothing
            self.check_block(&h.body);
            self.current_return = saved;
            self.table.exit_scope();
        }

        self.current_actor = saved_actor;
        self.table.exit_scope();
    }

    // ---------- Pass 2: statements ----------

    // Check a block in the CURRENT scope (function/handler body).
    fn check_block(&mut self, block: &[Stmt]) {
        for stmt in block {
            self.check_stmt(stmt);
        }
    }

    // Check a block in its OWN fresh scope (si / dum / nativum).
    fn check_scoped_block(&mut self, block: &[Stmt]) {
        self.table.enter_scope();
        self.check_block(block);
        self.table.exit_scope();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { mutable, name, ty, value } => {
                let var_ty = self.infer_let_type(name, ty, value, "Variable");
                if !self.table.declare(name, SymbolInfo {
                    kind: SymbolKind::Variable { mutable: *mutable, ty: var_ty },
                }) {
                    self.errors.push(format!(
                        "'{}' is already declared in this scope", name
                    ));
                }
            }
            Stmt::Assign { target, value } => {
                let tt = self.type_of(target);
                let vt = self.type_of(value);
                if !compatible(&tt, &vt) {
                    self.errors.push(format!(
                        "Cannot assign a value of type {} to a target of type {}",
                        vt.name(), tt.name()
                    ));
                }
                // a plain variable target must be mutable
                if let Expr::Ident(n) = target {
                    let k = self.table.lookup(n).map(|i| i.kind.clone());
                    if let Some(SymbolKind::Variable { mutable: false, .. }) = k {
                        self.errors.push(format!(
                            "Cannot assign to '{}': it is a constant (fixum)", n
                        ));
                    }
                }
            }
            Stmt::If { cond, then_block, elif, else_block } => {
                self.expect_bool(cond, "'si' condition");
                self.check_scoped_block(then_block);
                for (ec, eb) in elif {
                    self.expect_bool(ec, "'aliter si' condition");
                    self.check_scoped_block(eb);
                }
                if let Some(eb) = else_block {
                    self.check_scoped_block(eb);
                }
            }
            Stmt::While { cond, body } => {
                self.expect_bool(cond, "'dum' condition");
                self.check_scoped_block(body);
            }
            Stmt::For { var, iter, body } => {
                let it = self.type_of(iter);
                let elem = match it {
                    Ty::List(e) => *e,
                    Ty::Unknown => Ty::Unknown,
                    other => {
                        self.errors.push(format!(
                            "'per' needs a series to iterate over, found {}", other.name()
                        ));
                        Ty::Unknown
                    }
                };
                self.table.enter_scope();
                self.table.declare(var, SymbolInfo {
                    kind: SymbolKind::Variable { mutable: false, ty: elem },
                });
                self.check_block(body);
                self.table.exit_scope();
            }
            Stmt::Return(opt) => {
                let actual = match opt {
                    Some(e) => self.type_of(e),
                    None => Ty::Nihil,
                };
                let expected = self.current_return.clone();
                if !compatible(&actual, &expected) {
                    self.errors.push(format!(
                        "'redde' returns {} but the function expects {}",
                        actual.name(), expected.name()
                    ));
                }
            }
            Stmt::Break | Stmt::Continue => {}
            Stmt::Send { message, target } => {
                self.check_message(message);
                let t = self.type_of(target);
                match t {
                    Ty::Actor(_) | Ty::Unknown => {}
                    other => self.errors.push(format!(
                        "'mitte ... ad' needs an actor as target, found {}", other.name()
                    )),
                }
            }
            Stmt::Nativum(body) => self.check_scoped_block(body),
            Stmt::Expr(e) => {
                self.type_of(e);
            }
        }
    }

    // Helper: an expression that must be a veritas (boolean).
    fn expect_bool(&mut self, expr: &Expr, what: &str) {
        let t = self.type_of(expr);
        if !compatible(&t, &Ty::Bool) {
            self.errors.push(format!(
                "{} must be a veritas, found {}", what, t.name()
            ));
        }
    }

    // The message of `mitte`: must name a nuntius.
    fn check_message(&mut self, message: &Expr) {
        match message {
            Expr::Ident(name) => {
                let k = self.table.lookup(name).map(|i| i.kind.clone());
                match k {
                    Some(SymbolKind::Nuntius { .. }) => {}
                    None => self.errors.push(format!("Unknown message '{}'", name)),
                    Some(_) => self.errors.push(format!(
                        "'{}' is not a message (nuntius)", name
                    )),
                }
            }
            Expr::Call { callee, args } => {
                if let Expr::Ident(name) = callee.as_ref() {
                    let k = self.table.lookup(name).map(|i| i.kind.clone());
                    match k {
                        Some(SymbolKind::Nuntius { params }) => {
                            if params.len() != args.len() {
                                self.errors.push(format!(
                                    "Message '{}' expects {} argument(s), got {}",
                                    name, params.len(), args.len()
                                ));
                            }
                            for (i, (pt, a)) in params.iter().zip(args).enumerate() {
                                let at = self.type_of(a);
                                if !compatible(pt, &at) {
                                    self.errors.push(format!(
                                        "Message '{}' argument {}: expected {}, got {}",
                                        name, i + 1, pt.name(), at.name()
                                    ));
                                }
                            }
                        }
                        None => self.errors.push(format!("Unknown message '{}'", name)),
                        Some(_) => self.errors.push(format!(
                            "'{}' is not a message (nuntius)", name
                        )),
                    }
                } else {
                    self.errors.push("A message must be a nuntius name".to_string());
                }
            }
            _ => self.errors.push("A message must be a nuntius name".to_string()),
        }
    }

    // ---------- Pass 2: expressions — compute the type ----------

    fn type_of(&mut self, expr: &Expr) -> Ty {
        match expr {
            Expr::IntLit(_) => Ty::Num,
            Expr::FloatLit(_) => Ty::Real,
            Expr::StrLit(_) => Ty::Str,
            Expr::BoolLit(_) => Ty::Bool,
            Expr::Nihil => Ty::Nihil,

            Expr::Ipse => match &self.current_actor {
                Some(name) => Ty::Actor(name.clone()),
                None => {
                    self.errors.push("'ipse' can only be used inside an actor".to_string());
                    Ty::Unknown
                }
            },

            Expr::Ident(name) => {
                let kind = self.table.lookup(name).map(|i| i.kind.clone());
                match kind {
                    None => {
                        self.errors.push(format!("Use of undeclared name '{}'", name));
                        Ty::Unknown
                    }
                    Some(SymbolKind::Variable { ty, .. }) => ty,
                    Some(SymbolKind::Parameter { ty }) => ty,
                    Some(SymbolKind::Function { .. }) => Ty::Unknown, // no first-class functions
                    Some(_) => {
                        self.errors.push(format!(
                            "'{}' is a type or message, not a value", name
                        ));
                        Ty::Unknown
                    }
                }
            }

            Expr::Unary { op, operand } => {
                let t = self.type_of(operand);
                match op {
                    UnOp::Neg => {
                        if !is_number(&t) {
                            self.errors.push(format!(
                                "Unary '-' needs a number, found {}", t.name()
                            ));
                            Ty::Unknown
                        } else {
                            t
                        }
                    }
                    UnOp::Not => {
                        if !compatible(&t, &Ty::Bool) {
                            self.errors.push(format!(
                                "'non' needs a veritas, found {}", t.name()
                            ));
                        }
                        Ty::Bool
                    }
                }
            }

            Expr::Binary { op, left, right } => {
                let l = self.type_of(left);
                let r = self.type_of(right);
                self.check_binary(op, &l, &r)
            }

            Expr::Call { callee, args } => self.type_of_call(callee, args),

            Expr::Index { object, index } => {
                let obj = self.type_of(object);
                let idx = self.type_of(index);
                if !compatible(&idx, &Ty::Num) {
                    self.errors.push(format!(
                        "A list index must be a numerus, found {}", idx.name()
                    ));
                }
                match obj {
                    Ty::List(elem) => *elem,
                    Ty::Unknown => Ty::Unknown,
                    other => {
                        self.errors.push(format!(
                            "Cannot index into {}; a series is required", other.name()
                        ));
                        Ty::Unknown
                    }
                }
            }

            Expr::Field { object, name } => {
                let obj = self.type_of(object);
                match obj {
                    Ty::Genus(gname) => {
                        let fields = match self.table.lookup(&gname).map(|i| i.kind.clone()) {
                            Some(SymbolKind::Genus { fields }) => fields,
                            _ => Vec::new(),
                        };
                        match fields.iter().find(|(fname, _)| fname == name) {
                            Some((_, fty)) => fty.clone(),
                            None => {
                                self.errors.push(format!(
                                    "Genus '{}' has no field '{}'", gname, name
                                ));
                                Ty::Unknown
                            }
                        }
                    }
                    Ty::Unknown => Ty::Unknown,
                    other => {
                        self.errors.push(format!(
                            "Cannot access field '{}' on {}", name, other.name()
                        ));
                        Ty::Unknown
                    }
                }
            }

            Expr::List(items) => {
                if items.is_empty() {
                    Ty::List(Box::new(Ty::Unknown))
                } else {
                    let first = self.type_of(&items[0]);
                    for it in &items[1..] {
                        let t = self.type_of(it);
                        if !compatible(&t, &first) {
                            self.errors.push(format!(
                                "List elements must share one type: {} vs {}",
                                first.name(), t.name()
                            ));
                        }
                    }
                    Ty::List(Box::new(first))
                }
            }

            Expr::Create { name, args } => {
                // Type-check all argument expressions first.
                for (_, expr) in args {
                    self.type_of(expr);
                }
                let kind = self.table.lookup(name).map(|i| i.kind.clone());
                match kind {
                    Some(SymbolKind::Actor) => {
                        if !args.is_empty() {
                            self.errors.push(format!(
                                "Actor '{}' is created without arguments", name
                            ));
                        }
                        Ty::Actor(name.clone())
                    }
                    Some(SymbolKind::Genus { fields }) => {
                        // Check that every supplied field exists in the genus.
                        for (field_name, _) in args {
                            if !fields.iter().any(|(f, _)| f == field_name) {
                                self.errors.push(format!(
                                    "'crea {}': unknown field '{}'", name, field_name
                                ));
                            }
                        }
                        // Check that every required field is supplied.
                        for (field_name, _) in &fields {
                            if !args.iter().any(|(k, _)| k == field_name) {
                                self.errors.push(format!(
                                    "'crea {}': missing field '{}'", name, field_name
                                ));
                            }
                        }
                        Ty::Genus(name.clone())
                    }
                    None => {
                        self.errors.push(format!("Use of undeclared type '{}'", name));
                        Ty::Unknown
                    }
                    Some(_) => {
                        self.errors.push(format!("'{}' is not a genus or actor", name));
                        Ty::Unknown
                    }
                }
            }
        }
    }

    // Type rules for binary operators.
    fn check_binary(&mut self, op: &BinOp, l: &Ty, r: &Ty) -> Ty {
        use BinOp::*;
        match op {
            Add | Sub | Mul | Div | Mod => {
                let ok = (compatible(l, &Ty::Num) && compatible(r, &Ty::Num))
                    || (compatible(l, &Ty::Real) && compatible(r, &Ty::Real));
                if !ok {
                    self.errors.push(format!(
                        "Arithmetic needs two numbers of the same type, found {} and {}",
                        l.name(), r.name()
                    ));
                    Ty::Unknown
                } else if *l == Ty::Unknown {
                    r.clone()
                } else {
                    l.clone()
                }
            }
            Eq | Neq => {
                if !compatible(l, r) {
                    self.errors.push(format!(
                        "Cannot compare {} with {}", l.name(), r.name()
                    ));
                }
                Ty::Bool
            }
            Lt | Gt | Le | Ge => {
                if !is_number(l) || !is_number(r) {
                    self.errors.push(format!(
                        "Comparison needs two numbers, found {} and {}",
                        l.name(), r.name()
                    ));
                }
                Ty::Bool
            }
            And | Or => {
                if !compatible(l, &Ty::Bool) || !compatible(r, &Ty::Bool) {
                    self.errors.push(format!(
                        "'et'/'vel' need two veritas, found {} and {}",
                        l.name(), r.name()
                    ));
                }
                Ty::Bool
            }
        }
    }

    // Type-check a function call and return its result type.
    fn type_of_call(&mut self, callee: &Expr, args: &[Expr]) -> Ty {
        let mut arg_types = Vec::new();
        for a in args {
            arg_types.push(self.type_of(a));
        }

        if let Expr::Ident(fname) = callee {
            // 'scribe' is the variadic built-in: accept any args
            if fname == "scribe" {
                return Ty::Nihil;
            }
            // 'lege' takes an optional verba prompt and returns a verba
            if fname == "lege" {
                if arg_types.len() > 1 {
                    self.errors.push(
                        "'lege' takes at most one argument (a prompt)".to_string()
                    );
                } else if arg_types.len() == 1 && !compatible(&arg_types[0], &Ty::Str) {
                    self.errors.push(format!(
                        "'lege' prompt must be a verba, found {}", arg_types[0].name()
                    ));
                }
                return Ty::Str;
            }
            let kind = self.table.lookup(fname).map(|i| i.kind.clone());
            match kind {
                Some(SymbolKind::Function { params, ret }) => {
                    if params.len() != arg_types.len() {
                        self.errors.push(format!(
                            "Function '{}' expects {} argument(s), got {}",
                            fname, params.len(), arg_types.len()
                        ));
                    } else {
                        for (i, (pt, at)) in params.iter().zip(&arg_types).enumerate() {
                            if !compatible(pt, at) {
                                self.errors.push(format!(
                                    "Function '{}' argument {}: expected {}, got {}",
                                    fname, i + 1, pt.name(), at.name()
                                ));
                            }
                        }
                    }
                    ret
                }
                None => {
                    self.errors.push(format!("Call to undeclared function '{}'", fname));
                    Ty::Unknown
                }
                Some(_) => {
                    self.errors.push(format!("'{}' is not a function", fname));
                    Ty::Unknown
                }
            }
        } else {
            self.type_of(callee);
            self.errors.push("Only named functions can be called".to_string());
            Ty::Unknown
        }
    }
}
