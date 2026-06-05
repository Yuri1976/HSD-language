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
        for p in &f.params {
            self.locals.insert(p.name.clone(), c_type(&p.ty)?);
        }
        let sig = self.func_signature(f)?;
        self.out.push_str(&sig);
        self.out.push_str(" {\n");
        self.indent = 1;
        self.gen_block(&f.body)?;
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
            "    {0}* self = ({0}*)malloc(sizeof({0}));\n", a.name
        ));
        self.out.push_str(&format!(
            "    if (self == NULL) {{ fprintf(stderr, \"hsd runtime: out of memory in crea {}\\n\"); exit(1); }}\n",
            a.name
        ));
        self.indent = 1;
        for stmt in &a.state {
            if let Stmt::Let { name, value, .. } = stmt {
                let v = self.gen_expr(value)?;
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
                let v = self.gen_expr(value)?;
                self.line(&format!("{} {} = {};", ctype, name, v));
            }
            Stmt::Assign { target, value } => {
                let t = self.gen_expr(target)?;
                let v = self.gen_expr(value)?;
                self.line(&format!("{} = {};", t, v));
            }
            Stmt::Return(opt) => match opt {
                Some(e) => {
                    let v = self.gen_expr(e)?;
                    self.line(&format!("return {};", v));
                }
                None => self.line("return;"),
            },
            Stmt::If { cond, then_block, elif, else_block } => {
                let c = self.gen_expr(cond)?;
                self.line(&format!("if ({}) {{", c));
                self.indent += 1;
                self.gen_block(then_block)?;
                self.indent -= 1;
                for (ec, eb) in elif {
                    let ecs = self.gen_expr(ec)?;
                    self.line(&format!("}} else if ({}) {{", ecs));
                    self.indent += 1;
                    self.gen_block(eb)?;
                    self.indent -= 1;
                }
                if let Some(eb) = else_block {
                    self.line("} else {");
                    self.indent += 1;
                    self.gen_block(eb)?;
                    self.indent -= 1;
                }
                self.line("}");
            }
            Stmt::While { cond, body } => {
                let c = self.gen_expr(cond)?;
                self.line(&format!("while ({}) {{", c));
                self.indent += 1;
                self.gen_block(body)?;
                self.indent -= 1;
                self.line("}");
            }
            Stmt::Break => self.line("break;"),
            Stmt::Continue => self.line("continue;"),
            Stmt::Nativum(body) => self.gen_block(body)?,
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
                let v = self.gen_expr(e)?;
                self.line(&format!("{};", v));
            }
            Stmt::For { var, iter, body } => {
                let iter_code = self.gen_expr(iter)?;
                let n = self.tmp_counter;
                self.tmp_counter += 1;
                let tmp_list = format!("_hsd_list_{}", n);
                let tmp_idx = format!("_hsd_idx_{}", n);
                self.line(&format!("hsd_list_num {} = {};", tmp_list, iter_code));
                self.line(&format!(
                    "for (long {} = 0; {} < {}.len; {}++) {{",
                    tmp_idx, tmp_idx, tmp_list, tmp_idx
                ));
                self.indent += 1;
                self.locals.insert(var.clone(), "long".into());
                self.line(&format!("long {} = {}.data[{}];", var, tmp_list, tmp_idx));
                self.gen_block(body)?;
                self.indent -= 1;
                self.line("}");
            }
            Stmt::Send { message, target } => {
                let target_type = self.c_type_of_expr(target)?;
                let actor_name = target_type.trim_end_matches('*').to_string();
                let target_code = self.gen_expr(target)?;

                let (msg_name, arg_codes) = match message {
                    Expr::Ident(n) => (n.clone(), Vec::new()),
                    Expr::Call { callee, args } => {
                        let n = match callee.as_ref() {
                            Expr::Ident(n) => n.clone(),
                            _ => return Err("a message must be a nuntius name".into()),
                        };
                        let mut vals = Vec::new();
                        for a in args {
                            vals.push(self.gen_expr(a)?);
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
            exprs.push(self.gen_expr(a)?);
        }
        fmt.push_str("\\n");
        if exprs.is_empty() {
            Ok(format!("printf(\"{}\");", fmt))
        } else {
            Ok(format!("printf(\"{}\", {});", fmt, exprs.join(", ")))
        }
    }

    fn gen_expr(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::IntLit(n) => Ok(n.to_string()),
            Expr::FloatLit(f) => Ok(format!("{:?}", f)),
            Expr::BoolLit(b) => Ok(if *b { "1".into() } else { "0".into() }),
            Expr::StrLit(s) => Ok(format!("\"{}\"", escape_c(s))),
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
                let v = self.gen_expr(operand)?;
                match op {
                    UnOp::Neg => Ok(format!("(-{})", v)),
                    UnOp::Not => Ok(format!("(!{})", v)),
                }
            }
            Expr::Binary { op, left, right } => {
                let l = self.gen_expr(left)?;
                let r = self.gen_expr(right)?;
                Ok(format!("({} {} {})", l, c_binop(op), r))
            }
            Expr::Call { callee, args } => {
                if let Expr::Ident(name) = callee.as_ref() {
                    if name == "lege" {
                        let arg = if args.is_empty() {
                            "NULL".to_string()
                        } else {
                            self.gen_expr(&args[0])?
                        };
                        return Ok(format!("hsd_lege({})", arg));
                    }
                    if name == "numerus_ex" {
                        if args.len() != 1 {
                            return Err("numerus_ex expects exactly 1 argument".into());
                        }
                        let a = self.gen_expr(&args[0])?;
                        return Ok(format!("hsd_numerus_ex({})", a));
                    }
                    if name == "realis_ex" {
                        if args.len() != 1 {
                            return Err("realis_ex expects exactly 1 argument".into());
                        }
                        let a = self.gen_expr(&args[0])?;
                        return Ok(format!("hsd_realis_ex({})", a));
                    }
                    if name == "numera" {
                        if args.len() != 2 {
                            return Err("numera expects exactly 2 arguments".into());
                        }
                        let a = self.gen_expr(&args[0])?;
                        let b = self.gen_expr(&args[1])?;
                        return Ok(format!("hsd_numera({}, {})", a, b));
                    }
                    if name == "scribe" {
                        return Err("'scribe' cannot be used as an expression here".into());
                    }
                    let mut a = Vec::new();
                    for arg in args {
                        a.push(self.gen_expr(arg)?);
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
                let o = self.gen_expr(object)?;
                let i = self.gen_expr(index)?;
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