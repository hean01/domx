extern crate domx;
use std::fs::File;
use std::io::BufReader;
use domx::ToHTML;

fn main() {

    if std::env::args().len() != 2 {
        println!("Usage: domtree <htmlfile>");
        return;
    }

    let filename = std::env::args().nth(1).unwrap();
    let file = File::open(filename).unwrap();
    let mut reader = BufReader::new(file);

    let mut dom = domx::Dom::new();
    dom.parse(&mut reader).unwrap();
    println!("{}", dom.to_html());
}
