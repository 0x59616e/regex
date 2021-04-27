#[derive(Debug, Copy, Clone, PartialEq)]
struct bitset<const N: usize> {
    bs: [usize; N],
}

