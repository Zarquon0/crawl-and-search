#![allow(dead_code)]
use crate::prelude::*;
use std::cmp::Ordering;

const LEVELS: u8 = 3;
const VALID_SPECIALS: &[char; 5] = &['.', '-', '_', '/', '%'];

pub struct UrlBrancher {
    level: u8,
    branches: HashMap<char, Arc<UrlTree>>
}

impl UrlBrancher {
    pub fn new(level: u8) -> UrlBrancher { UrlBrancher { level, branches: HashMap::<char, Arc<UrlTree>>::new() } }
}

pub enum UrlTree {
    Fork(RwLock<UrlBrancher>),
    Leaf(RwLock<Vec<String>>)
}

impl UrlTree {
    pub fn root() -> UrlTree {
        let root_brancher = UrlBrancher::new(0);
        UrlTree::Fork(RwLock::new(root_brancher))
    }
    pub fn add_url(&self, url: String) {
        match self {
            UrlTree::Fork(brancher) => {
                let next_node = {
                    let slf = brancher.read();
                    let curr_char = url.chars().nth(slf.level as usize).expect("URL shorter than it should be...");
                    if !valid_url_char(curr_char) { panic!("URL contains invalid characters - will not create branch for it") }
                    match slf.branches.get(&curr_char) {
                        Some(tree) => NextNode::Node(tree.clone()),
                        None => NextNode::Nonexistent(curr_char)
                    }
                };
                match next_node {
                    NextNode::Node(tree) => tree.add_url(url),
                    NextNode::Nonexistent(curr_char) => {
                        let mut slf = brancher.write();
                        let new_node = match slf.level.cmp(&(LEVELS - 1)) {
                            Ordering::Less => {
                                let new_brancher = RwLock::new(UrlBrancher::new(slf.level + 1));
                                Arc::new(UrlTree::Fork(new_brancher))
                            },
                            Ordering::Equal => {
                                let new_bucket = RwLock::new(Vec::<String>::new());
                                Arc::new(UrlTree::Leaf(new_bucket))
                            },
                            Ordering::Greater => panic!("Brancher has higher level than LEVEL - 1")    
                        };
                        let node_clone = new_node.clone();
                        slf.branches.insert(curr_char, new_node);
                        node_clone.add_url(url);
                    }
                }
            }
            UrlTree::Leaf(bucket) => {
                println!("ADDED TO THE MAP");
                let mut slf = bucket.write();
                if !slf.contains(&url) { slf.push(url) }
            }
        }
    }
    pub fn check_url(&self, url: &String) -> bool {
        match self {
            UrlTree::Fork(brancher) => {
                let slf = brancher.read();
                let curr_char = url.chars().nth(slf.level as usize).expect("URL shorter than it should be...");
                match slf.branches.get(&curr_char) {
                    Some(tree) => tree.clone().check_url(url),
                    None => return false //Branch doesn't exist
                }
            },
            UrlTree::Leaf(bucket) => {
                let slf = bucket.read();
                slf.contains(&url)
            }
        }
    }
}

enum NextNode {
    Node(Arc<UrlTree>),
    Nonexistent(char)
}

//HELPERS
pub fn valid_url_char(ch: char) -> bool { ch.is_alphanumeric() || VALID_SPECIALS.contains(&ch) }

#[cfg(test)]
mod tests {
    use super::*;
    //UrlTree Tests
    #[test]
    fn in_tree_simple() {
        let tree = UrlTree::root();
        tree.add_url("hello.com".to_string());
        assert!(tree.check_url(&"hello.com".to_string()));
    }
    #[test]
    fn in_tree_complex() {
        let tree = UrlTree::root();
        tree.add_url("hello.com".to_string());
        tree.add_url("unrelated.com".to_string());
        tree.add_url("hallo.com".to_string());
        tree.add_url("healo.com".to_string());
        tree.add_url("help.com".to_string());
        assert!(tree.check_url(&"hello.com".to_string()));
    }
    #[test]
    fn double_add() {
        let tree = UrlTree::root();
        tree.add_url("hello.com".to_string());
        tree.add_url("hello.com".to_string());
        assert!(tree.check_url(&"hello.com".to_string()));
    }
    #[test]
    fn add_special() {
        let tree = UrlTree::root();
        tree.add_url("1/.-sf.com".to_string());
        assert!(tree.check_url(&"1/.-sf.com".to_string()));
    }
    #[test]
    #[should_panic]
    fn add_improper_special() {
        let tree = UrlTree::root();
        tree.add_url("1/}-sf.com".to_string());
    }
    #[test]
    fn not_in_tree_simple() {
        let tree = UrlTree::root();
        tree.add_url("hello.com".to_string());
        assert!(!tree.check_url(&"goodbye.com".to_string()));
    }
    #[test]
    fn not_in_tree_mid() {
        let tree = UrlTree::root();
        tree.add_url("hello.com".to_string());
        assert!(!tree.check_url(&"heap.com".to_string()));
    }
    #[test]
    fn not_in_tree_complex() {
        let tree = UrlTree::root();
        tree.add_url("hello.com".to_string());
        tree.add_url("heloo.com".to_string());
        assert!(!tree.check_url(&"hello.co".to_string()));
    }
    #[test]
    #[should_panic]
    fn short_url() {
        let tree = UrlTree::root();
        tree.add_url("12".to_string());
    }
}