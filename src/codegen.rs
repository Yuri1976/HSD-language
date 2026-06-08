// ============================================================
//  HSD — Hic Sunt Dracones
//  C BACKEND (code generator)
//
//  Translates the AST into C source. A C compiler (gcc/cl)
//  turns that into a native executable.
//
//  Currently covered:
//    — functions, numbers, booleans, si/dum, redde,
//      scribe with literals and numbers
//    - verba variables, lege/numerus_ex/realis_ex,
//      series[numerus], numera, the 'per' loop
//    — actors: struct, crea, mitte, accipe, ipse
//    — Phase 6b: ARC integration for hsd_list_num and actor pointers
//    — Phase 6c: ARC integration for verba (strings)
//    — Phase 6d: reassignment releases the old value
//
//  Still NOT covered: genus (records), series[verba/realis],
//  list literals like [1, 2, 3].
// ============================================================

#![allow(dead_code)]

use std::collections::HashMap;
use crate::ast::*;

pub struct CodeGen {
    out: String,
    indent: usize,
    func_ret: HashMap<String, String>,     // function name -> C return type
    locals: HashMap<String, String>,       // variable name -> C type
    state_vars: HashMap<String, String>,   // actor state fields (when in a handler)
    current_actor: Option<String>,         // the actor whose handler is being emitted
    tmp_counter: usize,                    // unique names for temporaries

    // Phase 6b — ARC scope tracking.
    // Each entry on the stack is a list of (var_name, c_type) for
    // heap-tracked variables in that scope. When a scope ends, those
    // variables get hsd_arc_release calls emitted before the closing }.
    scope_stack: Vec<Vec<(String, String)>>,

    // Phase 6b — when inside a nativum block, suspend ARC tracking
    // (the programmer manages memory manually inside nativum).
    in_nativum: bool,
}

impl CodeGen {
    pub fn new() -> CodeGen {
        CodeGen {
            out: String::new(),
            indent: 0,
            func_ret: HashMap::new(),
            locals: HashMap::new(),
            state_vars: HashMap::new(),
            current_actor: None,
            tmp_counter: 0,
            scope_stack: Vec::new(),
            in_nativum: false,
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String, String> {
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    let ret = match &f.return_type {
                        Some(t) => c_type(t)?,
                        None => "void".to_string(),
                    };
                    self.func_ret.insert(f.name.clone(), ret);
                }
                Item::Genus(_) => {
                    return Err("the C backend does not support 'genus' yet".into());
                }
                _ => {}
            }
        }

        // ---- 1. Includes ----
        self.out.push_str("#include <stdio.h>\n");
        self.out.push_str("#include <stdlib.h>\n");
        self.out.push_str("#include \"runtime.h\"\n\n");

        // ---- 2. Actor typedefs ----
        let has_actor = program.items.iter().any(|i| matches!(i, Item::Actor(_)));
        for item in &program.items {
            if let Item::Actor(a) = item {
                self.out.push_str(&format!("typedef struct {} {};\n", a.name, a.name));
            }
        }
        if has_actor { self.out.push('\n'); }

        // ---- 3. Actor struct definitions ----
        for item in &program.items {
            if let Item::Actor(a) = item {
                self.gen_actor_struct(a)?;
            }
        }

        // ---- 4. Forward declarations ----
        for item in &program.items {
            match item {
                Item::Function(f) => {
                    let sig = self.func_signature(f)?;
                    self.out.push_str(&sig);
                    self.out.push_str(";\n");
                }
                Item::Actor(a) => {
                    self.out.push_str(&self.actor_ctor_sig(a));
                    self.out.push_str(";\n");
                    for h in &a.handlers {
                        self.out.push_str(&self.actor_handler_sig(&a.name, h)?);
                        self.out.push_str(";\n");
                    }
                }
                _ => {}
            }
        }
        self.out.push('\n');

        // ---- 5. Function bodies ----
        for item in &program.items {
            if let Item::Function(f) = item {
                self.gen_function(f)?;
                self.out.push('\n');
            }
        }

        // ---- 6. Actor constructors and handlers ----
        for item in &program.items {
            if let Item::Actor(a) = item {
                self.gen_actor_ctor(a)?;
                for h in &a.handlers {
                    self.gen_actor_handler(&a.name, a, h)?;
                }
            }
        }

        // ---- 7. main bridge ----
        self.out.push_str("int main(void) {\n    principale();\n    return 0;\n}\n");

        Ok(self.out.clone())
    }

    // ============================================================
    // ARC scope management (Phase 6b/6c/6d)
    // ============================================================

    /// Returns true if the given C type is heap-tracked (managed by ARC).
    /// As of Phase 6c: const char* (verba), hsd_list_num, and any pointer
    /// other than void*.
    fn is_arc_tracked(c_type: &str) -> bool {
        if c_type == "hsd_list_num" {
            return true;
        }
        if c_type == "void*" {
            return false;
        }
        if c_type == "const char*" {
            return true; // Phase 6c: strings are now ARC-tracked
        }
        c_type.ends_with('*')
    }

    /// Emit a single release call for a heap-tracked variable. Handles
    /// the special case of hsd_list_num (release .data, not the struct).
    /// For const char* we cast to (void*) to silence MSVC's warning
    /// about const qualifier mismatch (C4090).
    fn release_call(name: &str, c_type: &str) -> String {
        match c_type {
            "hsd_list_num" => format!("hsd_arc_release({}.data);", name),
            "const char*" => format!("hsd_arc_release((void*){});", name),
            _ => format!("hsd_arc_release({});", name),
        }
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(Vec::new());
    }

    /// Pop the top scope and emit hsd_arc_release calls for its variables.
    /// Variables are released in reverse declaration order (LIFO).
    fn leave_scope(&mut self) {
        if let Some(scope) = self.scope_stack.pop() {
            self.emit_releases(&scope);
        }
    }

    /// Emit release calls without modifying the scope stack.
    fn emit_releases(&mut self, scope: &[(String, String)]) {
        for (name, c_type) in scope.iter().rev() {
            let call = Self::release_call(name, c_type);
            self.line(&call);
        }
    }

    /// Emit releases for ALL scopes, from innermost outward.
    /// Used before return statements to clean up before transferring
    /// control out of the function.
    /// If `skip_var` is Some, that variable is NOT released — it's
    /// the value being returned (ownership transfer).
    fn emit_releases_for_return(&mut self, skip_var: Option<&str>) {
        let scopes: Vec<_> = self.scope_stack.iter().rev().cloned().collect();
        for scope in scopes {
            let filtered: Vec<_> = scope.iter()
                .filter(|(name, _)| Some(name.as_str()) != skip_var)
                .cloned()
                .collect();
            self.emit_releases(&filtered);
        }
    }

    /// Add a variable to the current (innermost) scope if it's heap-tracked.
    /// No-op inside a nativum block or if not heap-tracked.
    fn add_to_current_scope_if_heap(&mut self, name: &str, c_type: &str) {
        if self.in_nativum {
            return;
        }
        if !Self::is_arc_tracked(c_type) {
            return;
        }
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.push((name.to_string(), c_type.to_string()));
        }
    }

    /// Look up a heap variable in the scope stack and return its C type
    /// (used by Phase 6d for reassignment releases).
    fn heap_var_type(&self, name: &str) -> Option<String> {
        for scope in self.scope_stack.iter().rev() {
            for (n, t) in scope.iter().rev() {
                if n == name {
                    return Some(t.clone());
                }
            }
        }
        None
    }

    /// Check whether a block ends with an explicit return.
    fn block_ends_with_return(block: &[Stmt]) -> bool {
        matches!(block.last(), Some(Stmt::Return(_)))
    }

    // -------- function signatures --------

    fn func_signature(&self, f: &Function) -> Result<String, String> {
        let ret = match &f.return_type {
            Some(t) => c_type(t)?,
            None => "void".to_string(),
        };
        let mut params = Vec::new();
        for p in &f.params {
            params.push(format!("{} {}", c_type(&p.ty)?, p.name));
        }
        let params = if params.is_empty() {
            "void".to_string()
        } else {
            params.join(", ")
        };
        Ok(format!("{} {}({})", ret, f.name, params))
    }

    fn gen_function(&mut self, f: &Function) -> Result<(), String> {
        self.locals.clear();
        self.scope_stack.clear();
        self.enter_scope();

        for p in &f.params {
            let pty = c_type(&p.ty)?;
            self.locals.insert(p.name.clone(), pty.clone());
            // NOTE: parameters are borrowed from caller — no ARC tracking,
            // no release at scope exit.
        }
        let sig = self.func_signature(f)?;
        self.out.push_str(&sig);
        self.out.push_str(" {\n");
        self.indent = 1;
        self.gen_block(&f.body)?;

        if !Self::block_ends_with_return(&f.body) {
            self.leave_scope();
        } else {
            self.scope_stack.pop();
        }

        self.indent = 0;
        self.out.push_str("}\n");
        Ok(())
    }

    // -------- actors --------

    fn gen_actor_struct(&mut self, a: &ActorDef) -> Result<(), String> {
        self.out.push_str(&format!("struct {} {{\n", a.name));
        for stmt in &a.state {
            if let Stmt::Let { name, ty, value, .. } = stmt {
                let ctype = match ty {
                    Some(t) => c_type(t)?,
                    None => self.c_type_of_expr(value)?,
                };
                self.out.push_str(&format!("    {} {};\n", ctype, name));
            }
        }
        self.out.push_str("};\n\n");
        Ok(())
    }

    fn actor_ctor_sig(&self, a: &ActorDef) -> String {
        format!("{}* hsd_crea_{}(void)", a.name, a.name)
    }

    fn actor_handler_sig(&self, actor_name: &str, h: &Handler) -> Result<String, String> {
        let mut params = vec![format!("{}* self", actor_name)];
        for p in &h.params {
            params.push(format!("{} {}", c_type(&p.ty)?, p.name));
        }
        Ok(format!(
            "void {}_handle_{}({})",
            actor_name, h.message, params.join(", ")
        ))
    }

    fn gen_actor_ctor(&mut self, a: &ActorDef) -> Result<(), String> {
        self.locals.clear();
        self.state_vars.clear();
        let sig = self.actor_ctor_sig(a);
        self.out.push_str(&sig);
        self.out.push_str(" {\n");
        self.out.push_str(&format!(
            "    {0}* self = ({0}*)hsd_arc_alloc(sizeof({0}));\n", a.name
        ));
        self.indent = 1;
        for stmt in &a.state {
            if let Stmt::Let { name, value, .. } = stmt {
                let v = self.gen_expr(value, false)?;
                self.line(&format!("self->{} = {};", name, v));
            }
        }
        self.indent = 0;
        self.out.push_str("    return self;\n}\n\n");
        Ok(())
    }

    fn gen_actor_handler(
        &mut self,
        actor_name: &str,
        a: &ActorDef,
        h: &Handler,
    ) -> Result<(), String> {
        self.locals.clear();
        self.state_vars.clear();
        self.scope_stack.clear();
        self.enter_scope();

        for stmt in &a.state {
            if let Stmt::Let { name, ty, value, .. } = stmt {
                let ctype = match ty {
                    Some(t) => c_type(t)?,
                    None => self.c_type_of_expr(value)?,
                };
                self.state_vars.insert(name.clone(), ctype);
            }
        }
        for p in &h.params {
            self.locals.insert(p.name.clone(), c_type(&p.ty)?);
        }
        let prev = self.current_actor.take();
        self.current_actor = Some(actor_name.to_string());

        let sig = self.actor_handler_sig(actor_name, h)?;
        self.out.push_str(&sig);
        self.out.push_str(" {\n");
        self.indent = 1;
        self.gen_block(&h.body)?;

        if !Self::block_ends_with_return(&h.body) {
            self.leave_scope();
        } else {
            self.scope_stack.pop();
        }

        self.indent = 0;
        self.out.push_str("}\n\n");

        self.current_actor = prev;
        self.state_vars.clear();
        Ok(())
    }

    // -------- statements --------

    fn gen_block(&mut self, block: &[Stmt]) -> Result<(), String> {
        for stmt in block {
            self.gen_stmt(stmt)?;
        }
        Ok(())
    }

    fn line(&mut self, s: &str) {
        let pad = "    ".repeat(self.indent);
        self.out.push_str(&pad);
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn gen_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, ty, value, .. } => {
                let ctype = match ty {
                    Some(t) => c_type(t)?,
                    None => self.c_type_of_expr(value)?,
                };
                self.locals.insert(name.clone(), ctype.clone());
                // Value goes to "owned" context — string literals get wrapped.
                let v = self.gen_expr(value, false)?;
                self.line(&format!("{} {} = {};", ctype, name, v));
                self.add_to_current_scope_if_heap(name, &ctype);
            }
            Stmt::Assign { target, value } => {
                // Phase 6d: if the target is a heap-tracked variable, release
                // the old value before assigning the new one.
                let old_release = match target {
                    Expr::Ident(name) if !self.in_nativum => {
                        self.heap_var_type(name)
                            .map(|t| Self::release_call(name, &t))
                    }
                    _ => None,
                };
                if let Some(call) = old_release {
                    self.line(&call);
                }
                let t = self.gen_expr(target, false)?;
                let v = self.gen_expr(value, false)?;
                self.line(&format!("{} = {};", t, v));
            }
            Stmt::Return(opt) => match opt {
                Some(e) => {
                    let returned_var = if let Expr::Ident(n) = e {
                        Some(n.clone())
                    } else {
                        None
                    };
                    self.emit_releases_for_return(returned_var.as_deref());
                    let v = self.gen_expr(e, false)?;
                    self.line(&format!("return {};", v));
                }
                None => {
                    self.emit_releases_for_return(None);
                    self.line("return;");
                }
            },
            Stmt::If { cond, then_block, elif, else_block } => {
                let c = self.gen_expr(cond, true)?;
                self.line(&format!("if ({}) {{", c));
                self.indent += 1;
                self.enter_scope();
                self.gen_block(then_block)?;
                if !Self::block_ends_with_return(then_block) {
                    self.leave_scope();
                } else {
                    self.scope_stack.pop();
                }
                self.indent -= 1;
                for (ec, eb) in elif {
                    let ecs = self.gen_expr(ec, true)?;
                    self.line(&format!("}} else if ({}) {{", ecs));
                    self.indent += 1;
                    self.enter_scope();
                    self.gen_block(eb)?;
                    if !Self::block_ends_with_return(eb) {
                        self.leave_scope();
                    } else {
                        self.scope_stack.pop();
                    }
                    self.indent -= 1;
                }
                if let Some(eb) = else_block {
                    self.line("} else {");
                    self.indent += 1;
                    self.enter_scope();
                    self.gen_block(eb)?;
                    if !Self::block_ends_with_return(eb) {
                        self.leave_scope();
                    } else {
                        self.scope_stack.pop();
                    }
                    self.indent -= 1;
                }
                self.line("}");
            }
            Stmt::While { cond, body } => {
                let c = self.gen_expr(cond, true)?;
                self.line(&format!("while ({}) {{", c));
                self.indent += 1;
                self.enter_scope();
                self.gen_block(body)?;
                self.leave_scope();
                self.indent -= 1;
                self.line("}");
            }
            Stmt::Break => self.line("break;"),
            Stmt::Continue => self.line("continue;"),
            Stmt::Nativum(body) => {
                let was_nativum = self.in_nativum;
                self.in_nativum = true;
                self.gen_block(body)?;
                self.in_nativum = was_nativum;
            }
            Stmt::Expr(e) => {
                if let Expr::Call { callee, args } = e {
                    if let Expr::Ident(n) = callee.as_ref() {
                        if n == "scribe" {
                            let call = self.gen_scribe(args)?;
                            self.line(&call);
                            return Ok(());
                        }
                    }
                }
                let v = self.gen_expr(e, true)?;
                self.line(&format!("{};", v));
            }
            Stmt::For { var, iter, body } => {
                let iter_code = self.gen_expr(iter, false)?;
                let borrowed_from_ident = matches!(iter, Expr::Ident(_));

                let n = self.tmp_counter;
                self.tmp_counter += 1;
                let tmp_list = format!("_hsd_list_{}", n);
                let tmp_idx = format!("_hsd_idx_{}", n);

                self.line(&format!("hsd_list_num {} = {};", tmp_list, iter_code));
                if borrowed_from_ident && !self.in_nativum {
                    self.line(&format!("hsd_arc_retain({}.data);", tmp_list));
                }

                self.line(&format!(
                    "for (long {} = 0; {} < {}.len; {}++) {{",
                    tmp_idx, tmp_idx, tmp_list, tmp_idx
                ));
                self.indent += 1;
                self.enter_scope();
                self.locals.insert(var.clone(), "long".into());
                self.line(&format!("long {} = {}.data[{}];", var, tmp_list, tmp_idx));
                self.gen_block(body)?;
                self.leave_scope();
                self.indent -= 1;
                self.line("}");

                if !self.in_nativum {
                    self.line(&format!("hsd_arc_release({}.data);", tmp_list));
                }
            }
            Stmt::Send { message, target } => {
                let target_type = self.c_type_of_expr(target)?;
                let actor_name = target_type.trim_end_matches('*').to_string();
                let target_code = self.gen_expr(target, true)?;

                let (msg_name, arg_codes) = match message {
                    Expr::Ident(n) => (n.clone(), Vec::new()),
                    Expr::Call { callee, args } => {
                        let n = match callee.as_ref() {
                            Expr::Ident(n) => n.clone(),
                            _ => return Err("a message must be a nuntius name".into()),
                        };
                        let mut vals = Vec::new();
                        for a in args {
                            vals.push(self.gen_expr(a, true)?);
                        }
                        (n, vals)
                    }
                    _ => return Err("a message must be a nuntius name".into()),
                };

                let mut all_args = vec![target_code];
                all_args.extend(arg_codes);
                self.line(&format!(
                    "{}_handle_{}({});",
                    actor_name, msg_name, all_args.join(", ")
                ));
            }
        }
        Ok(())
    }

    fn gen_scribe(&self, args: &[Expr]) -> Result<String, String> {
        // Inside scribe, all arguments are passed to printf — they're
        // not stored. String literals as printf args don't need ARC
        // wrapping (printf reads them and is done).
        let mut fmt = String::new();
        let mut exprs = Vec::new();
        for a in args {
            let spec = match self.c_type_of_expr(a)?.as_str() {
                "long" => "%ld",
                "double" => "%f",
                "int" => "%d",
                "const char*" => "%s",
                _ => "%ld",
            };
            fmt.push_str(spec);
            exprs.push(self.gen_expr(a, true)?);
        }
        fmt.push_str("\\n");
        if exprs.is_empty() {
            Ok(format!("printf(\"{}\");", fmt))
        } else {
            Ok(format!("printf(\"{}\", {});", fmt, exprs.join(", ")))
        }
    }

    /// Generate code for an expression.
    /// `as_arg` is true when the expression is being used as a function
    /// argument that won't be stored — in that case, string literals
    /// don't need to be wrapped in hsd_arc_copy_str. When false (the
    /// value is being stored in a variable), string literals get wrapped.
    fn gen_expr(&self, expr: &Expr, as_arg: bool) -> Result<String, String> {
        match expr {
            Expr::IntLit(n) => Ok(n.to_string()),
            Expr::FloatLit(f) => Ok(format!("{:?}", f)),
            Expr::BoolLit(b) => Ok(if *b { "1".into() } else { "0".into() }),
            Expr::StrLit(s) => {
                // Phase 6c: when a string literal will be stored in a
                // variable, wrap it in hsd_arc_copy_str so the resulting
                // const char* is ARC-tracked. When it's passed as an
                // argument (printf, etc.), leave it as a plain literal.
                if as_arg || self.in_nativum {
                    Ok(format!("\"{}\"", escape_c(s)))
                } else {
                    Ok(format!("hsd_arc_copy_str(\"{}\")", escape_c(s)))
                }
            }
            Expr::Nihil => Ok("0".into()),
            Expr::Ident(name) => {
                if self.state_vars.contains_key(name) {
                    Ok(format!("self->{}", name))
                } else {
                    Ok(name.clone())
                }
            }
            Expr::Ipse => Ok("self".into()),
            Expr::Unary { op, operand } => {
                let v = self.gen_expr(operand, true)?;
                match op {
                    UnOp::Neg => Ok(format!("(-{})", v)),
                    UnOp::Not => Ok(format!("(!{})", v)),
                }
            }
            Expr::Binary { op, left, right } => {
                let l = self.gen_expr(left, true)?;
                let r = self.gen_expr(right, true)?;
                Ok(format!("({} {} {})", l, c_binop(op), r))
            }
            Expr::Call { callee, args } => {
                if let Expr::Ident(name) = callee.as_ref() {
                    if name == "lege" {
                        let arg = if args.is_empty() {
                            "NULL".to_string()
                        } else {
                            // The prompt argument is passed to printf-like
                            // output inside hsd_lege — use as_arg=true.
                            self.gen_expr(&args[0], true)?
                        };
                        return Ok(format!("hsd_lege({})", arg));
                    }
                    if name == "numerus_ex" {
                        if args.len() != 1 {
                            return Err("numerus_ex expects exactly 1 argument".into());
                        }
                        let a = self.gen_expr(&args[0], true)?;
                        return Ok(format!("hsd_numerus_ex({})", a));
                    }
                    if name == "realis_ex" {
                        if args.len() != 1 {
                            return Err("realis_ex expects exactly 1 argument".into());
                        }
                        let a = self.gen_expr(&args[0], true)?;
                        return Ok(format!("hsd_realis_ex({})", a));
                    }
                    if name == "numera" {
                        if args.len() != 2 {
                            return Err("numera expects exactly 2 arguments".into());
                        }
                        let a = self.gen_expr(&args[0], true)?;
                        let b = self.gen_expr(&args[1], true)?;
                        return Ok(format!("hsd_numera({}, {})", a, b));
                    }
                    if name == "scribe" {
                        return Err("'scribe' cannot be used as an expression here".into());
                    }
                    let mut a = Vec::new();
                    for arg in args {
                        // User function calls — assume args are borrowed,
                        // so string literals don't need to be ARC-wrapped.
                        a.push(self.gen_expr(arg, true)?);
                    }
                    Ok(format!("{}({})", name, a.join(", ")))
                } else {
                    Err("only named function calls are supported".into())
                }
            }
            Expr::Create { name, args } => {
                if !args.is_empty() {
                    return Err(
                        "'crea' does not take arguments; state is set by 'sit' fields".into(),
                    );
                }
                Ok(format!("hsd_crea_{}()", name))
            }
            Expr::Index { object, index } => {
                let o = self.gen_expr(object, true)?;
                let i = self.gen_expr(index, true)?;
                Ok(format!("{}.data[{}]", o, i))
            }
            Expr::Field { .. } | Expr::List(_) => {
                Err("this construct is not supported by the C backend yet".into())
            }
        }
    }

    fn c_type_of_expr(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::IntLit(_) => Ok("long".into()),
            Expr::FloatLit(_) => Ok("double".into()),
            Expr::BoolLit(_) => Ok("int".into()),
            Expr::StrLit(_) => Ok("const char*".into()),
            Expr::Ident(n) => {
                if let Some(t) = self.state_vars.get(n) {
                    return Ok(t.clone());
                }
                Ok(self.locals.get(n).cloned().unwrap_or_else(|| "long".into()))
            }
            Expr::Ipse => {
                if let Some(name) = &self.current_actor {
                    Ok(format!("{}*", name))
                } else {
                    Ok("void*".into())
                }
            }
            Expr::Unary { op, operand } => match op {
                UnOp::Not => Ok("int".into()),
                UnOp::Neg => self.c_type_of_expr(operand),
            },
            Expr::Binary { op, left, .. } => {
                use BinOp::*;
                match op {
                    Lt | Gt | Le | Ge | Eq | Neq | And | Or => Ok("int".into()),
                    _ => self.c_type_of_expr(left),
                }
            }
            Expr::Call { callee, .. } => {
                if let Expr::Ident(n) = callee.as_ref() {
                    match n.as_str() {
                        "lege" => return Ok("const char*".into()),
                        "numerus_ex" => return Ok("long".into()),
                        "realis_ex" => return Ok("double".into()),
                        "numera" => return Ok("hsd_list_num".into()),
                        _ => {}
                    }
                    Ok(self.func_ret.get(n).cloned().unwrap_or_else(|| "long".into()))
                } else {
                    Ok("long".into())
                }
            }
            Expr::Create { name, .. } => Ok(format!("{}*", name)),
            _ => Ok("long".into()),
        }
    }
}

fn c_type(t: &Type) -> Result<String, String> {
    match t {
        Type::Named(n) => match n.as_str() {
            "numerus" => Ok("long".into()),
            "realis" => Ok("double".into()),
            "veritas" => Ok("int".into()),
            "nihil" => Ok("void".into()),
            "verba" => Ok("const char*".into()),
            other => Ok(format!("{}*", other)),
        },
        Type::Generic(name, inner) => {
            if name == "series" {
                let inner_c = c_type(inner)?;
                if inner_c == "long" {
                    Ok("hsd_list_num".into())
                } else {
                    Err(format!(
                        "the C backend supports only series[numerus] for now, got series[{}]",
                        inner_c
                    ))
                }
            } else {
                Err(format!("unknown generic type '{}'", name))
            }
        }
    }
}

fn c_binop(op: &BinOp) -> &'static str {
    use BinOp::*;
    match op {
        Add => "+", Sub => "-", Mul => "*", Div => "/", Mod => "%",
        Lt => "<", Gt => ">", Le => "<=", Ge => ">=", Eq => "==", Neq => "!=",
        And => "&&", Or => "||",
    }
}

fn escape_c(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}
