/// Barebones library for working with hex grids. Everything good about this
/// was derived from Amit Patel's indispensable guide.
/// https://www.redblobgames.com/grids/hexagons/

/// A traditional Hey, That's My Fish! setup is square-ish, with every even
/// row shoved a little to the right of every odd row. Amit's guide calls this
/// the "even-r" layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvenR {
    pub col: i64,
    pub row: i64,
}

impl EvenR {
    pub const fn from_cube(cube: &Cube) -> EvenR {
        EvenR {
            col: cube.x + (cube.z + (cube.z & 1)) / 2,
            row: cube.z,
        }
    }

    pub const fn in_line(&self, rhs: &EvenR) -> bool {
        Cube::from_evenr(self).in_line(&Cube::from_evenr(rhs))
    }

    pub const fn neighbors(&self) -> [EvenR; 6] {
        let cube_neighbors = Cube::from_evenr(self).neighbors();
        let mut neighbors = [EvenR { col: 0, row: 0 }; 6];
        neighbors[0] = EvenR::from_cube(&cube_neighbors[0]);
        neighbors[1] = EvenR::from_cube(&cube_neighbors[1]);
        neighbors[2] = EvenR::from_cube(&cube_neighbors[2]);
        neighbors[3] = EvenR::from_cube(&cube_neighbors[3]);
        neighbors[4] = EvenR::from_cube(&cube_neighbors[4]);
        neighbors[5] = EvenR::from_cube(&cube_neighbors[5]);
        neighbors
    }
}

/// You can imagine each hexagon as the shadow of a cube. A hexagon has six
/// sides, which correspond to a cube's six faces. If you look at a hex grid
/// this way, you get a number of nice properties that allow you to easily find
/// the hex's neighbors and elements in line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cube {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Cube {
    pub const fn from_evenr(hex: &EvenR) -> Cube {
        let x = hex.col - (hex.row + (hex.row & 1)) / 2;
        let z = hex.row;
        let y = -(x + z);
        Cube { x, y, z }
    }

    pub const fn in_line(&self, rhs: &Cube) -> bool {
        self.x == rhs.x || self.y == rhs.y || self.z == rhs.z
    }

    pub const fn neighbors(&self) -> [Cube; 6] {
        [
            Cube {
                x: self.x + 1,
                y: self.y - 1,
                z: self.z,
            },
            Cube {
                x: self.x + 1,
                y: self.y,
                z: self.z - 1,
            },
            Cube {
                x: self.x,
                y: self.y + 1,
                z: self.z - 1,
            },
            Cube {
                x: self.x - 1,
                y: self.y + 1,
                z: self.z,
            },
            Cube {
                x: self.x - 1,
                y: self.y,
                z: self.z + 1,
            },
            Cube {
                x: self.x,
                y: self.y - 1,
                z: self.z + 1,
            },
        ]
    }
}

/// Return a ray which passes through both src and dst and extends forever.
pub fn line(src: &EvenR, dst: &EvenR) -> Iterator {
    if src == dst {
        panic!("Source and destination cells cannot be the same.");
    }

    let src_cube = Cube::from_evenr(src);
    let dst_cube = Cube::from_evenr(dst);

    if !src_cube.in_line(&dst_cube) {
        panic!("source and destination hexes aren't actually in the same line");
    }

    fn get_x(c: &Cube) -> i64 {
        c.x
    }
    fn get_y(c: &Cube) -> i64 {
        c.y
    }

    let (constant_element, get_other_element, make_) = if src_cube.x == dst_cube.x {
        fn make_xy(x: i64, y: i64) -> Cube {
            Cube { x, y, z: -(x + y) }
        }
        (
            src_cube.x,
            get_y as fn(&Cube) -> i64,
            make_xy as fn(i64, i64) -> Cube,
        )
    } else if src_cube.y == dst_cube.y {
        fn make_yx(y: i64, x: i64) -> Cube {
            Cube { x, y, z: -(x + y) }
        }
        (
            src_cube.y,
            get_x as fn(&Cube) -> i64,
            make_yx as fn(i64, i64) -> Cube,
        )
    } else {
        fn make_zx(z: i64, x: i64) -> Cube {
            Cube { x, y: -(x + z), z }
        }
        (
            src_cube.z,
            get_x as fn(&Cube) -> i64,
            make_zx as fn(i64, i64) -> Cube,
        )
    };

    Iterator {
        constant: constant_element,
        get_other_element,
        _make_cube: make_,
        src: src_cube,
        cur: src_cube,
        dst: dst_cube,
    }
}

pub struct Iterator {
    constant: i64,
    get_other_element: fn(&Cube) -> i64,
    _make_cube: fn(i64, i64) -> Cube,
    src: Cube,
    cur: Cube,
    dst: Cube,
}

impl Iterator {
    fn make_cube(&self, other: i64) -> Cube {
        (self._make_cube)(self.constant, other)
    }
}

impl ::std::iter::Iterator for Iterator {
    type Item = EvenR;

    fn next(&mut self) -> Option<Self::Item> {
        let src_other = (self.get_other_element)(&self.src);
        let cur_other = (self.get_other_element)(&self.cur);
        let dst_other = (self.get_other_element)(&self.dst);

        let next_other = if src_other < dst_other {
            cur_other + 1
        } else {
            cur_other - 1
        };

        let ret = self.cur;
        self.cur = self.make_cube(next_other);
        Some(EvenR::from_cube(&ret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn some_equivalences() {
        assert_eq!(
            Cube::from_evenr(&EvenR { col: 0, row: 0 }),
            Cube { x: 0, y: 0, z: 0 }
        );
        assert_eq!(
            Cube::from_evenr(&EvenR { col: 1, row: 1 }),
            Cube { x: 0, y: -1, z: 1 }
        );
        assert_eq!(
            Cube::from_evenr(&EvenR { col: 1, row: 2 }),
            Cube { x: 0, y: -2, z: 2 }
        );
        assert_eq!(
            Cube::from_evenr(&EvenR { col: 3, row: 6 }),
            Cube { x: 0, y: -6, z: 6 }
        );
    }

    #[test]
    fn inverse_cube_hex() {
        for row in -10..10 {
            for col in -10..10 {
                assert_eq!(
                    EvenR { col: col, row: row },
                    EvenR::from_cube(&Cube::from_evenr(&EvenR { col: col, row: row }))
                );
            }
        }
    }

    #[test]
    fn inverse_hex_cube() {
        for x in -10..10 {
            for y in -10..10 {
                let z = -(x + y);
                assert_eq!(
                    Cube { x: x, y: y, z: z },
                    Cube::from_evenr(&EvenR::from_cube(&Cube { x: x, y: y, z: z }))
                );
            }
        }
    }

    #[test]
    fn cube_neighbors_distinct() {
        let c = Cube::from_evenr(&EvenR { col: 1, row: 2 });
        let neighbors = c.neighbors();
        for &x in neighbors.iter() {
            assert!(x != c);
        }
    }

    #[test]
    fn neighbors_distinct() {
        let c = EvenR { col: 1, row: 2 };
        let neighbors = c.neighbors();
        for &x in neighbors.iter() {
            assert!(x != c);
        }
    }

    #[test]
    fn neighbors_in_line() {
        let c1 = EvenR { col: 1, row: 2 };
        let c2 = EvenR { col: 2, row: 3 };
        assert!(c1.in_line(&c2));
    }

    #[test]
    fn cells_in_line() {
        let c1 = EvenR { col: 1, row: 2 };
        let c2 = EvenR { col: 3, row: 5 };
        assert!(c1.in_line(&c2));
    }

    #[test]
    fn cells_not_in_line() {
        let c1 = EvenR { col: 1, row: 2 };
        let c2 = EvenR { col: 4, row: 6 };
        assert!(!c1.in_line(&c2));
    }

    #[test]
    fn enumerate_line() {
        let c1 = EvenR { col: 0, row: 0 };
        let c2 = EvenR { col: 3, row: 6 };

        let expected_line = vec![
            EvenR { col: 0, row: 0 },
            EvenR { col: 1, row: 1 },
            EvenR { col: 1, row: 2 },
            EvenR { col: 2, row: 3 },
            EvenR { col: 2, row: 4 },
            EvenR { col: 3, row: 5 },
            EvenR { col: 3, row: 6 },
        ];

        for (actual, expected) in line(&c1, &c2).zip(expected_line) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn line_is_distinct() {
        let src = EvenR { col: 1, row: 2 };
        let dst = EvenR { col: 2, row: 2 };
        let ln: Vec<EvenR> = line(&src, &dst).take(5).collect();
        for i in 0..ln.len() {
            for j in 0..ln.len() {
                if i == j {
                    continue;
                }
                assert_ne!(ln[i], ln[j]);
            }
        }
    }
}
