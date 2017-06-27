//! # domx - HTML Parser and DOM builder
//!
//! __domx__ includes a small HTML [Parser] and [DOM] builder for
//! easing the work with HTML data as structured data. The goal is to
//! be very resilience against invalid HTML documents, eg. missing
//! closing tag etc. In worst case you just get strange data from the
//! parser.
//!
//! The [Parser] itself runs through the HTML document and using the
//! trait [IsParser], implemented by the caller as handler, you will
//! be notified when a opening tag, closing tag and data is parsed.
//! Information through the callback is provided as [Tag], a vector of
//! [Attribute] and data as a vector of u8. See example below how to
//! use the [Parser] and a simple implementation of [IsParser].
//!
//! The [DOM] builder uses the parser to build up a tree data
//! structure of the HTML document. Which you can traverse and perform
//! operations on such as cleaning up the document or just simplify
//! it. Running a broken HTML, eg missing closing tags, into DOM and
//! then saving it you will get a nice consistent and valid HTML file.
//!
//! __domx__ is licensed under GPLv3
//!
//! [DOM]: struct.Dom.html
//! [Parser]: struct.Parser.html
//! [IsParser]: trait.IsParser.html
//! [Tag]: enum.Tag.html
//! [Attribute]: struct.Attribute.html
//!
//!
//! # Panics
//!
//! There is only one place a panic!() is called and that is were an
//! unknown HTML tag is encountered. This is temporary and will be
//! removed in stable release.
//!
//!
//! # Examples
//!
//! Here follows a simple example how to use the DOM parser to filter
//! a HTML document reming a few element with their childs. The
//! retain() method are used and a closure to test nodes in tree, just
//! as one would use the retain function on rust std vector.
//!
//! ```rust
//! #[macro_use]
//! extern crate domx;
//!
//! use domx::{ToHTML, Tag};
//!
//! fn main() {
//!   let mut d = dom!("<html><header><title>An example</title></header> \
//!     <body><h1>Header</h1><p>Some text</p></body></html>");
//!
//!   println!("BEFORE: {} nodes\n{}", d.len(), d.to_html());
//!
//!   d.retain(|&ref node| {
//!     match node.element() {
//!       None => true,
//!       Some(x) => match *x.tag() {
//!         Tag::HEADER => false,
//!         Tag::H1 => false,
//!         _ => true,
//!       }
//!     }
//!   });
//!
//!   println!("AFTER: {} nodes\n{}", d.len(), d.to_html());
//! }
//! ```
//!
//! To use the parser you need to implement the trait IsParser and the
//! three handler callbacks. The following example will show how to do
//! this.
//!
//! ```
//! extern crate domx;
//!
//! use domx::{Parser, IsParser, Tag, Attribute};
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! struct MyParser;
//! impl IsParser for MyParser {
//!     fn handle_starttag(self: &mut Self, tag: &Tag, attributes: &Vec<Attribute>) {
//!         let mut av: Vec<String> = Vec::new();
//!
//!         av.push(tag.to_string());
//!
//!         for ref attr in attributes {
//!             av.push(format!("{}", attr));
//!         }
//!
//!         print!("<{}>", av.join(" ").as_str());
//!     }
//!
//!     fn handle_endtag(self: &mut Self, tag: &Tag) {
//!         print!("{}", tag.clone());
//!     }
//!
//!     fn handle_data(self: &mut Self, data: &Vec<u8>) {
//!         print!("{}", String::from_utf8(data.clone()).unwrap());
//!     }
//! }
//!
//! fn main() {
//!
//!     if std::env::args().len() != 2 {
//!         println!("Usage: passthrough <htmlfile>");
//!         return;
//!     }
//!
//!     let filename = std::env::args().nth(1).unwrap();
//!     let file = File::open(filename).unwrap();
//!     let mut reader = BufReader::new(file);
//!     Parser::parse(&mut reader, &mut MyParser{}).unwrap();
//! }
//! ```

mod traits;
pub use traits::{ToHTML};

mod tag;
pub use tag::{Tag};

mod attribute;
pub use attribute::{Attribute};

mod parser;
pub use parser::{Parser, IsParser};

#[macro_use]
mod dom;
pub use dom::{Dom};




