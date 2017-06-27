extern crate domx;
use std::fs::File;
use std::io::BufReader;

struct Dummy;
impl domx::IsParser for Dummy {
    fn handle_starttag(self: &mut Self, element: &domx::Tag, attributes: &Vec<domx::Attribute>) {

        let mut av: Vec<String> = Vec::new();

        av.push(element.to_string());

        for ref attr in attributes {
            av.push(format!("{}", attr));
        }

        print!("<{}>", av.join(" ").as_str());
    }

    fn handle_endtag(self: &mut Self, element: &domx::Tag) {
        print!("</{}>", element.to_string());
    }

    fn handle_data(self: &mut Self, data: &Vec<u8>) {
        print!("{}", String::from_utf8(data.clone()).unwrap());
    }
}

fn main() {

    if std::env::args().len() != 2 {
        println!("Usage: passthrough <htmlfile>");
        return;
    }

    let filename = std::env::args().nth(1).unwrap();
    let file = File::open(filename).unwrap();
    let mut reader = BufReader::new(file);
    domx::Parser::parse(&mut reader, &mut Dummy{}).unwrap();
}
