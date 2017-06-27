use std;

use traits::{ToHTML};

/// Attribute representing a HTML attribute name and value.
///
#[derive(Clone)]
pub struct Attribute {
    #[doc(hidden)]
    pub name: Vec<u8>,
    #[doc(hidden)]
    pub value: Vec<u8>,
}

impl Attribute {
    /// Create new attribute
    pub fn new(name: &str, value: &str) -> Attribute {
        Attribute {
            name: name.to_string().into_bytes(),
            value: value.to_string().into_bytes(),
        }
    }

    /// Create new boolean attribute, eg. no value
    pub fn new_boolean(name: &str) -> Attribute {
        Attribute {
            name: name.to_string().into_bytes(),
            value: Vec::new(),
        }
    }

    /// Test if attribute is boolean value
    pub fn is_boolean(&self) -> bool {
        match self.value.len() {
            0 => true,
            _ => false,
        }
    }

    /// Get attribute name as utf8 encoded string
    pub fn name(&self) -> String {
        String::from_utf8(self.name.clone()).unwrap()
    }

    /// Get attribute value as utf8 encoded string
    pub fn value(&self) -> String {
        String::from_utf8(self.value.clone()).unwrap()
    }
}

impl ToHTML for Attribute {
    fn to_html(&self) -> String {
        match self.is_boolean() {
            true => self.name(),
            false => format!("{}=\"{}\"", self.name(), self.value())
        }
    }
}

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.is_boolean() {
            true => f.write_str(&self.name()),
            false => f.write_str(&format!("{}=\"{}\"", self.name(), self.value()))
        }
    }
}

#[cfg(test)]
mod tests {
    use traits::ToHTML;
    use attribute::Attribute;

    #[test]
    fn new_boolean_is_boolean() {
        let a = Attribute::new_boolean("selected");
        assert_eq!(a.is_boolean(), true);
    }

    #[test]
    fn new_is_not_boolean() {
        let a = Attribute::new("class", "info");
        assert_eq!(a.is_boolean(), false);
    }

    #[test]
    fn new_with_utf8_value() {
        let a = Attribute::new("id", "💖");
        assert_eq!(a.value(), "💖");
    }

    #[test]
    fn attribute_to_html() {
        let a = Attribute::new("id", "💖");
        assert_eq!(a.to_html(), "id=\"💖\"");
    }
}
