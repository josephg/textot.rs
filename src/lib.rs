// TODO: Remove me.
#![allow(unused_variables)]

use std::borrow::Cow;
use std::{result, error, fmt};

//use std::convert::From;

mod editablestring;
use editablestring::EditableText;

#[derive(Debug)]
pub enum TextOTError {

}

impl fmt::Display for TextOTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {

        }
    }
}

impl error::Error for TextOTError {
    fn description(&self) -> &str {
        match *self {

        }
    }
}

pub type Result<T> = result::Result<T, TextOTError>;

#[derive(Debug, PartialEq, Eq)]
pub enum OpComponent<'a> {
    Skip(usize),
    Del(usize),
    Ins(Cow<'a, str>),
}

impl<'a> OpComponent<'a> {
    fn is_empty(&self) -> bool {
        use OpComponent::*;
        match *self {
            Skip(len) => len == 0,
            Del(len) => len == 0,
            Ins(ref s) => s.len() == 0,
        }
    }

    /*
    fn same_type(&self, other: &OpComponent) -> bool {
        use OpComponent::*;
        match *self {
            Skip(_) => match *other {Skip(_) => true, _ => false},
            Del(_) => match *other {Del(_) => true, _ => false},
            Ins(_) => match *other {Ins(_) => true, _ => false},
        }
    }*/
}

pub type Op<'a> = Vec<OpComponent<'a>>;

fn append_op<'a>(op: &mut Op<'a>, c: OpComponent<'a>) {
    use OpComponent::*;

    if c.is_empty() { return; } // No-op! Ignore!

    if let Some(last) = op.pop() {
        let (new_last, is_merged) = match last {
            Skip(a) => match c { Skip(b) => (Skip(a+b), true), _ => (Skip(a), false) },
            Del(a) => match c { Del(b) => (Del(a+b), true), _ => (Del(a), false) },
            Ins(a) => match c {
                Ins(ref b) => (Ins(Cow::Owned(a.into_owned()+&b)), true),
                _ => (Ins(a), false)
            },
        };

        op.push(new_last);
        if !is_merged { op.push(c); }
    } else {
        op.push(c);
    }
}



fn trim(op: &mut Op) {
    // Throw away anything at the back that isn't an insert.
    while op.last().map_or(false, |last| match last {
        &OpComponent::Ins(_) => false, _ => true,
    }) {
        op.pop();
    }
}

pub fn normalize(op: Op) -> Op {
    // This is a really lazy way to write this function - it involves a new vector allocation. Much
    // better would be to edit it in-place.
    let mut result = Op::with_capacity(op.capacity());
    for c in op {
        append_op(&mut result, c);
    }

    trim(&mut result);
    result
}

pub fn text_apply<S: EditableText>(s: &mut S, op: &Op) -> Result<()> {
    use OpComponent::*;
    let mut pos = 0;

    for c in op {
        match *c {
            Skip(len) => pos += len,
            Del(len) => s.remove_at(pos, len),
            Ins(ref ins) => {
                s.insert_at(pos, ins);
                pos += ins.chars().count();
            },
        }
    }

    Ok(())
}

pub fn text_compose<'a>(op1: Op, op2: Op) -> Op<'a> {
    let result = Op::new();



    result
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use super::*;
    use OpComponent::*;

    fn ins(s: &str) -> OpComponent {
        Ins(Cow::Borrowed(s))
    }

    #[test]
    fn normalize_works() {
        assert_eq!(Op::new(), normalize(vec![Skip(0)]));
        assert_eq!(Op::new(), normalize(vec![ins("")]));
        assert_eq!(Op::new(), normalize(vec![Del(0)]));

        assert_eq!(Op::new(), normalize(vec![Skip(1), Skip(1)]));
        assert_eq!(Op::new(), normalize(vec![Skip(2), Skip(0)]));
        assert_eq!(vec![Skip(2), ins("hi")], normalize(vec![Skip(1), Skip(1), ins("hi")]));
        assert_eq!(vec![Del(2), ins("hi")], normalize(vec![Del(1), Del(1), ins("hi")]));
        assert_eq!(vec![ins("a")], normalize(vec![ins("a"), Skip(100)]));
        assert_eq!(vec![ins("ab")], normalize(vec![ins("a"), ins("b")]));
        assert_eq!(vec![ins("ab")], normalize(vec![ins("ab"), ins("")]));
        assert_eq!(vec![ins("ab")], normalize(vec![Skip(0), ins("a"), Skip(0), ins("b"), Skip(0)]));

        assert_eq!(vec![ins("a"), Skip(1), Del(1), ins("b")], normalize(vec![ins("a"), Skip(1), Del(1), ins("b")]));
    }

    #[test]
    fn apply() {
        let mut s = String::new();
        text_apply(&mut s, &vec![ins("hi")]).unwrap();
        assert_eq!(s, "hi");
        text_apply(&mut s, &vec![Skip(1), ins("a")]).unwrap();
        assert_eq!(s, "hai");

    }
}
