use prusti_contracts::*;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

#[extern_spec(std::mem)]
#[ensures(snap(dest) === src)]
#[ensures(result === old(snap(dest)))]
fn replace<T>(dest: &mut T, src: T) -> T;

// Specs for std::option::Option<T>::unwrap(self) (and others) can be found here (work in progress):
// https://github.com/viperproject/prusti-dev/pull/1249/files#diff-bccda07f8a48357687e26408251041072c7470c188092fb58439de39974bdab5R47-R49

impl<T> List<T> where T: PartialEq + Copy {
    #[pure]
    pub fn len(&self) -> usize {
        link_len(&self.head)
    }

    #[pure]
    pub fn is_empty(&self) -> bool {
        matches!(self.head, None)
    }

    #[ensures(result.len() == 0)]
    pub fn new() -> Self {
        List { head: None }
    }

    #[pure]
    #[requires(index < self.len())]
    pub fn lookup(&self, index: usize) -> &T {
        link_lookup(&self.head, index)
    }

    #[pure]
    pub fn contains(&self, value: T) -> bool {
        link_contains(&self.head, value)
    }

    #[ensures(self.len() == old(self.len()) + 1)]
    #[ensures(snap(self.lookup(0)) === elem)]
    #[ensures(forall(|i: usize| (i < old(self.len())) ==>
                 old(self.lookup(i)) === self.lookup(i + 1)))]
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    #[ensures(result.len() == old(self.len()) + 1)]
    pub fn union(mut self, elem: T) -> Self {
        self.push(elem);
        self
    }

    predicate!(
        pub fn is_union(&self, subset: &List<T>, elem: T) -> bool {
            self.len() == subset.len() + 1
            && forall(|i: usize| (i < subset.len()) ==>
            subset.lookup(i) === self.lookup(i + 1))
            && self.contains(elem)
            && *self.lookup(0) == elem
        }
    );
    

    #[ensures(result.len() == old(self.len()) + 1)]
    #[ensures(result.contains(elem))]
    #[ensures(snap(result.lookup(0)) === elem)]
    #[ensures(forall(|i: usize| (i < old(self.len())) ==>
                 old(self.lookup(i)) === result.lookup(i + 1)))]
    #[ensures(result.is_union(&snap(&self), elem))]
    pub fn add(self, elem: T) -> Self {
        let new_node = Box::new(Node {
            elem,
            next: self.head,
        });
        List { head: Some(new_node) }
    }

    // #[pure]
    // pub fn equals(&self, other: &List<T>, n: usize) -> bool {
    //     link_eq(&self.head, &other.head, n)
    // }

    predicate! {
        // two-state predicate to check if the head of a list was correctly removed
        fn head_removed(&self, prev: &Self) -> bool {
            self.len() == prev.len() - 1 // The length will decrease by 1
            && forall(|i: usize| // Every element will be shifted forwards by one
                (1 <= i && i < prev.len())
                    ==> prev.lookup(i) === self.lookup(i - 1))
        }
    }

    #[pure]
    pub fn contains_all(&self, other: &List<T>) -> bool {
        link_contains_all(&self.head, &other.head)
    }

    #[ensures(old(self.is_empty()) ==>
        result.is_none() &&
        self.is_empty()
    )]
    #[ensures(!old(self.is_empty()) ==>
        self.head_removed(&old(snap(self))) &&
        result === Some(snap(old(snap(self)).lookup(0)))
    )]
    pub fn try_pop(&mut self) -> Option<T> {
        match self.head.take() {
            None => None,
            Some(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }

    #[requires(!self.is_empty())]
    #[ensures(self.head_removed(&old(snap(self))))]
    #[ensures(result === old(snap(self)).lookup(0))]
    pub fn pop(&mut self) -> T {
        self.try_pop().unwrap()
    }

    #[pure]
    #[requires(!self.is_empty())]
    pub fn peek(&self) -> &T {
        self.lookup(0)
    }

    #[trusted] // required due to unsupported reference in enum
    #[requires(!self.is_empty())]
    #[ensures(snap(result) === old(snap(self.peek())))]
    #[after_expiry(
        old(self.len()) === self.len() // (1. condition)
        && forall(|i: usize| 1 <= i && i < self.len() // (2. condition)
            ==> old(snap(self.lookup(i))) === snap(self.lookup(i)))
        && snap(self.peek()) === before_expiry(snap(result)) // (3. condition)
    )]
    pub fn peek_mut(&mut self) -> &mut T {
        // This does not work in Prusti at the moment:
        // "&mut self.head" has type "&mut Option<T>"
        // this gets auto-dereferenced by Rust into type: "Option<&mut T>"
        // this then gets matched to "Some(node: &mut T)"
        // References in enums are not yet supported, so this cannot be verified by Prusti
        if let Some(node) = &mut self.head {
            &mut node.elem
        } else {
            unreachable!()
        }
        // ...
    }

}

#[pure]
#[requires(index < link_len(link))]
fn link_lookup<T>(link: &Link<T>, index: usize) -> &T {
    match link {
        Some(node) => {
            if index == 0 {
                &node.elem
            } else {
                link_lookup(&node.next, index - 1)
            }
        }
        None => unreachable!(),
    }
}

#[pure]
fn link_contains<T>(link: &Link<T>, value: T) -> bool 
where T: PartialEq + Copy {
    match link {
        Some(node) => {
            if node.elem == value {
                true
            } else {
                link_contains(&node.next, value)
            }
        }
        None => false,
    }
}

#[pure]
#[ensures(result === forall(|elem: T| link_contains(other, elem) ==> link_contains(link, elem)))]
fn link_contains_all<T>(link: &Link<T>, other: &Link<T>) -> bool 
where T: PartialEq + Copy {
    match other {
        Some(node) => {
            if !link_contains(link, node.elem) {
                false
            } else {
                link_contains_all(link, &node.next)
            }
        }
        None => true,
    }
}

#[pure]
fn link_len<T>(link: &Link<T>) -> usize {
    match link {
        None => 0,
        Some(node) => 1 + link_len(&node.next),
    }
}

#[pure]
fn link_eq<T>(link1: &Link<T>, link2: &Link<T>, n: usize) -> bool 
where T: PartialEq {
    if link_len(link1) != n {
        return false;
    } else if link_len(link2) != n {
        return false;
    } else {
        match link1 {
            Some(node1) => {
                match link2 {
                    Some(node2) => {
                        if node1.elem == node2.elem {
                            link_eq(&node1.next, &node2.next, n-1)
                        } else {
                            false
                        }
                    }
                    None => false,
                }
            }
            None => match link2 {
                Some(_) => false,
                None => true,
            },
        }
    }
    
}



#[cfg(test)]
mod prusti_tests {
    use crate::types::messaging::Packet;

    use super::*;

    #[test]
    fn _test_contains() {
        
        let mut list = List::<Packet>::new();
        list.push(Packet { seq: 0, data: 1 });
        list.push(Packet { seq: 1, data: 1 });
        prusti_assert!(list.contains(Packet { seq: 0, data: 1 }));
        prusti_assert!(list.contains(Packet { seq: 1, data: 1 }));
        prusti_assert!(!list.contains(Packet { seq: 3, data: 1 }));
    }
    #[test]
    fn _test_contains_all() {
        let mut l = List::<u8>::new();
        for i in 0..10 {
            l.push(i);
        }
        let mut even = List::<u8>::new();
        for i in 0..10 {
            if i % 2 == 0 {
                even.push(i);
            }
        }
        prusti_assert!(l.contains_all(even));
    }

    #[test]
    fn test_equals() {
        let mut l1 = List::<u8>::new();
        let mut l2 = List::<u8>::new();
        let mut l3 = List::<u8>::new();
        for i in 0..10 {
            l1.push(i);
            l2.push(i);
            l3.push(i);
        }
        l3.push(22);
        prusti_assert!(l1.equals(&l2));
        prusti_assert!(!l1.equals(&l3));
        l3.pop();
        prusti_assert!(l1.equals(&l3));

    }

}