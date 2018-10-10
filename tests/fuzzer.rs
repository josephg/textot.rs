extern crate json;

extern crate text;

use text::{OpComponent, TextOp, transform};
use std::fs::File;
use std::io::{Lines, BufReader, BufRead, Result};
use json::JsonValue;

struct JsonStreamIter {
    iter: Lines<BufReader<File>>,
}

impl JsonStreamIter {
    fn read_file(filename: &str) -> Result<JsonStreamIter> {
        let f = File::open(filename)?;
        let bufreader = BufReader::new(f);
        Ok(JsonStreamIter { iter: bufreader.lines() })
    }
}

impl Iterator for JsonStreamIter {
    type Item = json::JsonValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|line| {
            let line = line.unwrap();
            if line.len() == 0 { None }
            else { Some(json::parse(&line).unwrap()) }
        })
    }
}

fn json_to_op(val: &JsonValue) -> Option<TextOp> {
    if !val.is_array() { return None }

    let mut op = TextOp::new();
    for m in val.members() {
        use JsonValue::*;
        use text::OpComponent::*;
        op.append_move(match m {
            Short(s) => OpComponent::ins_from(s.as_str()),
            JsonValue::String(s) => OpComponent::ins_from(s.clone()),
            Number(n) => Skip(usize::from(*n)),
            Object(obj) => Del(obj["d"].as_usize().unwrap()),
            _ => { panic!("Invalid data {}", m); }
        });
    }

    Some(op)
}


#[test]
fn test_apply() -> Result<()> {
    for data in JsonStreamIter::read_file("../text/apply.json")? {
        let doc = data["str"].as_str().unwrap();
        let op = json_to_op(&data["op"]).unwrap();
        let expect = data["result"].as_str().unwrap();
        
        // print!("\n\nDOC: '{}'\nOP:  {:?}\nEXPECT: '{:?}'\n", doc, op, expect);
        let mut result = doc.to_string().clone();
        op.apply(&mut result);
        assert_eq!(result, expect);
    }

    Ok(())
}

#[test]
fn test_transform() -> Result<()> {
    for data in JsonStreamIter::read_file("../text/transform.json")? {
        let op = json_to_op(&data["op"]).unwrap();
        let other_op = json_to_op(&data["otherOp"]).unwrap();
        let side_is_left = data["side"].as_str().unwrap() == "left";
        let expect = json_to_op(&data["result"]).unwrap();

        let result = transform(&op, &other_op, side_is_left);
        assert_eq!(result, expect);
    }

    Ok(())
}