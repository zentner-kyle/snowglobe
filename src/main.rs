use std::fmt;

#[derive(Clone)]
struct Space {
    width: u64,
    height: u64,
    character: Option<char>
}

impl Space {
    fn new(width: u64, height: u64, character: char) -> Self {
        Space {
            width: width,
            height: height,
            character: Some(character)
        }
    }

    fn size(&self) -> u64 {
        self.width * self.height
    }
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(c) = self.character {
            write!(f, "Space ({}x{}) with character {}",
                   self.width, self.height, c)
        } else {
            write!(f, "Space ({}x{})", self.width, self.height)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct BitGrid {
    bits: u64
}

impl BitGrid {
    fn parse(to_extract: char, to_parse: &str) -> Option<(Self, Space)> {
        let mut bits = 0;
        let mut index = 0;
        let mut line_width : Option<u64> = None;

        for line in to_parse.lines() {
            for c in line.chars() {
                if c == to_extract {
                    bits = bits | (1 << index);
                }
                index = index + 1;
            }
            if let Some(width) = line_width {
                if index % width != 0 {
                    return None;
                }
            } else {
                line_width = Some(index);
            }
            
        }

        let line_width = line_width.unwrap_or(index);

        let line_height = if line_width == 0 { 0 } else { index / line_width };

        let space = Space::new(line_width, line_height, to_extract);

        return Some((BitGrid { bits: bits }, space));
    }

    fn zero() -> Self {
        return BitGrid { bits: 0 };
    }

    fn x(&self, s: &Space) -> u8 {
        if self.bits.count_ones() != 1 {
            panic!("self for BitGrid::x() should only have one bit set");
        }
        (self.bits % s.width) as u8
    }

    fn y(&self, s: &Space) -> u8 {
        if self.bits.count_ones() != 1 {
            panic!("self for BitGrid::y() should only have one bit set");
        }
        (self.bits / s.width) as u8
    }
}

impl fmt::Display for BitGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BitGrid {{ {:#x} }}", self.bits)
    }
}

fn main() {
    let pos = "___\n\
               ___\n\
               ___\n";
    if let Some((blanks, blank_space)) = BitGrid::parse('_', pos) {
        println!("blanks = {}", blanks);
        println!("blank_space = {}", blank_space);
    }
}
