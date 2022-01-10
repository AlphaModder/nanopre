#![forbid(unsafe_code)]

use std::{collections::HashMap, ops::Deref, io::BufRead};

mod expr;

fn is_define_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

type Macros = HashMap<String, String>;

fn substitute(macros: &Macros, s: &str) -> String {
    let boundaries = s.char_indices().scan(None, |last_char, (start, char)| {
        let result = match *last_char {
            Some(last_char) if is_define_char(last_char) != is_define_char(char) => Some(start),
            _ => None,
        };
        *last_char = Some(char);
        Some(result) // yes this is Option<Option<usize>>
    }).flatten().chain(Some(s.len()));

    let parts = boundaries.scan(0, move |start, end| {
        let part = &s[*start..end];
        *start = end;
        Some(part)
    });

    parts.map(|part| macros.get(part).map(Deref::deref).unwrap_or(part)).collect()
}

#[derive(Clone)]
pub struct Context {
    macros: Macros,
}

impl Context {
    pub fn new() -> Context { Context { macros: Macros::new() } }

    pub fn define(&mut self, def: impl Into<String>, val: impl Into<String>) {
        fn is_define_name(s: &str) -> bool {
            let mut chars = s.chars();
            chars.next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false) && chars.all(is_define_char)
        }

        let (def, val) = (def.into(), val.into());
        assert!(is_define_name(&def));
        self.macros.insert(def, val);
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    BadExpr(&'static str),
    UnexpectedDirective(&'static str),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self { Error::Io(error) }
}

pub fn process_buf<B: BufRead>(b: &mut B, ctx: &mut Context) -> Result<String, Error> {
    #[derive(Copy, Clone)]
    enum FoundBranch { NotYet, Now, Already }

    #[derive(Copy, Clone)]
    enum ParseItem {
        If(FoundBranch),
        Else { active: bool },
        InactiveIf,
    }
    use ParseItem::*;

    fn is_active(val: Option<&ParseItem>) -> bool {
        match val {
            None | Some(If(FoundBranch::Now)) => true,
            Some(InactiveIf | If(FoundBranch::NotYet | FoundBranch::Already)) => false,
            Some(Else { active }) => *active,
        }
    }

    let mut output = String::new();
    let mut stack = Vec::new();
    let mut line = String::new();
    while b.read_line(&mut line)? > 0 {
        match line.split("//").next().unwrap().trim() {
            cmd if cmd.starts_with("#if ") => stack.push(
                match is_active(stack.last()) {
                    true => If(match expr::eval(&substitute(&ctx.macros, &cmd[4..]))? {
                        true => FoundBranch::Now,
                        false => FoundBranch::NotYet,
                    }),
                    false => InactiveIf
                }
            ),
            cmd if cmd.starts_with("#elseif ") => match stack.last_mut() {
                Some(If(found_branch @ FoundBranch::NotYet)) => if expr::eval(&substitute(&ctx.macros, &cmd[8..]))? {
                    *found_branch = FoundBranch::Now;
                },
                Some(If(found_branch @ (FoundBranch::Now | FoundBranch::Already))) => *found_branch = FoundBranch::Already,
                Some(InactiveIf) => {},
                _ => return Err(Error::UnexpectedDirective("unexpected #elseif"))
            },
            "#else" => match stack.last_mut() {
                Some(prev @ If(FoundBranch::Now | FoundBranch::Already)) => *prev = Else { active: false },
                Some(prev @ If(FoundBranch::NotYet)) => *prev = Else { active: true },
                Some(InactiveIf) => {}, // TODO: InactiveElse?
                _ => return Err(Error::UnexpectedDirective("unexpected #else"))
            },
            "#endif" => if let None = stack.pop() { return Err(Error::UnexpectedDirective("unexpected #endif")) },
            _ => if is_active(stack.last()) { output += &substitute(&ctx.macros, &line); }
        }
        line.clear();
    }

    Ok(output)
}

pub fn process_str(s: &str, ctx: &mut Context) -> Result<String, Error> { process_buf(&mut s.as_bytes(), ctx) }

#[test]
fn basic_substitution() {
    let mut ctx = Context::new();
    ctx.define("DEF1", "Hello");
    ctx.define("DEF2", "World");
 
    assert_eq!(&substitute(&mut ctx.macros, "DEF1 DEF2!"), "Hello World!");
    assert_eq!(&substitute(&mut ctx.macros, "Hello DEF2s!"), "Hello DEF2s!");
    assert_eq!(&substitute(&mut ctx.macros, "0DEF1"), "0DEF1");
}

#[test]
fn preprocess() {
    macro_rules! assert_process {
        ($ctx:expr, $s:expr, $result:pat) => {
            let val = process_str($s, $ctx);
            assert!(matches!(val.as_deref(), $result));
        }
    } 

    let mut ctx = Context::new();
    ctx.define("_TRUE", "1");
    ctx.define("_FALSE", "0");
    ctx.define("_OR", "||");
    ctx.define("_AND", "&&");

    assert_process!(&mut ctx, "#if _TRUE _OR _FALSE\nyes\n#else\nno\n#endif", Ok("yes\n"));
    assert_process!(&mut ctx, "#if 0\nstuff\n#endif", Ok(""));
    assert_process!(&mut ctx, "#if 1\n#if 0\n#if 1\nApple\n#endif\n#elseif 0\nBanana\n#else\nOrange\n#endif\n#endif", Ok("Orange\n"));
    assert_process!(&mut ctx, "#if _FALSE _AND _TRUE\nGoodbye\n#elseif _TRUE\nHello\n#else \nThe\n#endif\nWorld!", Ok("Hello\nWorld!"));
    assert_process!(&mut ctx, "#version 140", Ok("#version 140"));

    assert_process!(&mut ctx, "#if 1\nstuff\n#endif\n#endif", Err(Error::UnexpectedDirective(_)));
    assert_process!(&mut ctx, "#if 1\nabc\n#else\ndef\n#elseif 1\nghi\n#endif", Err(Error::UnexpectedDirective(_)));
}