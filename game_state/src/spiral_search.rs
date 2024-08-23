enum GoinTo {
    Right,
    Down,
    Left,
    Up,
}
pub struct SpiralSearch {
    center: (i32, i32),
    x: i32,
    y: i32,
    level: i32,
    state: GoinTo,
}

impl SpiralSearch {
    pub fn new(center: (i32, i32)) -> Self {
        SpiralSearch {
            center,
            x: 0,
            y: 0,
            level: 0,
            state: GoinTo::Right,
        }
    }

    fn next(&mut self) -> (i32, i32) {
        let result = (self.x + self.center.0, self.y + self.center.1);
        match self.state {
            GoinTo::Right => {
                if self.x >= self.level + 1 {
                    self.state = GoinTo::Down;
                    self.level += 1;
                    self.y += 1;
                } else {
                    self.x += 1;
                }
            }
            GoinTo::Down => {
                if self.y >= self.level {
                    self.state = GoinTo::Left;
                    self.x -= 1;
                } else {
                    self.y += 1;
                }
            }
            GoinTo::Left => {
                if self.x <= -self.level {
                    self.state = GoinTo::Up;
                    self.y -= 1;
                } else {
                    self.x -= 1;
                }
            }
            GoinTo::Up => {
                if self.y <= -self.level {
                    self.state = GoinTo::Right;
                    self.x += 1;
                } else {
                    self.y -= 1;
                }
            }
        }
        return result;
    }
}

impl Iterator for SpiralSearch {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}

#[cfg(test)]
mod test {
    use super::SpiralSearch;

    #[test]
    fn test_search() {
        let spiral = SpiralSearch::new((0, 0));
        let v = spiral.take(4).collect::<Vec<_>>();
        assert_eq!(v, vec![(0, 0), (1, 0), (1, 1), (0, 1)]);
    }
    #[test]
    fn total_spin() {
        let spiral = SpiralSearch::new((0, 0));
        let v = spiral.take(9).collect::<Vec<_>>();
        assert_eq!(
            v,
            vec![
                (0, 0),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
            ]
        );
    }
}
