use std::collections::{hash_map::Entry, HashMap};

use syn::{Error, Ident, Result};

pub struct LinkedTable<T> {
    map: HashMap<Ident, Node<T>>,
    head_tail: Option<(Ident, Ident)>,
}

struct Node<T> {
    value: T,
    next: Option<Ident>,
}

impl<T> LinkedTable<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            head_tail: None,
        }
    }

    pub fn push(&mut self, key: Ident, item: T) -> Result<()> {
        match self.map.entry(key.clone()) {
            Entry::Occupied(_) => {
                return Err(Error::new(
                    key.span(),
                    format!("generic parameter `{key}` has already exists"),
                ));
            }
            Entry::Vacant(vac) => {
                vac.insert(Node {
                    value: item,
                    next: None,
                });
            }
        }

        match &mut self.head_tail {
            Some((_, tail)) => {
                self.map.get_mut(&tail).unwrap().next = Some(key.clone());
                *tail = key;
            }
            None => {
                self.head_tail = Some((key.clone(), key));
            }
        }

        Ok(())
    }

    pub fn get_mut(&mut self, ident: &Ident) -> Option<&mut T> {
        self.map.get_mut(ident).map(|node| &mut node.value)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            map: self,
            next: self.head_tail.as_ref().map(|(head, _)| head),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

pub struct Iter<'a, T> {
    map: &'a LinkedTable<T>,
    next: Option<&'a Ident>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let el = &self.map.map[self.next?];
        self.next = el.next.as_ref();
        Some(&el.value)
    }
}
