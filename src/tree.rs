extern crate chrono;

use crate::errors::RstzError;
use crate::events::DataPoint;
use chrono::Duration;
use node::KEY_BYTE_LENGHT;
use node::{ChdPtr, Node, NodeType};
use std::rc::Rc;

mod node {

    use crate::errors::RstzError;

    pub(super) type ChdPtr = Option<Box<NodeType>>;

    const MAX_CHILDREN_PER_NODE: usize = 32; // Use module to fit the 254 possible results in a byte into 32.
    pub const KEY_BYTE_LENGHT: usize = 16;

    pub enum NodeType {
        TreeNode(Node),
        LeafNode(TSNode),
    }

    pub struct Node {
        pub pidx: Option<usize>, //Index id in parent Node
        key: [u8; KEY_BYTE_LENGHT],
        children: [ChdPtr; MAX_CHILDREN_PER_NODE],
    }

    impl Node {
        pub(crate) fn new(
            pidx: Option<usize>,
            key: [u8; KEY_BYTE_LENGHT],
        ) -> Self {
            Node {
                pidx,
                key,
                children: Default::default(),
            }
        }

        pub fn child_as_mut(&mut self, idx: usize) -> &mut ChdPtr {
            &mut self.children[idx]
        }

        pub fn child_as_ref(&self, idx: usize) -> &ChdPtr {
            &self.children[idx]
        }

        pub fn add_child(&mut self, child: NodeType, index: usize) -> Result<(), RstzError> {
            if index >= MAX_CHILDREN_PER_NODE {
                return Err(RstzError::new("Child index out of bounds"));
            }
            self.children[index] = Some(Box::new(child));
            Ok(())
        }
    }

    pub struct TSNode {
        ts: Vec<u8>,
    }
}

pub struct LazzyTree {
    root: Box<node::Node>,
    timewindow: Duration,
}

impl LazzyTree {
    pub fn new(
        key: [u8; KEY_BYTE_LENGHT],
        value: &str,
        timewindow: Duration,
    ) -> Self {
        let root_node = Node::new(None, key);
        
        LazzyTree {
            root: Box::new(node::Node::new(None, key)),
            timewindow,
        }
    }

    /*
    pub fn insert(&mut self, key: [u8; KEY_BYTE_LENGHT], value: &str) -> Result<(), RstzError> {
        let pidx = LazzyTree::keycmp(self.root.as_ref(), key);
        LazzyTree::place(pidx, self.root.child_as_mut(pidx), key, value)
    }

     fn place(
        pidx: usize,
        ptr: &mut ChdPtr,
        key: [u8; KEY_BYTE_LENGHT],
        value: &str,
    ) -> Result<(), RstzError> {
        if let Some(child) = ptr {
            match child.as_mut() {
                NodeType::TreeNode(n) => {
                    let npidx = LazzyTree::keycmp(n, key);
                    LazzyTree::place(npidx, &mut n.children[npidx], key, value)?
                }
                NodeType::LeafNode(d) => {}
            }
        }
        Ok(())
    } 

    fn keycmp(ptr: &Node, key: [u8; KEY_BYTE_LENGHT]) -> usize {
        let mut res: usize = 0;
        for i in 0..KEY_BYTE_LENGHT {
            if ptr.key[i] != key[i] {
                return res;
            }
            res += 1;
        }
        res
    } */
}
