use std::rc::{Rc};
use std::fmt::{Debug};

type Symbol = i64;

#[derive(Clone, Debug)]
enum ContextError {
    AlreadyExists,
}

type ContextResult<T> = Result<T, ContextError>;

#[derive(Clone, Debug)]
struct SymbolSet {
    values: Vec<Symbol>,
    numeric: bool,
}

impl SymbolSet {
    fn named<I>(v: I) -> Self where I: Iterator<Item=Symbol> {
        SymbolSet {
            values: v.collect(),
            numeric: false,
        }
    }
    fn numeric<I>(v: I) -> Self where I: Iterator<Item=Symbol> {
        SymbolSet {
            values: v.collect(),
            numeric: true,
        }
    }
}

#[derive(Clone, Debug)]
struct Map {
    domains: Vec<SymbolSet>,
    variables: Vec<Rc<Variable>>,
}

#[derive(Clone, Debug)]
struct Variable {
    name: String,
    number: usize,
    range: SymbolSet,
}

impl Variable {
    fn new(name: String, number: usize, range: SymbolSet) -> Self {
        Variable {
            name: name,
            number: number,
            range: range,
        }
    }
}

#[derive(Clone, Debug)]
struct Context {
    max_symbolic: i64,
    symbolic_names: Vec<String>,
    variables: Vec<Rc<Variable>>,
}

impl Context {
    fn new() -> Self {
        Context {
            max_symbolic: 1,
            symbolic_names: vec!["null".to_owned()],
            variables: Vec::new(),
        }
    }

    fn get_symbol(&mut self, name: &str) -> Symbol {
        let idx;
        {
            idx = self.symbolic_names.iter().position(|ref n| *n == name);
        }
        match idx {
            Some(idx) => idx as Symbol,
            None => {
                let owned = name.to_owned();
                let symbol = self.max_symbolic;
                self.max_symbolic += 1;
                self.symbolic_names.push(owned);
                symbol as Symbol
            }
        }
    }

    fn new_variable(&mut self, name: &str, range: SymbolSet) -> ContextResult<Rc<Variable>> {
        let idx;
        {
            idx = self.variables.iter().position(|ref v| v.name == name);
        }
        match idx {
            Some(_) => Err(ContextError::AlreadyExists),
            None => {
                let name = name.to_owned();
                let variable = Rc::new(Variable::new(name, self.variables.len(), range));
                self.variables.push(variable);
                Ok(self.variables[self.variables.len() - 1].clone())
            }
        }
    }

    fn new_map<I>(&mut self, name: &str, size: usize, range: SymbolSet, domains: I) -> ContextResult<Map> where I: Iterator<Item=SymbolSet> {
        let vs: ContextResult<Vec<Rc<Variable>>> = (0usize..size).map(|idx|
                self.new_variable(&format!("{}[{}]", name, idx), range.clone()))
            .collect();
        vs.map(|vs: Vec<Rc<Variable>>|
                Map {
                    domains: domains.collect(),
                    variables: vs,
                }
            )
    }
}

#[derive(Clone, Debug)]
enum Instruction {
    NextIndex{
        map: usize,
        domain: usize,
        end: isize,
    },
    Push(Symbol),
    Pop,
    Jump(isize),
    Debug,
}

#[derive(Clone, Debug)]
struct Program {
    instructions: Vec<Instruction>
}

impl Program {
    fn exec<E>(&self, env: E) where E: Env + Debug {
        let mut pc = 0usize;
        let mut stack: Vec<Symbol> = Vec::new();
        let inst = &self.instructions;
        loop {
            match inst[pc] {
                Instruction::NextIndex {
                    map: map,
                    domain: domain,
                    end: end,
                } => {
                    let map = env.get_map(map);
                    let ref domain = map.domains[domain];
                    let stack_len = stack.len();
                    let index = stack[stack_len - 2];
                    match domain.values.get(index as usize) {
                        Some(sym) => {
                            stack[stack_len - 2] += 1;
                            stack[stack_len - 1] = sym.clone();
                        },
                        None => {
                            pc = (pc as isize + end) as usize;
                        }
                    }
                },
                Instruction::Push(sym) => {
                    stack.push(sym);
                },
                Instruction::Pop => {
                    stack.pop().expect("Pop instruction on empty stack");
                },
                Instruction::Debug => {
                    print!("env: {:?}\n", env);
                },
                Instruction::Jump(pc_adjust) => {
                    pc = (pc as isize + pc_adjust) as usize;
                },
            }
        }
    }
}

trait Env {
    fn get(&self, varnum: usize) -> Symbol;
    fn get_map(&self, mapnum: usize) -> Map;
    fn set(&mut self, varnum: usize, val: isize);
    fn get_control(&self, symbol: Symbol) -> bool;
}

#[derive(Clone, Debug)]
struct NormalEnv {
    values: Vec<Symbol>,
    maps: Vec<Map>,
}

fn main() {
    let mut ctxt = Context::new();
    let X = ctxt.get_symbol("X");
    let O = ctxt.get_symbol("O");
    let blank = ctxt.get_symbol("_");

    let player = ctxt.new_variable("player", SymbolSet::named([X, O].iter().cloned()));
    let winner = ctxt.new_variable("winner", SymbolSet::named([blank, X, O].iter().cloned()));
    let board = ctxt.new_map("board", 9, SymbolSet::named([blank, X, O].iter().cloned()),
        [SymbolSet::numeric([0, 1, 2].iter().cloned()),
         SymbolSet::numeric([0, 1, 2].iter().cloned())].iter().cloned());
}
