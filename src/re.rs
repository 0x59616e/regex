/* precedence:
 * escaped characters:  \
 * bracket expression:  []        
 * grouping:            ()
 * duplication:         * + ? {m, n}
 * concatenation
 * anchoring:           ^ $
 * alternation:         |
 */

pub struct Regex {
    infix:   Vec<Op>,
    postfix: Vec<Op>,
}

#[derive(PartialOrd)]
#[derive(PartialEq)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub enum Op {
    Leftparen,
    Rightparen,
    Alter,
    Concat,
    Dupstar,
    Word(u8),
}

struct ConcatDetector {
    state: i32,
}

impl ConcatDetector {
    fn new() -> ConcatDetector {
        ConcatDetector {
            state: 2,
        }
    }

    fn have_concat(&mut self, op: Op) -> bool {
        let mut res: bool = false;
        // magic
        match op {
            Op::Rightparen | Op::Dupstar => self.state = 1,
            Op::Alter        => self.state = 2,
            Op::Leftparen => {
                res = self.state < 2;
                self.state = 2;
            },
            _ => {
                res = self.state < 2;
                self.state >>= 1;
            },
        }
        res
    }
}

impl Regex {
    fn default() -> Regex {
        Regex {
            infix  : vec![],
            postfix: vec![],
        }
    }

    pub fn get_postfix_form(&self) -> &Vec<Op> {
        &self.postfix
    }

    // TODO: Use LR(1) parser
    pub fn build_postfix_form(self) -> Regex {
        // Shunting yard
        let mut op_stack: Vec<Op> = vec![];
        let mut result:   Vec<Op> = vec![];

        // Shunting yard algorithm
        for op in self.infix {
            if let Op::Word(_) = op {
                result.push(op);
            } else if op_stack.is_empty() || op == Op::Leftparen {
                op_stack.push(op);
            } else if op == Op::Rightparen {
                // pop until left paren
                while let Some(tmp) = op_stack.pop() {  
                    if tmp == Op::Leftparen {
                        break;
                    }
                    result.push(tmp);
                }
            } else {
                // if higher or equal precedence then pop it 
                // honestly, I still don't know what tmp in Some(&tmp) is 
                // Does it take the ownership ?
                while let Some(&tmp) = op_stack.last() {
                    if tmp < op {
                        break;
                    }
                    result.push(op_stack.pop().unwrap());
                }
                op_stack.push(op);
            }
        }

        while !op_stack.is_empty() {
            result.push(op_stack.pop().unwrap());
        }

        Regex {
            postfix: result,
            ..Regex::default()
        }
    }
}

impl From<&String> for Regex {
    fn from(s: &String) -> Regex {
        let tmp: Vec<Op> = s.as_bytes()
                            .to_vec()
                            .into_iter()
                            .map(|x| match x {
                                b'(' => Op::Leftparen,
                                b')' => Op::Rightparen,
                                b'*' => Op::Dupstar,
                                b'|' => Op::Alter,
                                _    => Op::Word(x)
                            }).collect();
        let mut res: Vec<Op> = vec![];
        let mut concat_detector = ConcatDetector::new();

        for op in tmp {
            if concat_detector.have_concat(op) {
                res.push(Op::Concat);
            }
            res.push(op);
        }

        Regex {
            infix: res,
            ..Regex::default()
        }.build_postfix_form()
    }
}

#[test]
fn test_1() {
    let s = String::from("a*|b*");
    let regex = Regex::from(&s);
    assert_eq!(regex.infix, vec![Op::Word(b'a'),
                                Op::Dupstar,
                                Op::Alter,
                                Op::Word(b'b'),
                                Op::Dupstar]);
    let regex = regex.build_postfix_form();
    assert_eq!(regex.postfix, vec![Op::Word(b'a'),
                                Op::Dupstar,
                                Op::Word(b'b'),
                                Op::Dupstar,
                                Op::Alter]);
}

#[test]
fn test_2() {
    let s = String::from("ab");
    let regex = Regex::from(&s);
    assert_eq!(regex.infix, vec![Op::Word(b'a'), Op::Concat, Op::Word(b'b')]);
    let regex = regex.build_postfix_form();
    assert_eq!(regex.postfix, vec![Op::Word(b'a'), Op::Word(b'b'), Op::Concat]);
}
#[test]
fn test_3() {
    let s = String::from("ab(b|c)*d");
    let regex = Regex::from(&s);
    assert_eq!(regex.infix, vec![Op::Word(b'a'),
                                Op::Concat, 
                                Op::Word(b'b'),
                                Op::Concat,
                                Op::Leftparen, 
                                Op::Word(b'b'),
                                Op::Alter,
                                Op::Word(b'c'),
                                Op::Rightparen,
                                Op::Dupstar,
                                Op::Concat,
                                Op::Word(b'd')
                                ]);
    let regex = regex.build_postfix_form();
    assert_eq!(regex.postfix, vec![Op::Word(b'a'),
                                Op::Word(b'b'),
                                Op::Concat,
                                Op::Word(b'b'),
                                Op::Word(b'c'),
                                Op::Alter,
                                Op::Dupstar,
                                Op::Concat,
                                Op::Word(b'd'),
                                Op::Concat,
                                ]);
}

