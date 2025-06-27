use std::collections::HashSet;

use crate::lr0::Lr0Item;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Production {
    pub index: Option<usize>,
    pub driver: char,
    pub body: Vec<char>,
}

impl Production {
    pub fn new(driver: char, body: Vec<char>) -> Self {
        Production {
            index: None,
            driver,
            body,
        }
    }

    pub fn as_lr0_item<'a>(&'a self) -> Lr0Item<'a> {
        Lr0Item::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct FirstFollowSet {
    pub first: HashSet<char>,
    pub follow: HashSet<char>,
    pub nullable: bool,
}

impl FirstFollowSet {
    pub fn new() -> Self {
        FirstFollowSet {
            first: HashSet::new(),
            follow: HashSet::new(),
            nullable: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Shift(usize),
    Reduce(usize),
    Acc,
    Goto(usize),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Shift(to) => write!(f, "s{}", to),
            Action::Reduce(prod_index) => write!(f, "r{}", prod_index + 1), // 1 based
            Action::Goto(to) => write!(f, "{}", to),
            Action::Acc => write!(f, "acc"),
        }
    }
}
