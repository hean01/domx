use std;
use traits::{ToHTML};
use tag::{Tag};
use attribute::{Attribute};
use parser::{IsParser, Parser};

/// Id for node references between nodes
pub type NodeId = usize;


const ROOT_NODE_ID: NodeId = 0;

pub struct NodeElement {
    tag: Tag,
    attributes: Vec<Attribute>
}

impl NodeElement {
    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }
}

impl ToHTML for NodeElement {
    fn to_html(&self) -> String {
        let mut html: String = "".to_owned();
        html.push_str("<");
        html.push_str(&self.tag().to_string());
        for attr in self.attributes().iter() {
            html.push_str(" ");
            html.push_str(&attr.to_html());
        }
        html.push_str(">");
        html
    }
}

impl std::fmt::Display for NodeElement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("<").unwrap();
        f.write_str(&format!("{}",self.tag)).unwrap();

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

pub enum NodeData {
    Element(NodeElement),
    Data(String),
}

impl ToHTML for NodeData {
    fn to_html(&self) -> String {
        (match self {
            &NodeData::Element(ref x) => x.to_html(),
            &NodeData::Data(ref x) => x.to_string()
        }).to_string()
    }
}

impl std::fmt::Display for NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &NodeData::Element(ref x) => f.write_str(&format!("{}", x)),
            &NodeData::Data(ref x) => f.write_str(x)
        }
    }
}

/// Representing a node in the DOM tree
pub struct Node {
    id: NodeId,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    data: Option<NodeData>,
}

impl Node {

    /// Create a new element node
    pub fn new_element(tag: Tag, attributes: Vec<Attribute>) -> Node {
        Node {
            id: 0,
            parent: None,
            children: Vec::new(),
            data: Some(NodeData::Element(NodeElement{
                tag: tag,
                attributes: attributes,
            })),
        }
    }

    /// Create a new data node
    pub fn new_data(data: String) -> Node {
        Node {
            id: 0,
            parent: None,
            children: Vec::new(),
            data: Some(NodeData::Data(data)),
        }
    }

    /// Test if node is an element.
    pub fn is_element(&self) -> bool {
        match self.data.as_ref().unwrap() {
            &NodeData::Element(_) => true,
            _ => false
        }
    }

    /// Test if node is data.
    pub fn is_data(&self) -> bool {
        match self.data.as_ref().unwrap() {
            &NodeData::Data(_) => true,
            _ => false
        }
    }

    pub fn element(&self) -> Option<&NodeElement> {
        match self.data.as_ref().unwrap() {
            &NodeData::Element(ref x) => Some(x),
            _ => None
        }
    }

    pub fn data(&self) -> &NodeData {
        self.data.as_ref().unwrap()
    }
}

/// Store used for allocation
struct Store {
    nodes: Vec<Option<Node>>
}

impl std::ops::Index<usize> for Store {
    type Output = Option<Node>;
    fn index(&self, idx: usize) -> &Option<Node> {
        &self.nodes[idx]
    }
}

impl std::ops::IndexMut<usize> for Store {
    fn index_mut(&mut self, idx: usize) -> &mut Option<Node> {
        &mut self.nodes[idx]
    }
}

impl Store {
    pub fn new() -> Store {
        Store {
            nodes: vec!(Some(Node{
                id: 0,
                parent: None,
                children: Vec::new(),
                data: None
            }))
        }
    }

    /// Add node to store and return NodeId
    pub fn add(self: &mut Store, node: Node) -> Result<NodeId, ()> {

        let parent = node.parent.unwrap();
        self.nodes.push(Some(node));

        let id = self.nodes.len() - 1;
        self[parent].as_mut().unwrap().children.push(id);

        self.nodes[id].as_mut().unwrap().id = id;
        Ok(id)
    }

    pub fn is_node(self: &Store, id: NodeId) -> bool {
        if id <= self.nodes.len() - 1 && self.nodes[id].is_some() {
            return true;
        }
        return false;
    }

    /// Create a new node with parent and return NodeId
    pub fn new_node_with_parent(self: &mut Store, parent: NodeId) -> Result<NodeId, ()> {

        // validate parent
        if !self.is_node(parent) {
            // Invalid parent
            return Err(());
        }

        // create and add new node returning new NodeId
        self.add(Node{
            id: 0,
            parent: Some(parent),
            children: Vec::new(),
            data: None
        })
    }

    fn _recurse<F>(self: &Store, id: NodeId, level: usize, enter: &mut F)
        where
        F: FnMut(NodeId, usize),
    {
        match self.nodes[id] {
            Some(ref x) => {
                for cid in x.children.iter() {
                    enter(*cid, level);
                    self._recurse(*cid, level + 1, enter);
                }
            }
            None => ()
        }
    }

    fn _recurse_with_output<F1, F2>(self: &Store, id: NodeId, enter: &mut F1, leave: &mut F2, output: &mut String)
        where
        F1: FnMut(&Node, &mut String),
        F2: FnMut(&Node, &mut String),
    {
        match self.nodes[id] {
            Some(ref x) => {
                for cid in x.children.iter() {
                    let node = self.nodes[*cid].as_ref().unwrap();
                    enter(node, output);
                    self._recurse_with_output(*cid, enter, leave, output);
                    leave(node, output);
                }
            }
            None => ()
        }
    }

    fn _recurse_remove_node(&self, id: NodeId, nodes: &mut Vec<NodeId>)
    {

        match self.nodes[id] {
            None => (),
            Some(ref x) => {
                // recurse to leaf and then remove nodes back to top
                {
                    for cid in x.children.iter() {
                        self._recurse_remove_node(*cid, nodes);
                    }
                }
            }
        };

        nodes.push(id);
    }

    // Get nodes that are not none in storage
    pub fn len(&self) -> usize {
        let mut cnt = 0;
        for n in self.nodes.iter() {
            cnt += match n {
                &None => 0,
                _ => 1,
            };
        }

        cnt
    }

    pub fn remove(&mut self, id: NodeId) {

        if !self.is_node(id) {
            return;
        }

        let mut nodes = Vec::new();
        self._recurse_remove_node(id, &mut nodes);

        for nid in nodes.iter() {
            {
                let parent_id = { self.nodes[*nid].as_mut().unwrap().parent.unwrap() };
                let parent = self[parent_id].as_mut().unwrap();
                parent.children.retain(|&x| x != *nid);
            }
            self[*nid] = None;
        }
    }

    pub fn recurse<F>(self: &Store, id: NodeId, mut enter: F)
        where
        F: FnMut(NodeId, usize),
    {
        match self.is_node(id) {
            true => self._recurse(id, 0, &mut enter),
            false => (),
        }
    }

    pub fn retain<F>(&mut self, mut keep: F)
        where
        F: FnMut(&Node) -> bool,
    {
        // recurse into tree and for each node call keep and store
        // node to be removed into vector for second remove pass node.
        let mut nodes = Vec::new();
        self.recurse(ROOT_NODE_ID, |id, _| {

            if keep(self[id].as_ref().unwrap()) == false {
                nodes.push(id);
            };
        });

        for id in nodes {
            self.remove(id);
        }
    }
}

impl ToHTML for Store {
    fn to_html(&self) -> String {
        let mut html = "".to_string().to_owned();
        self._recurse_with_output(ROOT_NODE_ID,&mut |node, output|{
            output.push_str(node.data().to_html().as_str());
        },&mut |node, output|{
            match node.is_element() {
                true => {
                    output.push_str("</");
                    output.push_str(node.element().as_ref().unwrap().tag().to_string().as_str());
                    output.push_str(">");
                },
                false => (),
            }
        }, &mut html);

        html
    }
}

/// Instantiates and parses a HTML document into a DOM tree structure.
///
/// # Examples
///
/// ```
/// # #[macro_use]
/// # extern crate domx;
/// # fn main() {
/// let mut dom = dom!("<html><p>Hello world!</p></html>");
/// # }
/// ```
#[macro_export] macro_rules! dom {
    ( $html:expr ) => {
        {
            let mut temp_dom = $crate::Dom::new();
            let data = format!("{}", $html).into_bytes();
            temp_dom.parse(&mut std::io::BufReader::new(&data[..])).unwrap();

            temp_dom
        }
    };
}

/// DOM tree data structure builder.
///
/// Uses [Parser] to build the tree and provides a set of methods to
/// work with the tree. Implement [ToHTML] trait so that one can dump
/// the tree into a HTML document.
///
/// [ToHtml]: trait.ToHTML.html
/// [Parser]: struct.Parser.html
///
pub struct Dom {
    store: Store,
    current: Option<NodeId>
}

impl Dom {
    pub fn new() -> Dom {
        Dom {
            store: Store::new(),
            current: None,
        }
    }

    /// Parse a HTML buffer and build DOM tree structure.
    ///
    /// Use the macro [dom!()] for easier use.
    ///
    /// [dom!()]: macro.dom.html
    pub fn parse(self: &mut Self, source: &mut dyn std::io::BufRead) -> Result<usize, std::io::Error> {
        Parser::parse(source, self)
    }

    /// Recurse the DOM with a callback for when entering each node.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate domx;
    /// # fn main() {
    /// let mut d = dom!("<html></html>");
    /// d.recurse(|id, level| {
    ///   println!("Enter {} id({})", level, id);
    /// });
    /// # }
    /// ```
    pub fn recurse<F>(self: &Dom, enter: F)
        where
        F: FnMut(NodeId, usize),
    {
        self.store.recurse(ROOT_NODE_ID, enter);
    }

    /// Retains only the nodes specified by the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate domx;
    /// # use domx::{Tag,ToHTML};
    /// # fn main() {
    /// let mut d = dom!("<html><body><div><p>remove</p></div><p>Hello World!</p><div><p>remove <img src=''></img></p></div></body></html>");
    ///
    /// println!("{}", d);
    /// d.retain(|&ref node| {
    ///   match node.element() {
    ///     None => true,
    ///     Some(x) => match x.tag() {
    ///       &Tag::DIV => false,
    ///       _ => true,
    ///     }
    ///   }
    /// });
    ///
    /// println!("{}\nLength: {}", d.to_html(), d.len());
    /// # }
    /// ```
    pub fn retain<F>(&mut self, keep: F)
        where F: FnMut(&Node) -> bool
    {
        self.store.retain(keep)
    }

    pub fn len(&self) -> usize {
        self.store.len() - 1
    }
}

impl IsParser for Dom {
    fn handle_starttag(self: &mut Self, tag: &Tag, attributes: &Vec<Attribute>) {
        let parent = {
            match self.current {
                Some(x) => x,
                None => ROOT_NODE_ID
            }
        };
        let id = self.store.new_node_with_parent(parent).unwrap();
        self.store[id].as_mut().unwrap().data = Some(NodeData::Element(NodeElement{
            tag: tag.clone(),
            attributes: attributes.clone(),
        }));
        self.current = Some(id);
    }

    fn handle_endtag(self: &mut Self, _tag: &Tag) {
        self.current = self.store[self.current.unwrap()].as_ref().unwrap().parent;
    }

    fn handle_data(self: &mut Self, data: &Vec<u8>) {
        let parent = {
            match self.current {
                Some(x) => x,
                None => ROOT_NODE_ID
            }
        };
        let id = self.store.new_node_with_parent(parent).unwrap();
        self.store[id].as_mut().unwrap().data = Some(NodeData::Data(String::from_utf8(data.clone()).unwrap()));
    }
}

impl std::fmt::Display for Dom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.recurse(|id, level| {
            let li = vec![0; level];
            let indent = li.iter().fold("".to_string(), |acc, _| acc + "  ");
            match self.store[id].as_ref().unwrap().data {
                None => (),
                Some(ref x) => {
                    match x {
                        &NodeData::Element(ref x) => f.write_str(&format!("{}node({}) element: {}\n", indent, id, x)).unwrap(),
                        &NodeData::Data(ref x)  => f.write_str(&format!("{}node({}) data: {:?}\n", indent, id, x)).unwrap(),
                    }
                }
            };

        });
        f.write_str("")
    }
}

impl std::ops::Index<usize> for Dom {
    type Output = Node;
    fn index(&self, idx: usize) -> &Node {
        self.store[idx].as_ref().unwrap()
    }
}

impl std::ops::IndexMut<usize> for Dom {
    fn index_mut(&mut self, idx: usize) -> &mut Node {
       self.store[idx].as_mut().unwrap()
    }
}

impl ToHTML for Dom {
    fn to_html(&self) -> String {
        self.store.to_html()
    }
}


#[cfg(test)]
mod tests {
    use dom::*;
    use tag::Tag;
    use attribute::Attribute;
    use std::io::BufReader;

    #[test]
    fn parse_empty_document() {
        let mut dom = ::Dom::new();
        let data = "".to_string().into_bytes();
        assert_eq!(dom.parse(&mut BufReader::new(&data[..])).unwrap(), 0);
    }

    #[test]
    fn parse_simple_document() {
        let dom = dom!("<html><body><p>Hello <b>World</b>!</p></body></html>");
        assert_eq!(dom[3].data().to_string(), "<p>");
        assert_eq!(dom[6].data().to_string(), "World");
    }

    #[test]
    fn node_new_element_to_html() {
        let el = "p".parse::<Tag>().unwrap();
        let attrs = vec!(Attribute::new("id", "myid"), Attribute::new("class", "info data"));
        let node = Node::new_element(el, attrs);
        assert_eq!(node.element().unwrap().to_html(), "<p id=\"myid\" class=\"info data\">");
    }

    #[test]
    fn dom_retain_all() {
        let mut dom = dom!("<html><body><p>Hello <b>World</b>!</p></body></html>");
        dom.retain(|_| {
            true
        });

        assert_eq!(dom.len(), 7);
    }

    #[test]
    fn dom_retain_none() {
        let mut dom = dom!("<html><body><p>Hello <b>World</b>!</p></body></html>");
        println!("{}", dom);
        dom.retain(|_| {
            false
        });

        assert_eq!(dom.len(), 0);
    }

    #[test]
    fn dom_retain_all_but_p() {
        let mut dom = dom!("<html><body><p>Hello <b>World</b>!</p></body></html>");
        println!("{}", dom);
        dom.retain(|&ref node| {
            match node.element() {
                None => true,
                Some(x) => match x.tag() {
                    &Tag::P => false,
                    _ => true,
                }
            }
        });

        assert_eq!(dom.len(), 2);
    }
}
