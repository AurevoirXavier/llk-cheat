// --- std ---
use std::collections::{
    HashSet,
    vec_deque::VecDeque,
};
// --- custom ---
use super::Cells;

impl Cells {
    fn horizon(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        if y1 != y2 { return false; }

        let (l, r) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in l + 1..r { if self.0[y1][x] != 255 { return false; } }

        true
    }

    fn vertical(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        if x1 != x2 { return false; }

        let (u, d) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        for y in u + 1..d { if self.0[y][x1] != 255 { return false; } }

        true
    }

    fn one_turn(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        if x1 == x2 || y1 == y2 { return false; }

        if self.0[y1][x2] == 255 && self.horizon(x2, y1, x1, y1) && self.vertical(x2, y1, x2, y2) { return true; }
        if self.0[y2][x1] == 255 && self.horizon(x1, y2, x2, y2) && self.vertical(x1, y2, x1, y1) { return true; }

        false
    }

    fn two_turn(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
        for x in 0..x1 { if self.0[y1][x] == 255 && self.horizon(x, y1, x1, y1) && self.one_turn(x, y1, x2, y2) { return true; } }
        for x in x1 + 1..self.0[0].len() { if self.0[y1][x] == 255 && self.horizon(x, y1, x1, y1) && self.one_turn(x, y1, x2, y2) { return true; } }

        for y in 0..y1 { if self.0[y][x1] == 255 && self.vertical(x1, y, x1, y1) && self.one_turn(x1, y, x2, y2) { return true; } }
        for y in y1 + 1..self.0.len() { if self.0[y][x1] == 255 && self.vertical(x1, y, x1, y1) && self.one_turn(x1, y, x2, y2) { return true; } }

        false
    }

    fn groups(&self) -> VecDeque<HashSet<(usize, usize)>> {
        // --- std ---
        use std::collections::HashMap;

        let mut groups = HashMap::new();
        for y in 0..self.0.len() {
            for x in 0..self.0[0].len() {
                if self.0[y][x] == 255 { continue; }

                let group = groups.entry(self.0[y][x]).or_insert(HashSet::new());
                group.insert((x, y));
            }
        }

        groups.into_iter()
            .map(|(_, group)| group)
            .collect()
    }

    pub fn solve(mut self) -> Vec<(usize, usize)> {
        let mut result = vec![];
        let mut groups = self.groups();
        let mut retry = 0u32;

        while let Some(mut group) = groups.pop_back() {
            retry += 1;
            let v: Vec<_> = group.iter().cloned().collect();

            'pair: for i in 0..v.len() - 1 {
                for j in i + 1..v.len() {
                    let (x1, y1) = v[i];
                    let (x2, y2) = v[j];
                    if self.horizon(x1, y1, x2, y2) || self.vertical(x1, y1, x2, y2) || self.one_turn(x1, y1, x2, y2) || self.two_turn(x1, y1, x2, y2) {
                        retry = 0;

                        self.0[y1][x1] = 255;
                        self.0[y2][x2] = 255;

                        group.remove(&v[i]);
                        group.remove(&v[j]);
                        result.push(v[i]);
                        result.push(v[j]);

                        break 'pair;
                    }
                }
            }

            if !group.is_empty() { groups.push_front(group); }
        }

        result
    }
}

#[test]
fn test_h() {
    let cells = Cells(vec![vec![1, 255, 2, 255, 1]]);
    assert_eq!(false, cells.horizon(0, 0, 4, 0));

    let cells = Cells(vec![vec![1, 1]]);
    assert_eq!(true, cells.horizon(0, 0, 1, 0));

    let cells = Cells(vec![vec![1, 255, 255, 255, 1]]);
    assert_eq!(true, cells.horizon(0, 0, 4, 0));
}

#[test]
fn test_v() {
    let cells = Cells(vec![
        vec![1],
        vec![255],
        vec![2],
        vec![255],
        vec![1],
    ]);
    assert_eq!(false, cells.vertical(0, 0, 0, 4));

    let cells = Cells(vec![
        vec![1],
        vec![1],
    ]);
    assert_eq!(true, cells.vertical(0, 0, 0, 1));

    let cells = Cells(vec![
        vec![1],
        vec![255],
        vec![255],
        vec![255],
        vec![1],
    ]);
    assert_eq!(true, cells.vertical(0, 0, 0, 4));
}

#[test]
fn test_o() {
    let cells = Cells(vec![
        vec![1, 2, 2, 2, 2],
        vec![2, 2, 2, 2, 2],
        vec![2, 2, 2, 2, 2],
        vec![2, 2, 2, 2, 2],
        vec![2, 2, 2, 2, 1],
    ]);
    assert_eq!(false, cells.one_turn(0, 0, 4, 4));

    let cells = Cells(vec![
        vec![1, 255],
        vec![2, 1],
    ]);
    assert_eq!(true, cells.one_turn(0, 0, 1, 1));

    let cells = Cells(vec![
        vec![1, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
        vec![255, 255, 255, 255, 1],
    ]);
    assert_eq!(true, cells.one_turn(0, 0, 4, 4));
}

#[test]
fn test_t() {
    let cells = Cells(vec![
        vec![255, 2, 255, 255, 255],
        vec![255, 1, 2, 2, 1],
        vec![255, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
    ]);
    assert_eq!(false, cells.two_turn(1, 1, 4, 1));

    let cells = Cells(vec![
        vec![255, 255, 255, 255, 255],
        vec![255, 1, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 1],
    ]);
    assert_eq!(false, cells.two_turn(1, 1, 4, 4));

    let cells = Cells(vec![
        vec![255, 255, 255, 255, 255],
        vec![255, 1, 2, 2, 1],
        vec![255, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
        vec![255, 2, 2, 2, 2],
    ]);
    assert_eq!(true, cells.two_turn(1, 1, 4, 1));

    let cells = Cells(vec![
        vec![255, 255, 255, 255, 255],
        vec![255, 1, 255, 2, 2],
        vec![255, 2, 255, 2, 2],
        vec![255, 2, 255, 2, 2],
        vec![255, 2, 255, 255, 1],
    ]);
    assert_eq!(true, cells.two_turn(1, 1, 4, 4));
}
