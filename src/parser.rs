use std;
use std::io::BufRead;

use tag::{Tag};
use attribute::{Attribute};

#[derive(Clone)]
struct ParserTag {
    name: String,
    pub id: Option<Tag>,
    closing: bool,
    data: Vec<u8>,
    pub attributes: Vec<Attribute>
}

impl std::fmt::Display for ParserTag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.closing {
            false => f.write_str("<"),
            true => f.write_str("</")
        }.unwrap();

        f.write_str(&format!("{}",self.id.as_ref().unwrap())).unwrap();

        let mut av: Vec<String> = Vec::new();
        for ref attr in self.attributes.clone() {
            av.push(format!("{}", attr));
        }
        if av.len() != 0 {
            f.write_str(" ").unwrap();
            f.write_str(av.join(" ").as_str()).unwrap();
        }

        f.write_str(">")
    }
}

/// A trait for handling callbacks from [Parser](struct.Parser.html).
///
pub trait IsParser {
    /// This method is called to handle the start tag.
    fn handle_starttag(self: &mut Self, tag: &Tag, attributes: &Vec<Attribute>);

    /// This method is called to handle the end tag of a element.
    fn handle_endtag(self: &mut Self, tag: &Tag);

    /// This method is called to process arbitrary data.
    ///
    /// Data beeing text nodes and the content of ```<script>...</script>```
    /// and ```<style>...</style>``` tags.
    fn handle_data(self: &mut Self, _data: &Vec<u8>);
}

/// Parse a HTML document and provide data through handler [IsParser].
///
/// The parser serves the basis for parsing text files formatted in
/// HTML and XHTML. The parser is not based on SGML.
///
/// [IsParser]: trait.IsParser.html
///
pub struct Parser;

enum ParserState {
    FindParserTag,
    SkipComment,
    ReadParserTagName,
    ReadData,
    ReadRawData,
    ReadAttributeName,
    ReadAttributeValue,
}

impl Parser {

    fn _state_find_tag(buf: &Vec<u8>, tag: &mut ParserTag, state: &mut ParserState) -> usize {
        let mut processed = 0;
        for b in buf {
            if *b == '<' as u8 {
                tag.name = "".to_string();
                tag.id = None;
                tag.data.clear();
                tag.closing = false;
                *state = ParserState::ReadParserTagName;
                break;
            }
            processed += 1;
        }
        return processed;
    }

    fn _state_skip_comment(buf: &Vec<u8>, state: &mut ParserState) -> usize {
        let mut processed = 0;
        loop {
            if (processed + 3) > buf.len() {
                // we need more data to continue
                break;
            }

            if buf[processed + 0] == '-' as u8 &&
                buf[processed + 1] == '-' as u8 &&
                buf[processed + 2] == '>' as u8 {

                *state = ParserState::ReadData;
                processed += 3;
                break;
            }

            processed += 1
        }

        return processed;
    }

    fn _state_read_tag_name(buf: &Vec<u8>, tag: &mut ParserTag, state: &mut ParserState) -> usize {
        let mut processed = 0;

        for b in buf {

            match *b as char {
                // Skip begin of tag
                '<' => processed += 1,

                '\r' | '\n' => {
                    processed += 1;
                },

                // Comment detected
                '!' => {
                    processed += 1;
                    *state = match buf[processed] as char {
                        '-' => ParserState::SkipComment,
                        _   => ParserState::FindParserTag
                    };
                    break;
                },

                // Closing tag detected
                '/' => {
                    tag.closing = true;
                    processed += 1;
                },

                // Complete tag name read 
                '>' | ' ' => {
                    tag.attributes.clear();
                    tag.attributes.push(Attribute{name: Vec::new(), value: Vec::new()});
                    *state = ParserState::ReadAttributeName;
                    break;
                },

                _ => {
                    tag.name.push(*b as char);
                    processed += 1;
                }
            }
        }

        return processed;
    }

    fn _is_element(buf: &Vec<u8>, el: &Option<Tag>) -> Result<bool, ()> {
        if el.is_none() {
            return Ok(false);
        }

        let t = el.as_ref().unwrap().to_string().into_bytes();

        // println!("buf: {}, tag: {}", buf.len(), t.len());
        if buf.len() < t.len() + 3 {
            return Err(())
        }

        let b = match buf[1] as char {
            '/' => buf[2..2+t.len()].to_vec(),
            _ => buf[..t.len()].to_vec()
        };

        Ok(t == b)
    }

    // Read data until next inline element
    fn _state_read_data(buf: &Vec<u8>, tag: &mut ParserTag, state: &mut ParserState, handler: &mut dyn IsParser) -> usize {
        let mut processed = 0;

        for b in buf {
            match *b as char {
                // Found begin of new tag which means we have read
                // available data
                '<' => {
                    if tag.data.len() > 0 {
                        handler.handle_data(&tag.data);
                    }
                    *state = ParserState::FindParserTag;
                    break;
                },

//                '\r' | '\n' => {
//                    processed += 1;
//                },

                _ => {
                    tag.data.push(*b);
                    processed += 1;
                }
            }
        }

        return processed;
    }

    // Read data until closing element, eg <script></script>
    fn _state_read_raw_data(buf: &Vec<u8>, tag: &mut ParserTag, state: &mut ParserState, handler: &mut dyn IsParser) -> usize {
        let mut processed = 0;

        for b in buf {

            match *b as char {

                // Found possible begin of closing tag
                '<' => {

                    match Parser::_is_element(&buf[processed..].to_vec(), &tag.id) {
                        Err(_) => break,
                        Ok(x) => {
                            match x {
                                true => {
                                    // Found closing tag, lets handle data
                                    if tag.data.len() > 0 {
                                        handler.handle_data(&tag.data);
                                    }
                                    *state = ParserState::FindParserTag;
                                    break;
                                },
                                false => {
                                    // Does not match, just byte add to data
                                    tag.data.push(*b);
                                    processed += 1; 
                                }
                            }
                        }
                    }
                },

                _ => {
                    tag.data.push(*b);
                    processed += 1;
                }
            }
        }

        return processed;
    }


    fn _state_read_attribute_value(buf: &Vec<u8>, tag: &mut ParserTag, state: &mut ParserState, handler: &mut dyn IsParser ) -> usize {
        let mut processed = 0;
        for b in buf {
            match *b as char {
                '>' => {
                    // pop last attribute if it is an empty placeholder
                    if tag.attributes.last().as_ref().unwrap().name() == "" {
                        tag.attributes.pop();
                    }

                    tag.id = Some(tag.name
                        .to_lowercase()
                        .parse::<Tag>().unwrap());

                    *state = match tag.closing {
                        false => {
                            handler.handle_starttag(tag.id.as_ref().unwrap(), &tag.attributes);
                            match *tag.id.as_ref().unwrap() {
                                Tag::SCRIPT => ParserState::ReadRawData,
                                Tag::STYLE => ParserState::ReadRawData,
                                _ => ParserState::ReadData,
                            }
                        },
                        true => {
                            handler.handle_endtag(tag.id.as_ref().unwrap());
                            ParserState::ReadData
                        }
                    };
                    tag.attributes.clear();
                    processed += 1;
                    break;
                },

                '"' | '\'' => {

                    let have_value = !(tag.attributes.last().as_mut().unwrap().value() == "");

                    if have_value && *b != ' ' as u8 {

                        {
                            // Trim " and ' from attribute value
                            let ref mut value = tag.attributes.last_mut().unwrap().value;
                            if value.len() != 0 && (value[0] == '\'' as u8 || value[0] == '"' as u8) {
                                *value = value[1..].to_vec();
                            }

                            if value.len() != 0 && (value[value.len() - 1] == '\'' as u8 || value[value.len() - 1] == '"' as u8) {
                                *value = value[..value.len() - 1].to_vec();
                            }

                        }

                        tag.attributes.push(Attribute{name: Vec::new(), value: Vec::new()});
                        *state = ParserState::ReadAttributeName;
                        processed += 1;
                        break;

                    } else {

                        let ref mut value = tag.attributes.last_mut().unwrap().value;
                        value.push(*b);
                        processed += 1;
                    }
                },

                ' ' => {

                    let (have_value, is_quoted) = {
                        let ref mut value = tag.attributes.last_mut().unwrap().value;
                        match value.is_empty() {
                            true => (false, false),
                            false => (true, (value[0] == '"' as u8 || value[0] == '\'' as u8)),
                        }
                    };

                    if have_value && !is_quoted {

                        tag.attributes.push(Attribute{name: Vec::new(), value: Vec::new()});
                        *state = ParserState::ReadAttributeName;
                        processed += 1;
                        break;

                    } else {

                        let ref mut value = tag.attributes.last_mut().unwrap().value;
                        value.push(*b);
                        processed += 1;
                    }
                }

                _ => {
                    let ref mut value = tag.attributes.last_mut().unwrap().value;
                    value.push(*b);
                    processed += 1;
                }
            }
        }
        return processed;
    }

    fn _state_read_attribute_name(buf: &Vec<u8>, tag: &mut ParserTag, state: &mut ParserState, handler: &mut dyn IsParser) -> usize {
        let mut processed = 0;

        for b in buf {

            match *b as char {

                // Found closing, lets finish up
                '>' | '/' => {

                    {
                        if tag.attributes.last().unwrap().name.is_empty() {
                            // pop last attribute if it is an empty placeholder
                            tag.attributes.pop();
                        }
                    }

                    match tag.name.to_lowercase().parse::<Tag>() {
                        Ok(x) => tag.id = Some(x),
                        Err(_) => panic!("Failed to parse element '{}' to enum", tag.name),
                    }

                    *state = match tag.closing {
                        false => {
                            handler.handle_starttag(tag.id.as_ref().unwrap(), &tag.attributes);
                            match *tag.id.as_ref().unwrap() {
                                Tag::SCRIPT => ParserState::ReadRawData,
                                _ => ParserState::ReadData,
                            }
                        },
                        true => {
                            handler.handle_endtag(tag.id.as_ref().unwrap());
                            ParserState::ReadData
                        }
                    };

                    processed += 1;
                    break;
                },

                '=' => {
                    *state = ParserState::ReadAttributeValue;
                    processed += 1;
                    break;
                },

                ' ' => {
                    if !tag.attributes.last().unwrap().name.is_empty() {
                        tag.attributes.push(Attribute{name: Vec::new(), value: Vec::new()});
                        *state = ParserState::ReadAttributeName;
                    }

                    processed += 1;
                },

                '\n' | '\r' | '\t' => {
                    processed += 1;
                },

                _ => {
                    let ref mut name = tag.attributes.last_mut().unwrap().name;
                    name.push(*b);
                    processed += 1;
                }
            }
        }

        return processed;
    }

    /// Parse a HTML document and call handlers.
    pub fn parse(source: &mut dyn BufRead, handler: &mut dyn IsParser) -> Result<usize, std::io::Error> {
        let mut total_parsed = 0;
        let mut state = ParserState::FindParserTag;

        let mut tag = ParserTag {
            name: "".to_string(),
            id: None,
            closing: false,
            data: Vec::new(),
            attributes: Vec::new()
        };


        let mut buf = Vec::new();
        let mut end_of_file = false;
        loop {

            // If buffer is low and there is still data to be read, read block
            if !end_of_file && buf.len() < 64 {
                let mut block = [0; 2048];
                let bytes_read = match source.read(&mut block[..]) {
                    Ok(x) => x,
                    Err(x) => return Err(x)
                };

                match bytes_read {
                    0 => end_of_file = true,
                    _ => {
                        let size = buf.len();
                        buf.extend_from_slice(&block);
                        buf = buf[..size+bytes_read].to_vec();
                    }
                };
            }

            // Break out of loop if buffer is empty
            if buf.len() == 0 {
                break;
            }

            loop {
                let processed = match state {
                    ParserState::FindParserTag => Parser::_state_find_tag(&buf, &mut tag, &mut state),
                    ParserState::SkipComment => Parser::_state_skip_comment(&buf, &mut state),
                    ParserState::ReadParserTagName => Parser::_state_read_tag_name(&buf, &mut tag, &mut state),
                    ParserState::ReadData => Parser::_state_read_data(&buf, &mut tag, &mut state, handler),
                    ParserState::ReadRawData => Parser::_state_read_raw_data(&buf, &mut tag, &mut state, handler),
                    ParserState::ReadAttributeName => Parser::_state_read_attribute_name(&buf, &mut tag, &mut state, handler),
                    ParserState::ReadAttributeValue => Parser::_state_read_attribute_value(&buf, &mut tag, &mut state, handler),
                };

                if processed == 0 {
                    break;
                }

                buf.drain(..processed);
                total_parsed += processed;

                if buf.len() == 0 {
                    break;
                }
            }
        }

        Ok(total_parsed)
    }
}

#[cfg(test)]
mod tests {
    use attribute::Attribute;
    use tag::Tag;
    use parser::{IsParser};
    use std::io::BufReader;

    struct TestTag {
        tag: Tag,
        attributes: Vec<Attribute>
    }

    struct Dummy {
        starttag: Vec<TestTag>,
        endtag: Vec<TestTag>,
        data: Vec<Vec<u8>>,
    }

    impl Dummy {
        pub fn new() -> Dummy {
            Dummy{
                starttag: Vec::new(),
                endtag: Vec::new(),
                data: Vec::new(),
            }
        }
    }

    impl IsParser for Dummy {
        fn handle_starttag(self: &mut Self, tag: &Tag, attributes: &Vec<Attribute>) {
            self.starttag.push(TestTag{
                tag: tag.clone(),
                attributes: attributes.clone(),
            });
        }

        fn handle_endtag(self: &mut Self, tag: &Tag) {
            self.endtag.push(TestTag{tag: tag.clone(), attributes: Vec::new()});
        }

        fn handle_data(self: &mut Self, data: &Vec<u8>) {
            self.data.push(data.clone());
        }
    }


    #[test]
    fn parse_empty_document() {
        let mut p = Dummy::new();
        let data = b"";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 0);
    }

    #[test]
    fn parse_simple_document() {
        let mut p = Dummy::new();
        let data = b"<html><head><title>Simple Example</title></head><body><h1>A simple doc</h1><p class=\"para\">This is a simple html document, as short and simple it can get.</p></body></html>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 172);
        assert_eq!(p.starttag.len(), 6);
        assert_eq!(p.endtag.len(), 6);
        assert_eq!(p.data.len(), 3);
        assert_eq!(p.starttag[5].attributes.len(), 1);
        assert_eq!(p.starttag[0].tag, Tag::HTML);
        assert_eq!(p.starttag[1].tag, Tag::HEAD);
        assert_eq!(p.starttag[2].tag, Tag::TITLE);
        assert_eq!(p.starttag[3].tag, Tag::BODY);
        assert_eq!(p.starttag[4].tag, Tag::H1);
        assert_eq!(p.starttag[5].tag, Tag::P);
    }

    #[test]
    fn parse_document_with_inline_comment() {
        let mut p = Dummy::new();
        let data = b"<html><head><title>Simple<!-- title --> Example</title></head><body><h1>A simple doc</h1><p class=\"para\">This is a simple html document, as short and simple it can get.</p></body></html>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 186);
        assert_eq!(p.starttag.len(), 6);
        assert_eq!(p.endtag.len(), 6);
        assert_eq!(p.data.len(), 4);
        assert_eq!(p.starttag[5].attributes.len(), 1);
        assert_eq!(String::from_utf8(p.data[0].clone()).unwrap(), "Simple");
        assert_eq!(String::from_utf8(p.data[1].clone()).unwrap(), " Example");
    }

    #[test]
    fn parse_document_with_comment() {
        let mut p = Dummy::new();
        let data = b"<html><head> <!-- set a title --> <title>Simple Example</title></head><body><h1>A simple doc</h1><p class=\"para\">This is a simple html document, as short and simple it can get.</p></body></html>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 194);
        assert_eq!(p.starttag.len(), 6);
        assert_eq!(p.endtag.len(), 6);
        assert_eq!(p.data.len(), 5);
        assert_eq!(p.starttag[5].attributes.len(), 1);
    }

    #[test]
    fn parse_tag_with_utf8_data() {
        let mut p = Dummy::new();
        let data = "<p>💖</p>".to_string().into_bytes();
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 11);
        assert_eq!(String::from_utf8(p.data[0].clone()).unwrap(), "💖");
    }

    #[test]
    fn parse_attribute_with_utf8_value() {
        let mut p = Dummy::new();
        let data = "<p id='💖'>Sparkle heart</p>".to_string().into_bytes();
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 30);
        assert_eq!(String::from_utf8(p.data[0].clone()).unwrap(), "Sparkle heart");
        assert_eq!(p.starttag[0].attributes[0].name(), "id");
        assert_eq!(p.starttag[0].attributes[0].value(), "💖");
    }

    #[test]
    fn parse_tag_with_one_attribute_without_qouted_value() {
        let mut p = Dummy::new();
        let data = b"<p id=1>Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 23);
        assert_eq!(p.starttag[0].attributes[0].name(), "id");
        assert_eq!(p.starttag[0].attributes[0].value(), "1");
    }

    #[test]
    fn parse_tag_with_one_attribute_with_doubleqouted_value() {
        let mut p = Dummy::new();
        let data = b"<p id=\"1\">Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 25);
        assert_eq!(p.starttag[0].attributes[0].name(), "id");
        assert_eq!(p.starttag[0].attributes[0].value(), "1");
    }

    #[test]
    fn parse_tag_with_one_attribute_with_singleqouted_value() {
        let mut p = Dummy::new();
        let data = b"<p id='1'>Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 25);
        assert_eq!(p.starttag[0].attributes[0].name(), "id");
        assert_eq!(p.starttag[0].attributes[0].value(), "1");
    }

    #[test]
    fn parse_tag_with_one_attribute_doubleqouted_with_space_in_value() {
        let mut p = Dummy::new();
        let data = b"<p class=\"info error\">Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 37);
        assert_eq!(p.starttag[0].attributes[0].name(), "class");
        assert_eq!(p.starttag[0].attributes[0].value(), "info error");
    }

    #[test]
    fn parse_tag_with_one_attribute_singlequoted_with_space_in_value() {
        let mut p = Dummy::new();
        let data = b"<p class='info error'>Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 37);
        assert_eq!(p.starttag[0].attributes[0].name(), "class");
        assert_eq!(p.starttag[0].attributes[0].value(), "info error");
    }

    #[test]
    fn parse_tag_with_one_attribute_with_space_ending_value() {
        let mut p = Dummy::new();
        let data = b"<p id=test class=info>Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 37);
        assert_eq!(p.starttag[0].attributes[0].name(), "id");
        assert_eq!(p.starttag[0].attributes[0].value(), "test");
        assert_eq!(p.starttag[0].attributes[1].name(), "class");
        assert_eq!(p.starttag[0].attributes[1].value(), "info");
    }

    #[test]
    fn parse_tag_with_one_attribute_with_space_ending_value2() {
        let mut p = Dummy::new();
        let data = b"<p id=test >Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 27);
        assert_eq!(p.starttag[0].attributes[0].name(), "id");
        assert_eq!(p.starttag[0].attributes[0].value(), "test");
    }

    #[test]
    fn parse_tag_with_two_attribute() {
        let mut p = Dummy::new();
        let data = b"<p id=\"myid\" class='info'>Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 41);
        assert_eq!(p.starttag[0].attributes[0].name(), "id");
        assert_eq!(p.starttag[0].attributes[0].value(), "myid");
        assert_eq!(p.starttag[0].attributes[1].name(), "class");
        assert_eq!(p.starttag[0].attributes[1].value(), "info");
    }

    #[test]
    fn parse_tag_with_two_attribute_separated_with_lf() {
        let mut p = Dummy::new();
        let data = b"<p id=\"myid\" \n\t class='info'>Hello world</p>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 44);
        assert_eq!(p.starttag[0].attributes[0].name(), "id");
        assert_eq!(p.starttag[0].attributes[0].value(), "myid");
        assert_eq!(p.starttag[0].attributes[1].name(), "class");
        assert_eq!(p.starttag[0].attributes[1].value(), "info");
    }

    #[test]
    fn parse_tag_with_one_boolean_attribute() {
        let mut p = Dummy::new();
        let data = b"<option selected>Hello world</option>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 37);
        assert_eq!(p.starttag[0].attributes[0].name(), "selected");
        assert_eq!(p.starttag[0].attributes[0].is_boolean(), true);
    }

    #[test]
    fn parse_tag_with_one_boolean_attribute_with_space_ending() {
        let mut p = Dummy::new();
        let data = b"<option selected >Hello world</option>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 38);
        assert_eq!(p.starttag[0].attributes[0].name(), "selected");
        assert_eq!(p.starttag[0].attributes[0].is_boolean(), true);
    }

    #[test]
    fn parse_tag_with_two_attribute_were_first_is_boolean_attribute() {
        let mut p = Dummy::new();
        let data = b"<option selected id=\"myid\">Hello world</option>";
        assert_eq!(::Parser::parse(&mut BufReader::new(&data[..]), &mut p).unwrap(), 47);
        assert_eq!(p.starttag[0].attributes[0].name(), "selected");
        assert_eq!(p.starttag[0].attributes[0].is_boolean(), true);
        assert_eq!(p.starttag[0].attributes[1].name(), "id");
        assert_eq!(p.starttag[0].attributes[1].value(), "myid");
    }
}

