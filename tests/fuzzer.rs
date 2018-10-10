extern crate json;

extern crate text;

use text::{OpComponent, TextOp};
use std::fs::File;
use std::io::{BufReader, BufRead, Result};

fn read_files() -> Result<()> {
    let f = File::open("../text/apply.json")?;
    let file = BufReader::new(&f);
    for line in file.lines() {
        let line = line?;
        if line.len() == 0 { continue; }
        let data = json::parse(&line).unwrap();

        let doc = data["str"].as_str().unwrap();
        let mut op = TextOp::new();
        for m in data["op"].members() {
            use json::JsonValue::*;
            use text::OpComponent::*;
            op.append_move(match m {
                Short(s) => OpComponent::ins_from(s.as_str()),
                json::JsonValue::String(s) => OpComponent::ins_from(s.clone()),
                Number(n) => Skip(usize::from(*n)),
                Object(obj) => Del(obj["d"].as_usize().unwrap()),
                _ => { panic!("Invalid data {}", m); }
            });
        }
        
        let expect = data["result"].as_str().unwrap().to_string();
        print!("========\n11pect: '{}'\n", expect);
        // print!("\n\nDOC: '{}'\nOP:  {:?}\nEXPECT: '{}'\n", doc, op, expect);
        let mut result = doc.to_string().clone();
        op.apply(&mut result);
        print!("========\ne2pect: '{}'\n", expect);
        assert_eq!(result, expect);
    }

    Ok(())
}

// #[derive(Deserialize, Debug)]
// struct ApplyEntry {
//     str: String,
//     op: 
// }

#[test]
fn foo() {
    read_files().unwrap();
}