/* Text OT!
 *
 * This is an OT implementation for plain text editing. It is a rust port of the standard
 * implementation of text used by ShareJS: https://github.com/ottypes/text
 *
 * Ops are lists of components which walk over the document.
 * Operations are made out of skips, inserts and deletes.
 *
 * The operation does not have to skip the last characters in the document.
 *
 * The apply() function requires text to implement the EditableText trait, which provides hooks for
 * efficiently inserting and deleting characters at utf8 character offset positions. An
 * implementation has been provided for String, though I would like to port an efficient skip
 * list based implementation.
 *
 * Cursors and cursor transformation hasn't been implemented yet.
 */

use std::borrow::Cow;

mod editablestring;
use editablestring::EditableText;

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

    fn split(self, offset: usize) -> (OpComponent<'a>, Option<OpComponent<'a>>) {
        assert!(offset > 0);
        let len = self.len();
        if len <= offset { return (self, None) }

        let offset = std::cmp::min(offset, len);
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

    /*
    fn shallow_clone(&'a self) -> OpComponent<'a> {
        match *self {
            Ins(Cow::Borrowed(s)) => Ins(Cow::Borrowed(s)),
            Ins(Cow::Owned(ref s)) => Ins(Cow::Borrowed(&s)),
            _ => self.clone(),
        }
    }*/

    fn is_insert(&self) -> bool {
        match *self { Ins(_) => true, _ => false }
    }
    fn is_del(&self) -> bool {
        match *self { Del(_) => true, _ => false }
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
    contents: std::vec::IntoIter<OpComponent<'a>>,
}


impl<'a> OpIter<'a> {
    fn new(op: Op<'a>) -> OpIter<'a> {
        let mut iter = OpIter {
            _next: None,
            contents: op.into_iter(),
        };
        iter.populate();
        iter
    }

    fn populate(&mut self) {
        if self._next.is_none() {
            self._next = self.contents.next();
        }
    }
    
    fn peek(&self) -> Option<&OpComponent<'a>> {
        self._next.as_ref()
    }

    fn take_whole(&mut self) -> Option<OpComponent<'a>> {
        self._next.take().map(|c| { self.populate(); c })
    }

    fn _take_indivis<F>(&mut self, size: usize, is_indivis: F) -> OpComponent<'a>
            where F: FnOnce(&OpComponent<'a>) -> bool {
        // We're at the end of the operation. The op has skips, forever. Infinity might make more
        // sense than null here.
        self._next.take().map_or(Skip(size), |c| {
            let result = if is_indivis(&c) { c }
            else {
                let (a, b) = c.split(size);
                self._next = b;
                a
            };

            self.populate();
            result
        })
    }

    // Take inserts whole. Other ops get split based on split size.
    fn take_ins(&mut self, size: usize) -> OpComponent<'a> {
        self._take_indivis(size, |c| match *c { Ins(_) => true, _ => false})
    }

    // Take deletes whole. Other ops get split based on split size.
    fn take_del(&mut self, size: usize) -> OpComponent<'a> {
        self._take_indivis(size, |c| match *c { Del(_) => true, _ => false})
    }

    fn append_rest(&mut self, op: &mut Op<'a>) {
        // Append any extra ops directly.
        while let Some(chunk) = self.take_whole() {
            append_op(op, chunk);
        }
    }
}

fn trim(op: &mut Op) {
    // Throw away anything at the back that isn't an insert.
    while op.last().map_or(false, |last| match last { &Skip(_) => true, _ => false }) {
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

pub fn text_apply<S: EditableText>(s: &mut S, op: &Op) {
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
}

#[derive(Debug, PartialEq)]
pub enum Side { Left, Right }

pub fn text_transform<'a>(op: Op<'a>, other_op: &Op, side: Side) -> Op<'a> {
    let mut result = Op::new();
    let mut iter = OpIter::new(op);

    for c in other_op {
        match c {
            &Skip(mut length) => {
                while length > 0 {
                    // Copy up to length components across.
                    let chunk = iter.take_ins(length);
                    if !chunk.is_insert() {
                        length -= chunk.len();
                    }
                    append_op(&mut result, chunk);
                }
            },

            &Ins(ref s) => {
                if side == Side::Left {
                    // The left insert should go first, if any.
                    if let Some(&Ins(_)) = iter.peek() {
                        append_op(&mut result, iter.take_whole().unwrap());
                    }
                }

                // Otherwise skip the foreign inserted text.
                append_op(&mut result, Skip(s.chars().count()));
            }

            &Del(mut length) => {
                while length > 0 {
                    let chunk = iter.take_ins(length);
                    match chunk {
                        Skip(n) => length -= n,
                        Ins(s) => append_op(&mut result, Ins(s)),
                        Del(n) => length -= n, // Delete is unnecessary now - text has been deleted by the other op.
                    }
                }
            }
        }
    }

    iter.append_rest(&mut result);
    trim(&mut result);

    result
}

// I could 'easily' write this to not consume the two operations.. O_o - not sure if its worth the
// memory cloning. Dunno!
pub fn text_compose<'a>(op1: Op<'a>, op2: Op<'a>) -> Op<'a> {
    let mut result = Op::new();
    let mut iter = OpIter::new(op1);

    for c in op2.into_iter() {
        match c {
            Skip(mut length) => {
                // Copy length from op1.
                while length > 0 {
                    let chunk = iter.take_del(length);
                    if !chunk.is_del() {
                        length -= chunk.len();
                    }
                    append_op(&mut result, chunk);
                }
            },

            Ins(_) => append_op(&mut result, c),

            Del(mut length) => {
                while length > 0 {
                    let chunk = iter.take_del(length);
                    match chunk {
                        Skip(n) => {
                            append_op(&mut result, Del(n));
                            length -= n;
                        },
                        Ins(s) => length -= s.chars().count(),
                        Del(_) => append_op(&mut result, chunk),
                    }
                }
            }
        }
    }

    iter.append_rest(&mut result);
    trim(&mut result);
    result
}

#[cfg(test)]
mod tests {
    extern crate rustc_serialize;

    use std::borrow::Cow;
    use super::*;
    use OpComponent::*;

    fn ins(s: &str) -> OpComponent {
        Ins(Cow::Borrowed(s))
    }

    fn json_str_to_op<'a>(s: &str) -> Op<'a> {
        use self::rustc_serialize::json::Json;

        let json = Json::from_str(s).unwrap();

        normalize(json.as_array().unwrap().into_iter().map(|item| {
            match item {
                &Json::U64(s) => Skip(s as usize),
                &Json::Object(ref d) => {
                    d.get("i").map(|s| Ins(Cow::Owned(s.as_string().unwrap().to_string())))
                        .or_else(|| d.get("d").map(|d| Del(d.as_string().unwrap().chars().count())))
                        .unwrap()
                },
                _ => panic!("Invalid JSON")
            }
        }).collect())
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

        assert_eq!(vec![Del(2)], normalize(vec![Del(2)]));
        assert_eq!(vec![ins("a"), Skip(1), Del(1), ins("b")], normalize(vec![ins("a"), Skip(1), Del(1), ins("b")]));
    }

    #[test]
    fn transform_conformation() {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open("text-transform-tests.json").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        
        let mut lines = s.lines();
        loop {
            let a = match lines.next() { None => break, Some(a) => a };
            let b = lines.next().unwrap();
            let side = if lines.next().unwrap() == "left" { Side::Left } else { Side::Right };
            let expected = lines.next().unwrap();

            let a = json_str_to_op(a);
            let b = json_str_to_op(b);
            let expected = json_str_to_op(expected);

            //println!("{:?} x {:?} ({:?}) expecting {:?}", a, b, side, expected);
            let result = text_transform(a, &b, side);
            //println!("got {:?}", result);
            assert!(result == expected);
        }
    }

    #[test]
    fn compose() {
        assert_eq!(vec![ins("ab")], text_compose(vec![ins("a")], vec![Skip(1), ins("b")]));
    }

    #[test]
    fn apply() {
        let mut s = String::new();
        text_apply(&mut s, &vec![ins("hi")]);
        assert_eq!(s, "hi");
        text_apply(&mut s, &vec![Skip(1), ins("a")]);
        assert_eq!(s, "hai");

    }
}
