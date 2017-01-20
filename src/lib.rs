// TODO: Remove me.
#![allow(unused_variables)]

use std::borrow::Cow;
use std::{result, error, fmt};
use std::cmp::min;

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OpComponent<'a> {
    Skip(usize),
    Del(usize),
    Ins(Cow<'a, str>),
}

use self::OpComponent::*;

impl<'a> OpComponent<'a> {
    fn len(&self) -> usize {
        match *self { Skip(len) => len, Del(len) => len, Ins(ref s) => s.chars().count() }
    }

    fn is_empty(&self) -> bool {
        // Does it make sense to *ever* have empty op components? Maybe they should be illegal to
        // express.
        match *self { Ins(ref s) => s.is_empty(), _ => self.len() == 0 }
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

    fn split(self, offset: usize) -> (OpComponent<'a>, Option<OpComponent<'a>>) {
        assert!(offset > 0);
        let len = self.len();
        if len <= offset { return (self, None) }

        let offset = min(offset, len);
        match self {
            Skip(s) => (Skip(offset), Some(Skip(s - offset))),
            Del(s) => (Del(offset), Some(Del(s - offset))),
            Ins(cow) => {
                // Need the byte offset either way.
                let byte_offset = cow.char_indices().skip(offset).next().unwrap().0;
                match cow {
                    Cow::Borrowed(s) => {
                        let (a, b) = s.split_at(byte_offset);
                        (Ins(Cow::Borrowed(a)), Some(Ins(Cow::Borrowed(b))))
                    },
                    Cow::Owned(mut s) => {
                        // Its sad I have to allocate a new string here. I wonder if it would
                        // make the algorithm faster to make a Cow::Borrowed clone set and
                        // compose/transform/etc with those instead of the originals.
                        let b = s[offset..].to_string();
                        s.truncate(byte_offset);
                        (Ins(Cow::Owned(s)), Some(Ins(Cow::Owned(b))))
                    }
                }
            }
        }
    }

    fn shallow_clone(&'a self) -> OpComponent<'a> {
        match *self {
            Ins(Cow::Borrowed(s)) => Ins(Cow::Borrowed(s)),
            Ins(Cow::Owned(ref s)) => Ins(Cow::Borrowed(&s)),
            _ => self.clone(),
        }
    }
}

pub type Op<'a> = Vec<OpComponent<'a>>;

fn append_op<'a>(op: &mut Op<'a>, c: OpComponent<'a>) {
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

struct OpIter<'a> {
    // Always populated.
    _next: Option<OpComponent<'a>>,
    contents: std::slice::Iter<'a, OpComponent<'a>>,
}


impl<'a> OpIter<'a> {
    fn new(op: &'a Op) -> OpIter<'a> {
        let mut iter = OpIter {
            _next: None,
            contents: op.iter(),
        };
        iter.populate();
        iter
    }

    fn populate(&mut self) {
        if self._next.is_none() {
            self._next = self.contents.next().map(|c| c.shallow_clone());
        }
    }

    /*
    fn peek(&'a mut self) -> Option<&OpComponent<'a>> {
        self._next.as_ref()
    }*/

    fn take_whole(&mut self) -> Option<OpComponent<'a>> {
        self._next.take().map(|c| { self.populate(); c })
    }

    fn _take_indivis<F>(&mut self, size: usize, is_indivis: F) -> Option<OpComponent<'a>>
            where F: FnOnce(&OpComponent<'a>) -> bool {
        self._next.take().map(|c| {
            if is_indivis(&c) { self.populate(); c }
            else {
                let (a, b) = c.split(size);
                self._next = b;
                self.populate();
                a
            }
        })
    }

    // Take inserts whole. Other ops get split based on split size.
    fn take_ins(&mut self, size: usize) -> Option<OpComponent<'a>> {
        self._take_indivis(size, |c| match *c { Ins(_) => true, _ => false})
    }

    // Take deletes whole. Other ops get split based on split size.
    fn take_del(&mut self, size: usize) -> Option<OpComponent<'a>> {
        self._take_indivis(size, |c| match *c { Del(_) => true, _ => false})
    }
}



fn trim(op: &mut Op) {
    // Throw away anything at the back that isn't an insert.
    while op.last().map_or(false, |last| match last { &Ins(_) => false, _ => true }) {
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

/*
pub enum Side { Left, Right }
pub fn text_transform<'a>(op: Op, other_op: &Op, side: Side) -> Op<'a> {
    let result = Op::new();
    let iter = OpIter::new(&op);

    for c in other_op {
        match c {
            Skip(mut s) => {
                while s > 0 {
                    // Copy components across.

                }
            },


        }
    }

    result
}
*/

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
