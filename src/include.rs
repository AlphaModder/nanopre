use std::{fmt::Debug, io::BufRead};

pub trait Includes {
    type Content: BufRead;
    type Error: Debug;

    fn find_content(&self, path: &str) -> Result<Self::Content, Self::Error>;
}

impl<'a, F, C: BufRead, E: Debug> Includes for F where F: Fn(&str) -> Result<C, E> {
    type Content = C;
    type Error = E;

    fn find_content(&self, path: &str) -> Result<Self::Content, Self::Error> { self(path) }
}

pub struct NoIncludes;
impl Includes for NoIncludes {
    type Content = std::io::Empty;
    type Error = NotSupported;
    fn find_content(&self, _path: &str) -> Result<Self::Content, Self::Error> { Err(NotSupported) }
}

pub struct NotSupported;
impl Debug for NotSupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "#include is not supported") }
}

#[test]
fn include_works() {
    use crate::Context;

    macro_rules! assert_process {
        ($ctx:expr, $s:expr, $result:pat) => {
            let val = crate::process_str($s, $ctx);
            assert!(matches!(dbg!(val.as_deref()), $result));
        }
    } 

    let includes = |s: &str| match dbg!(s) {
        "foo" => Ok("line2".as_bytes()),
        _ => Err(())
    };

    let mut ctx = Context::with_includes(includes);

    assert_process!(&mut ctx, "line1\n#include foo\nline3", Ok("line1\nline2\nline3"));
}
