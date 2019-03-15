// --- custom ---
use super::Cell;

fn horizon_detect(c1: &Cell, c2: &Cell) -> bool {
    if c1.y == c2.y {
        let (l, r) = if c1.x < c2.x { (c1, c2) } else { (c2, c1) };
        if l.e == r.x - l.x - 1 { return true; }
    }

    false
}

fn vertical_detect(c1: &Cell, c2: &Cell) -> bool {
    if c1.x == c2.x {
        let (t, b) = if c1.y < c2.y { (c1, c2) } else { (c2, c1) };
        if t.s == b.y - t.y - 1 { return true; }
    }

    false
}

fn turn_once_detect(c1: &Cell, c2: &Cell) -> bool {
    let (l, r) = if c1.x < c2.x { (c1, c2) } else { (c2, c1) };
    let h_d = r.x - l.x;

    if l.y > r.y {
        let v_d = r.y - l.y;

        if l.e == h_d && r.n == v_d { return true; }
        if l.s == v_d && r.w == h_d { return true; }
    } else {
        let v_d = l.y - r.y;

        if l.n == v_d && r.w == h_d { return true; }
        if l.e == h_d && r.s == v_d { return true; }
    }

    false
}

fn turn_twice_detect(c1: &Cell, c2: &Cell, cells: &Vec<Vec<Cell>>) -> bool {
    {
        {
            let x = c1.x + c1.e;
            let c3 = &cells[c1.y as usize][x as usize];
            if turn_once_detect(c2, c3) { return true; }
        }
        {
            let x = c1.x - c1.w;
            let c3 = &cells[c1.y as usize][x as usize];
            if turn_once_detect(c2, c3) { return true; }
        }
        {
            let y = c1.y + c1.s;
            let c3 = &cells[y as usize][c1.x as usize];
            if turn_once_detect(c2, c3) { return true; }
        }
        {
            let y = c1.y - c1.n;
            let c3 = &cells[y as usize][c1.x as usize];
            if turn_once_detect(c2, c3) { return true; }
        }
    }
    {
        {
            let x = c2.x + c2.e;
            let c3 = &cells[c2.y as usize][x as usize];
            if turn_once_detect(c1, c3) { return true; }
        }
        {
            let x = c2.x - c2.w;
            let c3 = &cells[c2.y as usize][x as usize];
            if turn_once_detect(c1, c3) { return true; }
        }
        {
            let y = c2.y + c2.s;
            let c3 = &cells[y as usize][c2.x as usize];
            if turn_once_detect(c1, c3) { return true; }
        }
        {
            let y = c2.y - c2.n;
            let c3 = &cells[y as usize][c2.x as usize];
            if turn_once_detect(c1, c3) { return true; }
        }
    }

    false
}

fn pair(c1: &Cell, c2: &Cell, cells: &Vec<Vec<Cell>>) -> bool { horizon_detect(c1, c2) || vertical_detect(c1, c2) || turn_once_detect(c1, c2) || turn_twice_detect(c1, c2, cells) }

fn groups(cells: &Vec<Vec<Cell>>) -> Vec<Vec<&Cell>> {
    // --- std ---
    use std::collections::HashMap;

    let mut groups = HashMap::new();
    for row in cells {
        for cell in row {
            if cell.v == 255 { continue; }

            let pair = groups.entry(cell.v).or_insert(vec![]);
            pair.push(cell);
        }
    }

    groups.into_iter()
        .map(|(_, v)| v)
        .collect()
}

pub fn solve(mut cells: Vec<Vec<Cell>>) -> Vec<Cell> {
    let mut groups = groups(&cells);
    while !groups.is_empty() {
        for i in 0..groups.len() {
            let len = groups[i].len();
            for j in 0..len {
                if groups[i][j].v == 255 { continue; }
                for k in 0..len {
                    if j == k || groups[i][k].v == 255 { continue; }

                    if pair(groups[i][j], groups[i][k], cells) {}
                }
            }
        }
    }

    vec![]
}
