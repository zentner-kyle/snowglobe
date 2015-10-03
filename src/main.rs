extern crate serde_json;
#[allow(dead_code)]


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

#[derive(Clone, Copy)]
enum Type {
    Grid,
    Number,
    Bool,
    Null
}

#[derive(Clone, Copy)]
enum Value {
    Grid(BitGrid),
    Number(i64),
    Bool(bool),
    Null
}

struct Operator {
    input: Type,
    secondary: Type,
    output: Type,
    op: Box<Fn(Value, Value) -> Value>,
    name: &'static str,
    cost: i64
}

fn create_operator() -> Vec<Operator> {
    vec![
        Operator::new("count", 2, Type::Grid, Type::Null, Type::Number,
                      Box::new(move |x, y| {
            match (x, y) {
                (Value::Grid(ref x),
                 Value::Null) => Value::Number(x.bits.count_ones() as i64),
                 _ => Value::Null
            }
        })),
        Operator::new("even", 2, Type::Number, Type::Null, Type::Bool,
                      Box::new(move |x, y| {
            match (x, y) {
                (Value::Number(ref x),
                 Value::Null) => Value::Bool(x % 2 == 0),
                 _ => Value::Null
            }
        })),
        Operator::new("mux", 1, Type::Grid, Type::Bool, Type::Grid,
                      Box::new(move |x, y| {
            match (x, y) {
                (Value::Grid(ref x2),
                 Value::Bool(y)) => if y {
                    x.clone()
                } else {
                    Value::Grid(BitGrid::zero())
                },
                _ => Value::Null
            }
        })),
        Operator::new("not", 1, Type::Bool, Type::Null, Type::Bool,
                      Box::new(move |x, y| {
            match (x, y) {
                (Value::Bool(ref x),
                 Value::Null) => Value::Bool(!x),
                 _ => Value::Null
            }
        })),
        Operator::new("union", 1, Type::Grid, Type::Grid, Type::Grid,
                      Box::new(move |x, y| {
            match (x, y) {
                (Value::Grid(ref x),
                 Value::Grid(ref y)) =>
                    Value::Grid(BitGrid { bits: x.bits | y.bits } ),
                 _ => Value::Null
            }
        })),
        Operator::new("intersect", 1, Type::Grid, Type::Grid, Type::Grid,
                      Box::new(move |x, y| {
            match (x, y) {
                (Value::Grid(ref x),
                 Value::Grid(ref y)) =>
                    Value::Grid(BitGrid { bits: x.bits & y.bits } ),
                 _ => Value::Null
            }
        })),
    ]
}

impl Operator {
    fn new(name: &'static str, cost: i64, input: Type, secondary: Type,
           out: Type, f: Box<Fn(Value, Value) -> Value>) -> Operator
    {
        Operator {
            input: input,
            secondary: secondary,
            output: out,
            op: f,
            name: name,
            cost: cost
        }
    }
}

struct MoveTree {
    board: String,
    children: Vec<MoveTree>
}

impl MoveTree {
    fn from_json_str(json: &str) -> Option<Self> {
        Self::from_json(serde_json::from_str(json))
    }

    fn from_json_reader(reader: &mut std::io::Read) -> Option<Self> {
        Self::from_json(serde_json::from_reader(reader))
    }

    fn from_json(json: serde_json::error::Result<serde_json::Value>) ->
            Option<Self> {
        if let Ok(data) = json {
            Self::from_json_inner(&data)
        } else {
            None
        }
    }

    fn from_json_inner(data: &serde_json::Value) -> Option<Self> {
        if let Some(obj) = data.as_object() {
            match (obj.get("board"),
                   obj.get("children")) {
                (Some(&serde_json::Value::String(ref board)),
                 Some(&serde_json::Value::Array(ref children))) => {
                    let children_parsed : Vec<Self> =
                        children.iter()
                                .filter_map(Self::from_json_inner)
                                .collect();
                    if children_parsed.len() == children.len() {
                        Some(MoveTree {
                            board: board.clone(),
                            children: children_parsed
                        })
                    } else {
                        None
                    }
                },
                (Some(&serde_json::Value::String(ref board)),
                 None) => {
                    Some(MoveTree {
                        board: board.clone(),
                        children: Vec::new()
                    })
                },
                _ => {
                    None
                }
            }
        } else {
            None
        }
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
    if let Ok(mut file) = std::fs::File::open("ttt.game_tree") {
        println!("opened file");
        if let Some(tree) = MoveTree::from_json_reader(&mut file as &mut std::io::Read) {
            println!("loaded game tree!");
        }
    }

}
