use std::ops::Index;

trait IndexLike {
  fn to_index(&self, max: &Self) -> usize;
  fn from_index(i: usize, max: &Self) -> Self;
  fn max_index(&self) -> usize;
}

trait Blank {
  fn blank() -> Self;
}

#[derive(Clone)]
struct Point2d {
  x: usize,
  y: usize,
}

impl Point2d {
  fn new(x: usize, y: usize) -> Point2d {
    Point2d{x: x, y: y}
  }
}

impl IndexLike for Point2d {
  fn to_index(&self, max: &Point2d) -> usize {
    if self.x > max.x || self.y > max.y {
      panic!("Point out of bounds");
    }
    (self.x % (max.x)) + (max.x) * (self.y % (max.y))
  }
  fn from_index(idx: usize, max: &Point2d) -> Point2d {
    Point2d::new(idx % (max.x - 1), idx / (max.x - 1))
  }
  fn max_index(&self) -> usize {
    (self.x) * (self.y) - 1
  }
}

impl Blank for char {
  fn blank() -> char {
    ' '
  }
}

struct Grid<K, V>
  where K : IndexLike + Clone,
        V : Blank + Clone
{
  cells : Vec<V>,
  max : K
}

struct GridIndices<'a, K : 'a, V : 'a>
  where K : IndexLike + Clone,
        V : Blank + Clone
{
  grid : &'a Grid<K, V>,
  current : K,
}

impl<K, V> Index<K> for Grid<K, V>
  where K : IndexLike + Clone,
        V : Blank + Clone
{
  type Output = V;
  fn index<'a>(&'a self, k: K) -> &'a V {
    &self.cells[k.to_index(&self.max)]
  }
}

impl<'a, K, V> Iterator for GridIndices<'a, K, V>
  where K : IndexLike + Clone,
        V : Blank + Clone
{
  type Item = K;
  fn next(&mut self) -> Option<K> {
    let max = self.grid.max.clone();
    let old = self.current.clone();
    let idx = old.to_index(&max);
    if idx > max.max_index() {
      None
    } else {
      self.current = K::from_index(idx + 1, &max);
      Some(old)
    }
  }
}

impl<K, V> Grid<K, V> 
  where K : IndexLike + Clone,
        V : Blank + Clone
{
  fn empty(max: &K) -> Grid<K, V> {
    let s = max.max_index();
    let cells : Vec<V> = vec![V::blank()].iter().cycle().take(s).cloned().collect();
    Grid {
      cells: cells,
      max: max.clone(),
    }
  }

  fn from_cell_iter(iter: &mut Iterator<Item=V>, max: K) -> Option<Grid<K, V>> 
  {
    let expected_len = max.max_index() + 1;
    let cells : Vec<V> = iter.collect();
    if expected_len == cells.len() {
      Some(Grid {
        cells: cells,
        max: max
      })
    } else {
      None
    }
  }
  fn keys<'a>(&'a self) -> GridIndices<'a, K, V> {
    GridIndices {
      grid: self,
      current: K::from_index(0, &self.max)
    }
  }

  fn fmap_mut<T, F>(&mut self, k: &K, f: F) -> T
    where F: FnOnce(&mut V) -> T {
    f(&mut self.cells[k.to_index(&self.max)])
  }

  fn fmap<T, F>(&self, k: &K, f: F) -> T
    where F: FnOnce(&V) -> T {
    f(&self.cells[k.to_index(&self.max)])
  }

  fn map<T, F>(&self, f: F) -> Grid<K, T>
    where F: Fn(&V) -> T,
          T: Clone + Blank {
    let new_cells = self.cells.iter().map(f).collect();
    Grid{
      cells: new_cells,
      max: self.max.clone()
    }
  }
}

fn parse_grid2d(s: &str) -> Result<Grid<Point2d, char>, String> {
  let s = s.trim_right();
  match s.chars().position(|c: char| c == '\n') {
    None => Err("Need at least two lines.".to_string()),
    Some(line_len) => {
      let line_count = s.lines().count();
      for (num, line) in s.lines().enumerate() {
        let this_line_len = line.chars().count();
        if this_line_len != line_len {
          return Err(format!("Line {} is {} chars long instead of {}.", num + 1, this_line_len, line_len));
        }
      }
      match Grid::from_cell_iter(&mut s.chars().filter(|c: &char| c.clone() != '\n'), Point2d::new(line_len, line_count)) {
        Some(g) => Ok(g),
        None => Err("Unknown error creating grid.".to_string())
      }
    }
  }
}

fn print_grid2d(grid: &Grid<Point2d, char>) -> String {
  let mut out = String::with_capacity(grid.max.max_index());
  for y in (0..grid.max.y) {
    for x in (0..grid.max.x) {
      grid.fmap(&Point2d::new(x, y), |c: &char| out.push(c.clone()));
    }
    out.push('\n');
  }
  out
}


fn main() {
  let maze =  "  ###########\n\
               # ###########\n\
               #           #\n\
               # ### ##### #\n\
               ### #   # # #\n\
               ###   # # # #\n\
               #######   # #\n\
               ########### #";
  let gmaze = parse_grid2d(maze).unwrap();
  println!("grid = \n{}", print_grid2d(&gmaze));
}
