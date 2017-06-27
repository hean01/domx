/// A trait for converting a type to HTML representation
pub trait ToHTML {
    /// Formats the value to HTML representation.
    fn to_html(&self) -> String;
}
