use std::rc::Rc;

mod node {

    type ChdPtr = Option<Box<NodeType>>;

    const MAX_CHILDREN_PER_NODE: usize = 60;

    pub enum NodeType {
        Node,
        DataNode,
    }

    pub struct Node {
        pub pidx: Option<usize>, //Index id in parent Node
        value: String,
        key: String,
        children: [ChdPtr; MAX_CHILDREN_PER_NODE]
    }

    impl Node {
        pub(crate) fn new(pidx: Option<usize>, key: &str, value: &str) -> Self {
            Node {
                pidx,
                value: value.to_string(),
                key: key.to_string(),
                children: [None; MAX_CHILDREN_PER_NODE],
            }
        }
    }

    pub struct DataNode {
        value: String
    }
}

pub struct LazzyTree {
    root: Rc<node::Node>,
    sluggish: usize,
}

impl LazzyTree {
    pub fn new(key: &str, value: &str, sluggish: usize) -> Self {
        LazzyTree {
            root: Rc::new(node::Node::new(None, key, value)),
            sluggish
        }
    }
}