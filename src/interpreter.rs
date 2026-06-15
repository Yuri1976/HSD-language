// ============================================================
//  HSD — Hic Sunt Dracones
//  Phase 4c: the INTERPRETER (with actors)
//
//  Adds a SIMPLIFIED, SYNCHRONOUS actor model:
//   - `crea` builds an actor instance with its own state
//   - `mitte` sends a message: the matching `accipe` handler
//     runs immediately, on this thread
//   - `ipse` refers to the current actor
//
//  This gives actors their SEMANTICS (state + handlers +
//  identity), not yet true concurrency. Real parallelism —
//  mailboxes, a scheduler, OS threads — is Phase 5 runtime
//  work.
// ============================================================

#![allow(dead_code)]

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;
use crate::ast::*;

// ---------- Runtime values ----------

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),
    // An actor instance. Rc<RefCell<...>> gives it a shared,
    // mutable IDENTITY: copying the Value copies the handle,
    // not the actor — both refer to the same actor.
    Actor(Rc<RefCell<ActorInstance>>),
    // A genus record instance. Rc<RefCell<...>> gives it reference
    // semantics: assigning a record copies the handle, not the data.
    Record(Rc<RefCell<RecordInstance>>),
    Nihil,
}

// A live genus record: its type name and its current field values.
#[derive(Debug)]
pub struct RecordInstance {
    pub type_name: String,
    pub fields: HashMap<String, Value>,
}

// A live actor: its type and its current state.
#[derive(Debug)]
pub struct ActorInstance {
    type_name: String,
    state: HashMap<String, Value>,
}

impl Value {
    fn display(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => {
                if *b { "verum".to_string() } else { "falsum".to_string() }
            }
            Value::Str(s) => s.clone(),
            Value::Nihil => "nihil".to_string(),
            Value::List(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.display()).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::Actor(rc) => format!("<actor {}>", rc.borrow().type_name),
            Value::Record(rc) => {
                let r = rc.borrow();
                let mut parts: Vec<String> = r.fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.display()))
                    .collect();
                parts.sort(); // stable display order
                format!("{}({})", r.type_name, parts.join(", "))
            }
        }
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Nihil, Value::Nihil) => true,
        // two actors are equal only if they are the SAME actor
        (Value::Actor(x), Value::Actor(y)) => Rc::ptr_eq(x, y),
        // two records are equal if they are the same instance (identity),
        // or if they have the same type and all fields are equal
        (Value::Record(x), Value::Record(y)) => {
            if Rc::ptr_eq(x, y) { return true; }
            let rx = x.borrow();
            let ry = y.borrow();
            if rx.type_name != ry.type_name { return false; }
            if rx.fields.len() != ry.fields.len() { return false; }
            rx.fields.iter().all(|(k, v)| {
                ry.fields.get(k).map_or(false, |w| values_equal(v, w))
            })
        }
        _ => false,
    }
}

// ---------- The Environment ----------

struct Environment {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
}

impl Environment {
    fn new() -> Environment {
        Environment { globals: HashMap::new(), locals: Vec::new() }
    }
    fn enter_scope(&mut self) {
        self.locals.push(HashMap::new());
    }
    fn exit_scope(&mut self) {
        self.locals.pop();
    }
    fn define(&mut self, name: &str, value: Value) {
        match self.locals.last_mut() {
            Some(scope) => { scope.insert(name.to_string(), value); }
            None => { self.globals.insert(name.to_string(), value); }
        }
    }
    fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.locals.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        self.globals.get(name)
    }
    fn set(&mut self, name: &str, value: Value) -> bool {
        for scope in self.locals.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return true;
        }
        false
    }
}

// ---------- Control flow signals ----------

enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

// ---------- The Interpreter ----------

pub struct Interpreter {
    env: Environment,
    functions: HashMap<String, Function>,
    actors: HashMap<String, ActorDef>,
    genera: HashMap<String, GenusDef>,      // Phase 8: genus definitions
    current_actor: Option<Value>, // the actor whose handler is running ('ipse')
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            env: Environment::new(),
            functions: HashMap::new(),
            actors: HashMap::new(),
            genera: HashMap::new(),
            current_actor: None,
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), String> {
        // collect functions, actor definitions, and genus definitions
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    self.functions.insert(f.name.clone(), f.clone());
                }
                Item::Actor(a) => {
                    self.actors.insert(a.name.clone(), a.clone());
                }
                Item::Genus(g) => {
                    self.genera.insert(g.name.clone(), g.clone());
                }
                _ => {}
            }
        }
        // run top-level statements (global constants)
        for item in &program.items {
            if let Item::Statement(stmt) = item {
                self.exec_stmt(stmt)?;
            }
        }
        if !self.functions.contains_key("principale") {
            return Err("no 'principale' function (the entry point) found".to_string());
        }
        self.call_function("principale", Vec::new())?;
        Ok(())
    }

    // ---------- Function calls ----------

    fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        let func = match self.functions.get(name) {
            Some(f) => f.clone(),
            None => return Err(format!("call to undefined function '{}'", name)),
        };
        if func.params.len() != args.len() {
            return Err(format!(
                "function '{}' expects {} argument(s), got {}",
                name, func.params.len(), args.len()
            ));
        }
        let saved = std::mem::replace(&mut self.env.locals, vec![HashMap::new()]);
        for (param, arg) in func.params.iter().zip(args) {
            self.env.define(&param.name, arg);
        }
        let result = self.exec_block(&func.body);
        self.env.locals = saved;
        match result? {
            Flow::Return(v) => Ok(v),
            Flow::Normal => Ok(Value::Nihil),
            Flow::Break | Flow::Continue => {
                Err("'frange'/'perge' used outside a loop".to_string())
            }
        }
    }

    // ---------- Actors ----------

    // `crea Name` : build a new actor instance with fresh state.
    fn create_actor(&mut self, name: &str) -> Result<Value, String> {
        let def = match self.actors.get(name) {
            Some(d) => d.clone(),
            None => return Err(format!("'crea': unknown actor '{}'", name)),
        };
        // evaluate the state initializers on a fresh frame
        let saved = std::mem::replace(&mut self.env.locals, vec![HashMap::new()]);
        let mut state = HashMap::new();
        let mut outcome = Ok(());
        for stmt in &def.state {
            if let Stmt::Let { name: var, value, .. } = stmt {
                match self.eval_expr(value) {
                    Ok(v) => { state.insert(var.clone(), v); }
                    Err(e) => { outcome = Err(e); break; }
                }
            }
        }
        self.env.locals = saved;
        outcome?;

        let instance = ActorInstance { type_name: name.to_string(), state };
        Ok(Value::Actor(Rc::new(RefCell::new(instance))))
    }

    // `crea Name(field: expr, ...)` : construct a genus record.
    fn create_record(&mut self, name: &str, args: &[(String, Expr)]) -> Result<Value, String> {
        let def = match self.genera.get(name) {
            Some(d) => d.clone(),
            None => return Err(format!("'crea': unknown genus '{}'", name)),
        };

        // Check that every supplied field exists in the genus definition.
        for (field_name, _) in args {
            if !def.fields.iter().any(|f| &f.name == field_name) {
                return Err(format!(
                    "'crea {}': unknown field '{}'", name, field_name
                ));
            }
        }

        // Check that every required field has been supplied.
        for field in &def.fields {
            if !args.iter().any(|(k, _)| k == &field.name) {
                return Err(format!(
                    "'crea {}': missing field '{}'", name, field.name
                ));
            }
        }

        // Evaluate argument expressions and build the field map.
        let mut fields = HashMap::new();
        for (field_name, expr) in args {
            let v = self.eval_expr(expr)?;
            fields.insert(field_name.clone(), v);
        }

        let instance = RecordInstance { type_name: name.to_string(), fields };
        Ok(Value::Record(Rc::new(RefCell::new(instance))))
    }

    // `mitte message ad target` : run the matching handler.
    fn send_message(&mut self, message: &Expr, target: &Expr) -> Result<(), String> {
        // the target must evaluate to an actor
        let target_val = self.eval_expr(target)?;
        let actor_rc = match target_val {
            Value::Actor(rc) => rc,
            other => {
                return Err(format!(
                    "'mitte ... ad' needs an actor, found {}", other.display()
                ));
            }
        };

        // work out the message name and its argument values
        let (msg_name, arg_values) = match message {
            Expr::Ident(n) => (n.clone(), Vec::new()),
            Expr::Call { callee, args } => {
                let n = match callee.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err("a message must be a nuntius name".to_string()),
                };
                let mut vals = Vec::new();
                for a in args {
                    vals.push(self.eval_expr(a)?);
                }
                (n, vals)
            }
            _ => return Err("a message must be a nuntius name".to_string()),
        };

        // find the actor's definition and the matching handler
        let type_name = actor_rc.borrow().type_name.clone();
        let def = match self.actors.get(&type_name) {
            Some(d) => d.clone(),
            None => return Err(format!("unknown actor type '{}'", type_name)),
        };
        let handler = match def.handlers.iter().find(|h| h.message == msg_name).cloned() {
            Some(h) => h,
            None => {
                return Err(format!(
                    "actor '{}' has no handler for message '{}'", type_name, msg_name
                ));
            }
        };
        if handler.params.len() != arg_values.len() {
            return Err(format!(
                "message '{}' expects {} argument(s), got {}",
                msg_name, handler.params.len(), arg_values.len()
            ));
        }

        // --- run the handler ---
        // Copy the actor's state out into a scope (we must not
        // hold a borrow on the actor while its handler runs).
        let state_snapshot = actor_rc.borrow().state.clone();

        // handler frame: bottom scope = state, upper scope =
        // parameters and the handler's own local variables
        let saved_locals = std::mem::replace(
            &mut self.env.locals, vec![state_snapshot]
        );
        self.env.enter_scope();
        for (p, v) in handler.params.iter().zip(arg_values) {
            self.env.define(&p.name, v);
        }
        // make 'ipse' refer to this actor
        let saved_actor = self.current_actor.take();
        self.current_actor = Some(Value::Actor(Rc::clone(&actor_rc)));

        let result = self.exec_block(&handler.body);

        // restore 'ipse'
        self.current_actor = saved_actor;
        // drop the params scope; the bottom scope is the new state
        self.env.exit_scope();
        let new_state = self.env.locals.pop().unwrap_or_default();
        self.env.locals = saved_locals;

        // commit the updated state back into the actor
        actor_rc.borrow_mut().state = new_state;

        result?;
        Ok(())
    }

    // ---------- Executing statements ----------

    fn exec_block(&mut self, block: &[Stmt]) -> Result<Flow, String> {
        for stmt in block {
            let flow = self.exec_stmt(stmt)?;
            if !matches!(flow, Flow::Normal) {
                return Ok(flow);
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_scoped_block(&mut self, block: &[Stmt]) -> Result<Flow, String> {
        self.env.enter_scope();
        let flow = self.exec_block(block);
        self.env.exit_scope();
        flow
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Flow, String> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let v = self.eval_expr(value)?;
                self.env.define(name, v);
                Ok(Flow::Normal)
            }

            Stmt::Assign { target, value } => {
                let v = self.eval_expr(value)?;
                match target {
                    Expr::Ident(name) => {
                        if !self.env.set(name, v) {
                            return Err(format!(
                                "assignment to undeclared variable '{}'", name
                            ));
                        }
                    }
                    Expr::Field { object, name: field_name } => {
                        // Evaluate the object to get the record, then update the field.
                        let obj_val = self.eval_expr(object)?;
                        match obj_val {
                            Value::Record(rc) => {
                                let mut r = rc.borrow_mut();
                                if !r.fields.contains_key(field_name.as_str()) {
                                    return Err(format!(
                                        "record '{}' has no field '{}'",
                                        r.type_name, field_name
                                    ));
                                }
                                r.fields.insert(field_name.clone(), v);
                            }
                            other => return Err(format!(
                                "field assignment '{}' on a non-record value: {}",
                                field_name, other.display()
                            )),
                        }
                    }
                    _ => return Err("invalid assignment target".to_string()),
                }
                Ok(Flow::Normal)
            }

            Stmt::If { cond, then_block, elif, else_block } => {
                if self.eval_bool(cond)? {
                    return self.exec_scoped_block(then_block);
                }
                for (ec, eb) in elif {
                    if self.eval_bool(ec)? {
                        return self.exec_scoped_block(eb);
                    }
                }
                if let Some(eb) = else_block {
                    return self.exec_scoped_block(eb);
                }
                Ok(Flow::Normal)
            }

            Stmt::While { cond, body } => {
                while self.eval_bool(cond)? {
                    match self.exec_scoped_block(body)? {
                        Flow::Normal | Flow::Continue => {}
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                    }
                }
                Ok(Flow::Normal)
            }

            Stmt::For { var, iter, body } => {
                let iter_val = self.eval_expr(iter)?;
                let items = match iter_val {
                    Value::List(items) => items,
                    _ => return Err("'per' needs a series to iterate over".to_string()),
                };
                for item in items {
                    self.env.enter_scope();
                    self.env.define(var, item);
                    let flow = self.exec_block(body);
                    self.env.exit_scope();
                    match flow? {
                        Flow::Normal | Flow::Continue => {}
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                    }
                }
                Ok(Flow::Normal)
            }

            Stmt::Return(opt) => {
                let v = match opt {
                    Some(e) => self.eval_expr(e)?,
                    None => Value::Nihil,
                };
                Ok(Flow::Return(v))
            }

            Stmt::Break => Ok(Flow::Break),
            Stmt::Continue => Ok(Flow::Continue),

            Stmt::Nativum(body) => self.exec_scoped_block(body),

            Stmt::Send { message, target } => {
                self.send_message(message, target)?;
                Ok(Flow::Normal)
            }

            Stmt::Expr(e) => {
                self.eval_expr(e)?;
                Ok(Flow::Normal)
            }
        }
    }

    // ---------- Evaluating expressions ----------

    fn eval_bool(&mut self, expr: &Expr) -> Result<bool, String> {
        match self.eval_expr(expr)? {
            Value::Bool(b) => Ok(b),
            other => Err(format!("expected a veritas, got {}", other.display())),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::IntLit(n) => Ok(Value::Int(*n)),
            Expr::FloatLit(f) => Ok(Value::Float(*f)),
            Expr::StrLit(s) => Ok(Value::Str(s.clone())),
            Expr::BoolLit(b) => Ok(Value::Bool(*b)),
            Expr::Nihil => Ok(Value::Nihil),

            Expr::Ident(name) => match self.env.get(name) {
                Some(v) => Ok(v.clone()),
                None => Err(format!("undeclared variable '{}'", name)),
            },

            Expr::Ipse => match &self.current_actor {
                Some(v) => Ok(v.clone()),
                None => Err("'ipse' used outside an actor handler".to_string()),
            },

            Expr::Unary { op, operand } => {
                let v = self.eval_expr(operand)?;
                match (op, v) {
                    (UnOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
                    (UnOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
                    (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err("invalid operand for a unary operator".to_string()),
                }
            }

            Expr::Binary { op, left, right } => {
                match op {
                    BinOp::And => {
                        if !self.eval_bool(left)? {
                            return Ok(Value::Bool(false));
                        }
                        return Ok(Value::Bool(self.eval_bool(right)?));
                    }
                    BinOp::Or => {
                        if self.eval_bool(left)? {
                            return Ok(Value::Bool(true));
                        }
                        return Ok(Value::Bool(self.eval_bool(right)?));
                    }
                    _ => {}
                }
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                apply_binary(op, l, r)
            }

            Expr::Call { callee, args } => {
                if let Expr::Ident(name) = callee.as_ref() {
                    let mut arg_values = Vec::new();
                    for a in args {
                        arg_values.push(self.eval_expr(a)?);
                    }
                    if name == "scribe" {
                        let mut out = String::new();
                        for v in &arg_values {
                            out.push_str(&v.display());
                        }
                        println!("{}", out);
                        return Ok(Value::Nihil);
                    }
                    if name == "numera" {
                        return builtin_numera(&arg_values);
                    }
                    if name == "lege" {
                        return builtin_lege(&arg_values);
                    }
                    if name == "numerus_ex" {
                        return builtin_numerus_ex(&arg_values);
                    }
                    if name == "realis_ex" {
                        return builtin_realis_ex(&arg_values);
                    }
                    return self.call_function(name, arg_values);
                }
                Err("only named functions can be called".to_string())
            }

            Expr::Index { object, index } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;
                match (obj, idx) {
                    (Value::List(items), Value::Int(i)) => {
                        if i < 0 || (i as usize) >= items.len() {
                            Err(format!(
                                "list index {} out of bounds (length {})",
                                i, items.len()
                            ))
                        } else {
                            Ok(items[i as usize].clone())
                        }
                    }
                    _ => Err("invalid indexing".to_string()),
                }
            }

            Expr::List(items) => {
                let mut values = Vec::new();
                for it in items {
                    values.push(self.eval_expr(it)?);
                }
                Ok(Value::List(values))
            }

            Expr::Create { name, args } => {
                // Is this a genus record or an actor?
                if self.genera.contains_key(name.as_str()) {
                    self.create_record(name, args)
                } else {
                    // actor: named args must be empty
                    if !args.is_empty() {
                        return Err(
                            "actors are created without arguments; their state \
                             is set by their 'sit' fields".to_string()
                        );
                    }
                    self.create_actor(name)
                }
            }

            Expr::Field { object, name } => {
                let val = self.eval_expr(object)?;
                match val {
                    Value::Record(rc) => {
                        match rc.borrow().fields.get(name.as_str()) {
                            Some(v) => Ok(v.clone()),
                            None => Err(format!(
                                "record '{}' has no field '{}'",
                                rc.borrow().type_name, name
                            )),
                        }
                    }
                    other => Err(format!(
                        "field access '{}' on a non-record value: {}",
                        name, other.display()
                    )),
                }
            }
        }
    }
}

// ---------- Built-in functions ----------

fn builtin_numera(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Int(a), Value::Int(b)] => {
            let mut list = Vec::new();
            let mut i = *a;
            while i <= *b {
                list.push(Value::Int(i));
                i += 1;
            }
            Ok(Value::List(list))
        }
        _ => Err("numera(a, b) needs two numerus arguments".to_string()),
    }
}

// Read one line from the keyboard, with an optional prompt.
// Always returns a verba (string), like Python's input().
fn builtin_lege(args: &[Value]) -> Result<Value, String> {
    if let Some(prompt) = args.first() {
        print!("{}", prompt.display());
        std::io::stdout().flush().ok(); // show the prompt before waiting
    }
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(0) => Ok(Value::Str(String::new())), // end of input
        Ok(_) => {
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
            Ok(Value::Str(trimmed.to_string()))
        }
        Err(e) => Err(format!("input error: {}", e)),
    }
}

// Convert a verba into a numerus.
fn builtin_numerus_ex(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s)] => match s.trim().parse::<i64>() {
            Ok(n) => Ok(Value::Int(n)),
            Err(_) => Err(format!("numerus_ex: '{}' is not a valid numerus", s)),
        },
        _ => Err("numerus_ex(verba) needs one verba argument".to_string()),
    }
}

// Convert a verba into a realis.
fn builtin_realis_ex(args: &[Value]) -> Result<Value, String> {
    match args {
        [Value::Str(s)] => match s.trim().parse::<f64>() {
            Ok(f) => Ok(Value::Float(f)),
            Err(_) => Err(format!("realis_ex: '{}' is not a valid realis", s)),
        },
        _ => Err("realis_ex(verba) needs one verba argument".to_string()),
    }
}

// ---------- Binary operators ----------

fn apply_binary(op: &BinOp, l: Value, r: Value) -> Result<Value, String> {
    use BinOp::*;
    use Value::*;
    match (op, l, r) {
        (Add, Int(a), Int(b)) => Ok(Int(a + b)),
        (Sub, Int(a), Int(b)) => Ok(Int(a - b)),
        (Mul, Int(a), Int(b)) => Ok(Int(a * b)),
        (Div, Int(a), Int(b)) => {
            if b == 0 { Err("division by zero".to_string()) }
            else { Ok(Int(a / b)) }
        }
        (Mod, Int(a), Int(b)) => {
            if b == 0 { Err("modulo by zero".to_string()) }
            else { Ok(Int(a % b)) }
        }
        (Add, Float(a), Float(b)) => Ok(Float(a + b)),
        (Sub, Float(a), Float(b)) => Ok(Float(a - b)),
        (Mul, Float(a), Float(b)) => Ok(Float(a * b)),
        (Div, Float(a), Float(b)) => Ok(Float(a / b)),
        (Mod, Float(a), Float(b)) => Ok(Float(a % b)),
        (Lt, Int(a), Int(b)) => Ok(Bool(a < b)),
        (Gt, Int(a), Int(b)) => Ok(Bool(a > b)),
        (Le, Int(a), Int(b)) => Ok(Bool(a <= b)),
        (Ge, Int(a), Int(b)) => Ok(Bool(a >= b)),
        (Lt, Float(a), Float(b)) => Ok(Bool(a < b)),
        (Gt, Float(a), Float(b)) => Ok(Bool(a > b)),
        (Le, Float(a), Float(b)) => Ok(Bool(a <= b)),
        (Ge, Float(a), Float(b)) => Ok(Bool(a >= b)),
        (Eq, a, b) => Ok(Bool(values_equal(&a, &b))),
        (Neq, a, b) => Ok(Bool(!values_equal(&a, &b))),
        _ => Err("operator applied to values of the wrong type".to_string()),
    }
}
