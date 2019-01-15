use std;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use super::*;

#[derive(Debug, Eq)]
pub struct HuffmanType {
    symbol: u8,
    frequency: u64,
}

impl HuffmanType {
    pub fn new(symbol: u8, frequency: u64) -> HuffmanType {
        HuffmanType { symbol, frequency }
    }
}

impl Ord for HuffmanType {
    fn cmp(&self, other: &Self) -> Ordering {
        (other.frequency, other.symbol).cmp(&(self.frequency, self.symbol))
    }
}

impl PartialOrd for HuffmanType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HuffmanType {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}


#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    pub left: Option<Box<Node<T>>>,
    pub right: Option<Box<Node<T>>>,
    pub parent: * mut Node<T>,
}

impl <T> Node<T> {
    pub fn new(value: T) -> Node<T> {
        Node {
            value,
            left: None,
            right: None,
            parent: std::ptr::null_mut(),
        }
    }

    pub fn set_left(&mut self, mut node: Box<Node<T>>) {
        node.parent = self;
        self.left = Some(node);
    }

    pub fn set_right(&mut self, mut node: Box<Node<T>>) {
        node.parent = self;
        self.right = Some(node);
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

pub type HuffmanNode = Node<HuffmanType>;

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl Eq for HuffmanNode {}

pub struct HuffmanTree {
    pub root_node: Box<HuffmanNode>,
}

impl HuffmanTree {
    pub fn new(freq_table: &[u64; NUM_BYTES]) -> Option<HuffmanTree> {
        let mut priority_queue: BinaryHeap<Box<HuffmanNode>> = BinaryHeap::new();

        for (symbol, &frequency) in freq_table.iter().enumerate() {
            if frequency != 0 {
                let node = HuffmanNode::new(HuffmanType::new(symbol as u8, frequency));

                priority_queue.push(Box::new(node));
            }
        }

        if priority_queue.len() == 0 {
            return None;
        }

        while priority_queue.len() > 1 {
            let node1 = priority_queue.pop().unwrap();
            let node2 = priority_queue.pop().unwrap();

            let mut new_node = HuffmanNode::new(
                HuffmanType::new(0, node1.value.frequency + node2.value.frequency));

            new_node.set_right(node1);
            new_node.set_left(node2);

            priority_queue.push(Box::new(new_node));
        }

        let root_node = priority_queue.pop().unwrap();

        Some(HuffmanTree { root_node })
    }

    pub fn get_code_lengths(&self) -> Vec<(u8, u8)> {
        // Queue for breadth-first-search with depth
        let mut queue: VecDeque<(&HuffmanNode, u8)> = VecDeque::new();

        // Push the root node onto the queue
        queue.push_back((self.root_node.as_ref(), 0));

        // Raw code lengths
        let mut code_lengths: Vec<(u8, u8)> = Vec::new();

        // Do a breadth first search, keeping track of depth
        while !queue.is_empty() {
            let (node, depth) = queue.pop_front().unwrap();

            if node.is_leaf() {
                code_lengths.push((node.value.symbol, depth));
                continue;
            }

            if let Some(ref left) = node.left {
                queue.push_back((left.as_ref(), depth + 1));
            }

            if let Some(ref right) = node.right {
                queue.push_back((right.as_ref(), depth + 1));
            }
        }

        code_lengths
    }
}