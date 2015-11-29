#[macro_use]
extern crate capnp;
extern crate mio;
extern crate clap;

pub mod common_capnp {
  include!(concat!(env!("OUT_DIR"), "/common_capnp.rs"));
}

pub mod update_capnp {
  include!(concat!(env!("OUT_DIR"), "/update_capnp.rs"));
}

pub mod command_capnp {
  include!(concat!(env!("OUT_DIR"), "/command_capnp.rs"));
}

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap};
use std::collections::{BinaryHeap};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;

const COMMAND: mio::Token = mio::Token(0);

fn main() {
    use clap::{Arg, App};
    let matches = App::new("snowglobe-server")
        .version("0.1")
        .author("Kyle R. Zentner <zentner.kyle@gmail.com>")
        .about("Strategy game.")
        .arg(Arg::with_name("address")
            .short("a")
            .long("addresss")
            .value_name("address")
            .help("Sets the address to bind to")
            .takes_value(true))
        .get_matches();

    let address = matches.value_of("address").unwrap_or("127.0.0.1:4410");

    println!("Starting server at {}", address);

    let addr = address.parse().unwrap();

    let server = mio::udp::UdpSocket::bound(&addr).unwrap();

    let mut event_loop = mio::EventLoop::new().unwrap();

    event_loop.register(&server, COMMAND).unwrap();

    let mut handler = start_simulation(&mut event_loop, server);

    event_loop.run(&mut handler).unwrap();
}

#[derive(Clone)]
enum SimulationEvent {
    Time(u64),
}

#[derive(Debug)]
enum ServerError {
    Io(String, std::io::Error),
    Intermittent,
    Capnp(capnp::Error),
    Parse(String)
}

type ServerResult<R> = Result<R, ServerError>;

fn start_simulation(event_loop: &mut mio::EventLoop<ServerHandler>, socket: mio::udp::UdpSocket) -> ServerHandler {
    let sender = event_loop.channel();
    let world = Arc::new(Mutex::new(World { entities: HashMap::new() }));
    let simulation_world = world.clone();
    std::thread::spawn(move || {
        {
            let mut world = simulation_world.lock().unwrap();
            world.entities.insert(0, Entity::new([0.0, 0.0, 0.0f32], Arc::new("simple".to_owned())));
            world.entities.insert(1, Entity::new([2.0, 0.0, 0.0f32], Arc::new("simple".to_owned())));
        }

        let mut t = 0.0f32;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1));
            {
                let mut world = simulation_world.lock().unwrap();
                if let Some(e) = world.entities.get_mut(&0) {
                    e.location[0] = f32::cos(t);
                    e.location[1] = f32::sin(t);
                }
            }
            t += 0.001f32;
        }
    });
    ServerHandler::new(socket, world)
}

struct Entity {
    location: [f32; 3],
    appearance: Arc<String>
}

impl Entity {
    fn new(location: [f32; 3], appearance: Arc<String>) -> Self {
        Entity {
            location: location,
            appearance: appearance
        }
    }
}

struct World {
    entities: HashMap<u64, Entity>,
}

struct ClientState {
    position: [f32; 3]
}

struct ClientPriority {
    priority: i32,
    address: std::net::SocketAddr
}

impl PartialEq for ClientPriority {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for ClientPriority {}

impl PartialOrd for ClientPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for ClientPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

struct ServerHandler {
    socket: mio::udp::UdpSocket,
    world: Arc<Mutex<World>>,
    subscriptions: HashMap<std::net::SocketAddr, ClientState>,
    subscription_priorities: BinaryHeap<ClientPriority>
}

const BUFFER_SIZE : usize = 4096;

impl ServerHandler {
    fn new(socket: mio::udp::UdpSocket, world: Arc<Mutex<World>>) -> Self {
        ServerHandler {
            socket: socket,
            world: world,
            subscriptions: HashMap::new(),
            subscription_priorities: BinaryHeap::new()
        }
    }
}

fn parse_point(point: common_capnp::point::Reader) -> [f32; 3] {
    [point.get_x(), point.get_y(), point.get_z()]
}

fn write_point(p: &mut common_capnp::point::Builder, point: &[f32; 3]) {
    p.set_x(point[0]);
    p.set_y(point[1]);
    p.set_z(point[2]);
}

impl ServerHandler {
    fn process_command(&mut self) -> ServerResult<()> {
        use capnp::serialize;
        let mut in_buf = mio::buf::ByteBuf::mut_with_capacity(BUFFER_SIZE);
        let maybe_addr : Option<std::net::SocketAddr> = try!(self.socket.recv_from(&mut in_buf)
                                                             .map_err(|e| ServerError::Io("COMMAND".to_owned(), e)));
        let addr = try!(maybe_addr.ok_or(ServerError::Intermittent));
        let reader = try!(serialize::read_message(&mut in_buf.flip(),
                                                  ::capnp::message::ReaderOptions::new())
                          .map_err(|e| ServerError::Capnp(e)));
        let command = try!(reader.get_root::<command_capnp::command::Reader>()
                           .map_err(|e| ServerError::Capnp(e)));
        match command.which() {
            Ok(command_capnp::command::Which::Move(mov)) => {
                if let Ok(mov) = mov {
                    match self.subscriptions.entry(addr.clone()) {
                        Occupied(mut e) => {
                            e.get_mut().position = parse_point(mov);
                        },
                        Vacant(e) => {
                            e.insert(ClientState { position: parse_point(mov) });
                            self.subscription_priorities.push(ClientPriority {
                                priority: 100,
                                address: addr
                            });
                            println!("Client connected.");
                        }
                    }
                    return Ok(());
                }
                else {
                    return Err(ServerError::Parse("location missing in move command".to_owned()));
                }
            }
            _ => {
                return Err(ServerError::Parse("Unknown command".to_owned()));
            }
        }
    }
    fn send_best_update(&mut self) -> ServerResult<()> {
        use capnp::serialize;
        if let Some(mut client) = self.subscription_priorities.pop() {
            client.priority -= 1;
            let world = self.world.lock().unwrap();
            let mut message = capnp::message::Builder::new_default();
            {

                let mut client_message = message.init_root::<update_capnp::client_message::Builder>();
                let mut updates = client_message.init_updates(world.entities.len() as u32);
                let mut i = 0;
                for (id, entity) in &world.entities {
                    let mut update = updates.borrow().get(i);
                    let mut alive = update.init_entity_alive();
                    alive.set_identity(*id);
                    alive.set_appearance(entity.appearance.as_ref());
                    write_point(&mut alive.init_location(), &entity.location);
                    i += 1;
                }
            }
            let mut out_buf = mio::buf::ByteBuf::mut_with_capacity(BUFFER_SIZE);
            serialize::write_message(&mut out_buf, &message).unwrap();
            self.socket.send_to(&mut out_buf.flip(), &client.address).unwrap();
            self.subscription_priorities.push(client);
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
        return Ok(());
    }
}

fn print_error<T>(res: ServerResult<T>) {
    if let Err(e) = res {
        match e {
            ServerError::Intermittent => {
            },
            _ => {
                println!("Error: {:?}", e);
            }
        }
    }
}

impl mio::Handler for ServerHandler {
    type Timeout = ();
    type Message = SimulationEvent;

    fn ready(&mut self, event_loop: &mut mio::EventLoop<ServerHandler>, token: mio::Token, events: mio::EventSet) {
        match token {
            COMMAND => {
                if events.is_readable() {
                    print_error(self.process_command());
                }
                if events.is_writable() {
                    print_error(self.send_best_update());
                }
            },
            _ => panic!("unexpected token"),
        }
    }
}
