use crate::re::{Regex, Op};
// use std::collections::{HashMap};

const EPSILON: usize = 256;

#[derive(Debug)]
pub struct Nfa {
    node   : Vec<Node>,
    start  : usize,
    accept : usize,
}

#[derive(Debug)]
struct Node(Option<Edge>, Option<Edge>);

#[derive(Debug)]
struct Edge {
    label   : usize,
    out     : usize,
}

impl Edge {
    fn new(label: usize, out: usize) -> Edge {
        Edge {
            label,
            out,
        }
    }
}

impl Node {
    fn insert_edge(&mut self, e: Edge) {
        match self {
            Node(None, _) => self.0 = Some(e),
            Node(_, None) => self.1 = Some(e),
            _ => panic!("Node is full, this shouldn't have happened"),
        }
    }
}

impl Nfa {
    pub fn new(re: &String) -> Nfa {
        let re = Regex::from(re);
        Nfa::build_nfa(re)
    }

    pub fn size(&self) -> usize {
        self.node.len()
    }

    fn get_start_node(&self) -> usize {
        self.start
    }

    fn get_accept_node(&self) -> usize {
        self.accept
    }

    fn get_node(&self, id: usize) -> &Node {
        self.node.get(id).unwrap()
    }

    fn insert_node(&mut self, n: Node) {
        self.node.push(n);
    }

    fn insert_edge(&mut self, from: usize, e: Edge) {
        let from = self.node.get_mut(from).unwrap();
        from.insert_edge(e);
    }

    fn add_relation(&mut self, from: usize, label: usize, to: usize) {
        self.insert_edge(from, Edge::new(label, to));
    }

    fn add_two_new_node(&mut self) -> (usize, usize) {
        let tmp = self.size();
        self.insert_node(Node(None, None));
        self.insert_node(Node(None, None));
        (tmp, tmp + 1)
    }
    
    pub fn build_nfa(re: Regex) -> Nfa {

        let re = re.get_postfix_form();
        let mut nfa = Nfa {
            node   : vec![],
            start  : 0,
            accept : 0,
        };
        let mut stack: Vec<(usize, usize)> = vec![];

        for op in re {
            if let Op::Word(c) = *op {
                let (new_start_id, new_accept_id) = nfa.add_two_new_node();
                nfa.add_relation(new_start_id, c as usize, new_accept_id);
                stack.push((new_start_id, new_accept_id));
            } else if *op == Op::Dupstar {
                let (new_start_id, new_accept_id) = nfa.add_two_new_node();
                let (old_start_id, old_accept_id) = stack.pop().unwrap();
                nfa.add_relation(old_accept_id, EPSILON, old_start_id);
                nfa.add_relation(new_start_id,  EPSILON, old_start_id);
                nfa.add_relation(old_accept_id, EPSILON, new_accept_id);
                nfa.add_relation(new_start_id,  EPSILON, new_accept_id);
                stack.push((new_start_id, new_accept_id));
            } else if *op == Op::Alter {
                let (new_start_id, new_accept_id) = nfa.add_two_new_node();
                let (n2_start, n2_accept) = stack.pop().unwrap();
                let (n1_start, n1_accept) = stack.pop().unwrap();
                nfa.add_relation(new_start_id, EPSILON, n1_start);
                nfa.add_relation(new_start_id, EPSILON, n2_start);
                nfa.add_relation(n1_accept, EPSILON, new_accept_id);
                nfa.add_relation(n2_accept, EPSILON, new_accept_id);
                stack.push((new_start_id, new_accept_id));
            } else if *op == Op::Concat {
                let (n2_start, n2_accept) = stack.pop().unwrap();
                let (n1_start, n1_accept) = stack.pop().unwrap();
                nfa.add_relation(n1_accept, EPSILON, n2_start);
                stack.push((n1_start, n2_accept));
            }
        }
        let (start, accept) = stack.pop().unwrap();
        assert_eq!(0, stack.len());

        // TODO: support unlimited size
        assert!(nfa.size() <= 64);

        nfa.start = start;
        nfa.accept = accept;
        nfa
    }
    
    fn get_start_state(&self) -> usize {
        self.step_epsilon(1 << self.get_start_node())
    }

    fn is_accept_state(&self, set: usize) -> bool {
        set & (1 << self.get_accept_node()) != 0
    }

    // magic
    fn remove_overlap_range(&self, mut range: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        range.sort_unstable_by(|a, b| {
            if a.0 != b.0 {
                a.0.cmp(&b.0)
            } else {
                b.1.cmp(&a.1)
            }
        });
        range.dedup_by(|a, b| {
            a.0 == b.0
        });

        range
    }
    // TODO: change usize to bitset ?
    // TODO: use HashMap to cache the result
    pub fn partial_match(&self, s: &String) -> Vec<(usize, usize)> {
        let mut state: Vec<(usize, usize)> = vec![];
        let mut res: Vec<(usize, usize)> = vec![];
        let start_set = self.get_start_state();

        for (i, c) in s.as_bytes().to_vec().into_iter().enumerate() {
            let mut new_state: Vec<(usize, usize)> = vec![];
            state.push((i, start_set));
            
            for (from, set) in state.into_iter() {
                let new_set = self.step(set, c);
                if new_set != 0 {
                    new_state.push((from, new_set));
                    if self.is_accept_state(new_set) {
                        res.push((from, i));
                        break;
                    }
                }
            }

            state = new_state;
        }
        self.remove_overlap_range(res)
    }
    pub fn step(&self, from: usize, c: u8) -> usize {
        let mut to = 0usize;

        for i in (0..64).filter(move |x| ((from >> x) & 1 != 0)) {
            let node = self.get_node(i);

            if let Some(e) = &node.0 {
                if e.label == c as usize {
                    to |= 1 << e.out;
                }
            }

            if let Some(e) = &node.1 {
                if e.label == c as usize {
                    to |= 1 << e.out;
                }
            }
        }
        self.step_epsilon(to)
    }

    pub fn step_epsilon(&self, from: usize) -> usize {
        let mut to = from;
        let mut skip = 0usize;
        loop {
            let old = to;
            to = 0;
            for i in (0..64).filter(move |x| ((old >> x) & 1) != 0 && ((skip >> x) & 1) == 0) {
                let mut useful = false;
                let node = self.get_node(i);
                let e1 = &node.0;
                let e2 = &node.1;
                
                if let Some(e1) = e1 {
                    if e1.label == EPSILON {
                        to |= 1 << e1.out;
                    } else {
                        useful = true;
                    }
                }

                if let Some(e2) = e2 {
                    if e2.label == EPSILON {
                        to |= 1 << e2.out;
                    } else {
                        useful = true;
                    }
                }

                useful |= self.get_accept_node() == i;

                if useful {
                    to |= 1 << i;
                }
            }
            
            to |= skip;

            if old == to {
                break;
            }

            skip = old & to;
        }
        to
    }
}


#[test]
fn test_4() {
    let s = String::from("a*|b*");
    let regex = Regex::from(&s).build_postfix_form();
    let nfa = Nfa::build_nfa(regex);
    let set = nfa.step_epsilon(1 << nfa.get_start_node());
    assert_eq!(set, 529);
    let set = nfa.step(set, b'a');
    assert_eq!(set, 513);
    let set = nfa.step(set, b'a');
    assert_eq!(set, 513);
    let set = nfa.step(set, b'b');
    assert_eq!(set, 0);
}

#[allow(dead_code)]
fn test_5() {
    // We can't handle this case right now
    // cause it will be transformed into
    // a NFA that contains epsilon loop.
    let _s = String::from("(b*|c*)*|(a*d)");
}

#[test]
fn test_6() {
    let s = String::from("(b|c)*|(a|d)*");
    let regex = Regex::from(&s).build_postfix_form();
    let nfa = Nfa::build_nfa(regex);
    let set = nfa.step_epsilon(1 << nfa.get_start_node());
    assert_eq!(set, 132357);
    let set = nfa.step(set, b'a');
    assert_eq!(set, 132352);
    let set = nfa.step(set, b'd');
    assert_eq!(set, 132352);
    let set = nfa.step(set, b'b');
    assert_eq!(set, 0);
}

#[test]
fn test_7() {
    let s = String::from("a*b*c*");
    let regex = Regex::from(&s).build_postfix_form();
    let nfa = Nfa::build_nfa(regex);
    let set = nfa.step_epsilon(1 << nfa.get_start_node());
    assert_eq!(set, 2321);
    let set = nfa.step(set, b'a');
    assert_eq!(set, 2321);
    let set = nfa.step(set, b'a');
    assert_eq!(set, 2321);
    let set = nfa.step(set, b'b');
    assert_eq!(set, 2320);
    let set = nfa.step(set, b'c');
    assert_eq!(set, 2304);
    let set = nfa.step(set, b'd');
    assert_eq!(set, 0);
}

#[test]
fn test_8() {
    let re = String::from("abb*");
    let regex = Regex::from(&re).build_postfix_form();
    let nfa = Nfa::build_nfa(regex);
    let set = nfa.step_epsilon(1 << nfa.get_start_node());
    assert_eq!(set, 1);
    let set = nfa.step(set, b'a');
    assert_eq!(set, 4);
}

#[test]
fn test_9() {
    let re = String::from("a*b*c*");
    let regex = Regex::from(&re).build_postfix_form();
    let nfa = Nfa::build_nfa(regex);
    let content = "dadaabbcadbdcd".to_string();
    let range = nfa.partial_match(&content);
    assert_eq!(range, vec![(1, 1),
                            (3, 7),
                            (8, 8),
                            (10, 10),
                            (12, 12)]);
}

#[test]
fn test_10() {
    let re = String::from("ab*a*");
    let regex = Regex::from(&re).build_postfix_form();
    let nfa = Nfa::build_nfa(regex);
    let content = "ababababa".to_string();
    let range = nfa.partial_match(&content);
    assert_eq!(range, vec![(0, 2),
                            (4, 6),
                            (8, 8)])
}




