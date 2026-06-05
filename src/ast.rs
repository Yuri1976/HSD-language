// ============================================================
//  HSD — Hic Sunt Dracones 
//
//  This file defines the SHAPE of the tree. It contains no
//  logic: only the data structures the parser will build.
//  The parser (next step) turns a list of tokens into a
//  `Program` made of these nodes.
// ============================================================

// These types are not used yet (the parser doesn't exist),
// so we silence the "unused" warnings for now.
#![allow(dead_code)]

// ---------- The whole program ----------
// A program is just a list of top-level items.

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

// ---------- Top-level items ----------
// The things that can appear at the outermost level of a file.

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Import(String),          // affer "module"
    Function(Function),      // munus name(...) -> type <block>
    Genus(GenusDef),         // genus Name <fields>
    Actor(ActorDef),         // actor Name <fields + handlers>
    Nuntius(NuntiusDef),     // nuntius Name(...)
    Statement(Stmt),         // a statement at top level
}

// ---------- Function ----------

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,   // None if not declared
    pub body: Vec<Stmt>,
}

// A single parameter, e.g. `n: numerus`
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

// ---------- Genus (record / struct) ----------

#[derive(Debug, Clone, PartialEq)]
pub struct GenusDef {
    pub name: String,
    pub fields: Vec<Field>,
}

// A single field, e.g. `value: numerus`
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

// ---------- Actor ----------
// An actor has internal state and message handlers.
// State entries are `sit`/`fixum` declarations (Stmt::Let):
// an actor's state needs initial values.

#[derive(Debug, Clone, PartialEq)]
pub struct ActorDef {
    pub name: String,
    pub state: Vec<Stmt>,        // each entry is a Stmt::Let
    pub handlers: Vec<Handler>,
}

// A message handler: `accipe Message(...) <block>`
#[derive(Debug, Clone, PartialEq)]
pub struct Handler {
    pub message: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

// ---------- Nuntius (message type) ----------

#[derive(Debug, Clone, PartialEq)]
pub struct NuntiusDef {
    pub name: String,
    pub params: Vec<Param>,
}

// ---------- Types ----------
// How a type is written in the source.

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(String),                  // numerus, verba, veritas, ...
    Generic(String, Box<Type>),     // series[numerus]
}

// ---------- Statements ----------
// Things that DO something. A block is a Vec<Stmt>.

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // sit / fixum : a name binding. `mutable` is true for `sit`.
    Let {
        mutable: bool,
        name: String,
        ty: Option<Type>,           // None if the type is inferred
        value: Expr,
    },

    // assignment to an existing target, e.g. `x = x + 1`
    Assign {
        target: Expr,
        value: Expr,
    },

    // si / aliter si / aliter
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        elif: Vec<(Expr, Vec<Stmt>)>,   // zero or more "aliter si"
        else_block: Option<Vec<Stmt>>,  // optional final "aliter"
    },

    // dum : while loop
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },

    // per ... in ... : for loop
    For {
        var: String,
        iter: Expr,
        body: Vec<Stmt>,
    },

    Return(Option<Expr>),   // redde [expr]
    Break,                  // frange
    Continue,               // perge

    // mitte <message> ad <target>
    Send {
        message: Expr,
        target: Expr,
    },

    // nativum : low-level opt-out block
    Nativum(Vec<Stmt>),

    // an expression used as a statement, e.g. a function call
    Expr(Expr),
}

// ---------- Expressions ----------
// Things that PRODUCE a value.
//
// Note the `Box<Expr>`: an expression can contain other
// expressions (recursion). Rust must know a type's size at
// compile time, and a directly self-containing type would be
// infinitely large. `Box` puts the inner expression on the
// heap and stores just a pointer — a fixed, known size.

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // literals
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    BoolLit(bool),       // verum / falsum
    Nihil,               // nihil

    Ident(String),       // a variable or function name
    Ipse,                // ipse : the current actor ("self")

    // binary operation: left <op> right
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    // unary operation: <op> operand
    Unary {
        op: UnOp,
        operand: Box<Expr>,
    },

    // function call: callee(args...)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    // indexing: object[index]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    // field access: object.name
    Field {
        object: Box<Expr>,
        name: String,
    },

    // list literal: [a, b, c]
    List(Vec<Expr>),

    // crea Name(args...) : spawn an actor
    Create {
        name: String,
        args: Vec<Expr>,
    },
}

// ---------- Operators ----------

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,        //  + - * / %
    Eq, Neq, Lt, Gt, Le, Ge,        //  == != < > <= >=
    And, Or,                        //  et  vel
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,    // -  (numeric negation)
    Not,    // non (logical negation)
}
