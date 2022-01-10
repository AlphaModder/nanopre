use crate::Error;

pub enum Token { LeftParen, RightParen, And, Or, Not, Zero, One }
use Token::*;

fn tokenize(mut str: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    loop {
        str = str.trim_start();
        match str {
            "" => return Ok(tokens),
            _ if str.starts_with("||") => { tokens.push(Or); str = &str[2..] }
            _ if str.starts_with("&&") => { tokens.push(And); str = &str[2..] }
            _ if str.starts_with("(") => { tokens.push(LeftParen); str = &str[1..] }
            _ if str.starts_with(")") => { tokens.push(RightParen); str = &str[1..] }
            _ if str.starts_with("!") => { tokens.push(Not); str = &str[1..] }
            _ if str.starts_with("0") => { tokens.push(Zero); str = &str[1..] }
            _ if str.starts_with("1") => { tokens.push(One); str = &str[1..] }
            _ => return Err(Error::BadExpr("unexpected symbol"))
        }
    }
}

fn eval_inner(tokens: &mut &[Token], paren: bool) -> Result<bool, Error> {
    enum Op { Set, And, Or }

    let (mut result, mut op) = (false, Op::Set);
    loop {
        let mut negate = false;
        while let [Not, rest @ ..] = tokens { negate = !negate; *tokens = rest; }
        let val = negate ^ match tokens {
            [One, rest @ ..] => { *tokens = rest; true }
            [Zero, rest @ ..] => { *tokens = rest; false }
            [LeftParen, rest @ ..] => { *tokens = rest; eval_inner(tokens, true)? }
            _ => return Err(Error::BadExpr("unexpected token"))
        };

        match op {
            Op::Set => result = val,
            Op::And => result &= val,
            Op::Or => result |= val,
        }

        op = match tokens {
            [And, rest @ ..] => { *tokens = rest; Op::And }
            [Or, rest @ ..] => { *tokens = rest; Op::Or }
            [RightParen, rest @ ..] if paren => { *tokens = rest; return Ok(result) }
            [] if !paren => return Ok(result),
            _ => return Err(Error::BadExpr("unexpected token"))
        };
    }
}

pub fn eval(expr: &str) -> Result<bool, Error> {
    let mut tokens: &[_] = &tokenize(expr)?;
    eval_inner(&mut tokens, false)
}

#[test]
fn tokenize_tests() {
    assert!(matches!(tokenize("    0 1 && ||  ( ) ! ").as_deref(), Ok(&[Zero, One, And, Or, LeftParen, RightParen, Not])));
    assert!(matches!(tokenize("!)   1&&   0||   (").as_deref(), Ok(&[Not, RightParen, One, And, Zero, Or, LeftParen])));
    assert!(matches!(tokenize("0    && 1 + 1    "), Err(_)));
    assert!(matches!(tokenize("0    && x     || 1"), Err(_)));
}

#[test]
fn eval_tests() {
    let eval = |s| eval(s).map_err(|_| ());
    assert_eq!(eval("0"), Ok(false));
    assert_eq!(eval("1"), Ok(true));
    assert_eq!(eval("1 || 0"), Ok(true));
    assert_eq!(eval("1 && 0"), Ok(false));
    assert_eq!(eval("!0 || 0"), Ok(true));
    assert_eq!(eval("(1 && 1)"), Ok(true));
    assert_eq!(eval("1 || 0 && 1"), eval("(1 || 0) && 1"));
    assert_eq!(eval("!(0 || (1 && 1)) && (!1 || !1 || (0 && 1))"), Ok(false));
    assert_eq!(eval("(1 || 0) && !0 || (1 && (1 && (0))) && 1 || !1"), Ok(true));
}