#[macro_use]
extern crate glium;
extern crate image;
extern crate capnp;
extern crate mio;
extern crate clap;
extern crate time;

pub mod common_capnp {
  include!(concat!(env!("OUT_DIR"), "/common_capnp.rs"));
}

pub mod update_capnp {
  include!(concat!(env!("OUT_DIR"), "/update_capnp.rs"));
}

pub mod command_capnp {
  include!(concat!(env!("OUT_DIR"), "/command_capnp.rs"));
}

use glium::backend::glutin_backend::GlutinFacade;
use glium::{Surface};

type Display = glium::backend::glutin_backend::GlutinFacade;

use std::collections::binary_heap::{BinaryHeap};
use std::collections::{HashMap};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;

#[derive(Debug)]
enum LoadError {
    Io(String, std::io::Error),
    Glium(String),
    Image(image::ImageError)
}

type LoadResult<R> = Result<R, LoadError>;

type Mat4 = [[f32; 4]; 4];

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coords);

fn open_file(path: &str) -> LoadResult<File> {
    File::open(path)
        .map_err(|e| LoadError::Io(path.to_owned(), e))
}

fn read_file_to_string(path: &str) -> LoadResult<String> {
    let mut out_string = String::new();
    open_file(path)
        .and_then(|mut file| {
            file.read_to_string(&mut out_string)
                .map_err(|e| LoadError::Io(path.to_owned(), e))
        })
        .map(move |_| out_string)
}

fn load_shaders(display: &Display, path: &str) -> LoadResult<glium::Program> {
    let vertex_src = try!(read_file_to_string(&format!("shaders/{}.vert", path)));
    let fragment_src = try!(read_file_to_string(&format!("shaders/{}.frag", path)));
    glium::Program::from_source(display, &vertex_src, &fragment_src, None)
        .map_err(|e| LoadError::Glium(format!("glium error: {:?}", e)))
}

struct Appearance {
    shaders: glium::Program,
    shape: glium::VertexBuffer<Vertex>,
    diffuse_map: glium::texture::Texture2d,
    normal_map: Option<glium::texture::Texture2d>
}

fn load_texture(display: &Display, path: &str) -> LoadResult<glium::texture::Texture2d> {
    open_file(path)
        .and_then(|file| {
            image::load(file, image::PNG)
                .map_err(|e| LoadError::Image(e))
        })
        .and_then(|img| {
            glium::texture::Texture2d::new(display, img)
                .map_err(|e| LoadError::Glium(format!("glium error: {:?}", e)))
        })
}

impl Appearance {
    fn load(display: &Display, name: &str) -> LoadResult<Appearance> {
        let diffuse = try!(load_texture(display, &format!("assets/{}.png", name)));
        let normal = load_texture(display, &format!("assets/{}-normal.png", name)).ok();

        let shape = try!(glium::vertex::VertexBuffer::new(display, &[
            Vertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 1.0] },
            Vertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 1.0] },
            Vertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], tex_coords: [1.0, 0.0] },
        ])
            .map_err(|e| LoadError::Glium(format!("glium error: {:?}", e))));

        load_shaders(display, "simple")
            .map(|prog| {
                Appearance {
                    shaders: prog,
                    shape: shape,
                    diffuse_map: diffuse,
                    normal_map: normal,
                }
            })
    }

    fn render(&self, target: &mut glium::Frame, model: &Mat4, view: &Mat4, perspective: &Mat4, light: &[f32; 3]) -> Result<(), glium::DrawError> {
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        let normal_tex = match self.normal_map {
            Some(ref map) => map,
            None => &self.diffuse_map
        };
        target.draw(&self.shape, glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &self.shaders,
                    &uniform! { model: model.clone(), view: view.clone(), perspective: perspective.clone(),
                                u_light: light.clone(), diffuse_tex: &self.diffuse_map, normal_tex: normal_tex },
                    &params)
    }
}

#[derive(Clone)]
enum CommandEvent {
    Move([f32; 3])
}

struct AppearanceCache {
    loaded: HashMap<Arc<String>, Appearance>,
}

impl AppearanceCache {
    fn new() -> Self {
        AppearanceCache {
            loaded: HashMap::new()
        }
    }

    fn get<'a>(&'a mut self, display: &Display, name: Arc<String>) -> LoadResult<&'a Appearance> {
        match self.loaded.entry(name.clone()) {
            Occupied(e) => Ok(e.into_mut()),
            Vacant(e) => {
                let to_insert = try!(Appearance::load(display, name.as_ref()));
                Ok(e.insert(to_insert))
            }
        }
    }
}

struct EntityInfo {
    location: [f32; 3],
    appearance: Arc<String>
}

impl EntityInfo {
    fn new(location: [f32; 3], appearance: Arc<String>) -> EntityInfo {
        EntityInfo {
            location: location.clone(),
            appearance: appearance
        }
    }
}

struct Scene {
    view_position: [f32; 3],
    light: [f32; 3],
    entities: HashMap<u64, EntityInfo>
}

impl Scene {
    fn render(&self, display: &Display, appearance_cache: &mut AppearanceCache) -> LoadResult<()> {

        use glium::{Surface};
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1e16);

        let view = view_matrix(&self.view_position, &[0.0, 0.0, 1.0], &[0.0, 1.0, 0.0]);

        let perspective = {
            let (width, height) = target.get_dimensions();
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = 3.141592 / 3.0;
            let zfar = 1024.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();

            [
                [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
                [         0.0         ,     f ,              0.0              ,   0.0],
                [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
                [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
            ]
        };

        for (_, entity) in self.entities.iter() {
            let model = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [entity.location[0], entity.location[1], entity.location[2], 1.0f32]
            ];
            appearance_cache.get(display, entity.appearance.clone())
                .and_then(|app| {
                    app.render(&mut target, &model, &view, &perspective, &self.light)
                    .map_err(|e| LoadError::Glium(format!("glium error: {:?}", e)))
                 })
                .unwrap();
        }
        target.finish()
            .map_err(|e| LoadError::Glium(format!("glium error: {:?}", e)))
    }
}

fn main() {
    use glium::{DisplayBuild};
    use clap::{Arg, App};
    let matches = App::new("angel-stage-2")
        .version("0.1")
        .author("Kyle R. Zentner <zentner.kyle@gmail.com>")
        .about("Strategy game.")
        .arg(Arg::with_name("server")
            .short("s")
            .long("server")
            .value_name("address")
            .help("Sets the server to use")
            .takes_value(true))
        .get_matches();

    let server = matches.value_of("server").unwrap_or("127.0.0.1:4410").parse().unwrap();

    println!("Connecting to server at {}", server);

    let display = glium::glutin::WindowBuilder::new()
                        .with_depth_buffer(24)
                        .with_gl(glium::glutin::GlRequest::Latest)
                        .with_vsync()
                        .with_title("snowglobe".to_owned())
                        .build_glium().unwrap();

    let mut scene = Arc::new(Mutex::new(Scene {
        view_position: [0.0, 0.0, -10.0],
        light: [1.4, 0.4, 0.7f32],
        entities: HashMap::new()
    }));

    let command_sender = connect_to_server(server, scene.clone());
    command_sender.send(CommandEvent::Move([0.0, 0.0, 0.0f32]));

    let mut appearance_cache = AppearanceCache::new();

    let mut last_render_time = time::SteadyTime::now();
    let frame_duration = time::Duration::seconds(1) / 60;
    let mut nframes = 0u64; 
    let mut total_wait = 0u64;
    let mut start_of_second = time::SteadyTime::now();
    let mut frames_this_second = 0;
    let mut fps = 0;
    loop {
        let now = time::SteadyTime::now();
        if now + time::Duration::milliseconds(1) > last_render_time + frame_duration {
            last_render_time = time::SteadyTime::now();
            scene.lock().unwrap().render(&display, &mut appearance_cache).unwrap();
            frames_this_second += 1;
            if now >= start_of_second + time::Duration::seconds(1) {
                fps = frames_this_second;
                frames_this_second = 0;
                start_of_second = now;
                display.get_window()
                    .map(|w| {
                        let title = format!("snowglobe ({} fps) ({}ms avg. slack / frame)", fps, total_wait / nframes);
                        w.set_title(&title)
                    });
            }
        } else {
            let wait_time = (last_render_time + frame_duration - now).num_milliseconds() as u64;
            total_wait += wait_time;
            nframes += 1;
            std::thread::sleep(std::time::Duration::from_millis(wait_time));
        }

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}


fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s[0], u[0], f[0], 0.0],
        [s[1], u[1], f[1], 0.0],
        [s[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

const SERVER: mio::Token = mio::Token(0);
const BUFFER_SIZE : usize = 4096;

fn parse_point(point: common_capnp::point::Reader) -> [f32; 3] {
    [point.get_x(), point.get_y(), point.get_z()]
}

fn write_point(p: &mut common_capnp::point::Builder, point: &[f32; 3]) {
    p.set_x(point[0]);
    p.set_y(point[1]);
    p.set_z(point[2]);
}

struct ClientHandler {
    socket: mio::udp::UdpSocket,
    server_address: std::net::SocketAddr,
    scene: Arc<Mutex<Scene>>,
}

impl ClientHandler {
    fn process_client_messsage(&mut self) -> ClientResult<()> {
        use capnp::serialize;
        let mut in_buf = mio::buf::ByteBuf::mut_with_capacity(BUFFER_SIZE);
        let maybe_addr : Option<std::net::SocketAddr> = try!(self.socket.recv_from(&mut in_buf)
                                                             .map_err(|e| ClientError::Io("SERVER".to_owned(), e)));
        let addr = try!(maybe_addr.ok_or(ClientError::Intermittent));
        let reader = try!(serialize::read_message(&mut in_buf.flip(),
                                                  ::capnp::message::ReaderOptions::new())
                          .map_err(|e| ClientError::Capnp(e)));
        let message = try!(reader.get_root::<update_capnp::client_message::Reader>()
                           .map_err(|e| ClientError::Capnp(e)));
        let mut err = None;
        for update in try!(message.get_updates()
                           .map_err(|e| ClientError::Capnp(e))).iter() {
            match update.which() {
                Ok(update_capnp::update::EntityAlive(Ok(info))) => {
                    let identity = info.get_identity();
                    let location = parse_point(try!(info.get_location()
                                        .map_err(|e| ClientError::Capnp(e))));
                    let appearance = try!(info.get_appearance()
                                          .map_err(|e| ClientError::Capnp(e)));
                    let mut scene = self.scene.lock().unwrap();
                    match scene.entities.entry(identity) {
                        Vacant(mut entry) => {
                            entry.insert(EntityInfo::new(location, Arc::new(appearance.to_owned())));
                        },
                        Occupied(mut entry) => {
                            let entity = entry.get_mut();
                            entity.location = location;
                            if *entity.appearance != appearance {
                                entity.appearance = Arc::new(appearance.to_owned());
                            }
                        }
                    }
                },
                Ok(update_capnp::update::EntityDead(identity)) => {
                    let mut scene = self.scene.lock().unwrap();
                    scene.entities.remove(&identity);
                }
                Ok(update_capnp::update::EntityAlive(Err(e))) => {
                    err = Some(ClientError::Capnp(e));
                },
                Err(capnp::NotInSchema(_)) => {
                    err = Some(ClientError::Intermittent);
                },
            }
        }
        match err {
            None => Ok(()),
            Some(e) => Err(e)
        }
    }
}

#[derive(Debug)]
enum ClientError {
    Io(String, std::io::Error),
    Intermittent,
    Capnp(capnp::Error),
    Parse(String)
}

type ClientResult<R> = Result<R, ClientError>;

fn print_error<T>(res: ClientResult<T>) {
    if let Err(e) = res {
        match e {
            ClientError::Intermittent => {
            },
            _ => {
                println!("Error: {:?}", e);
            }
        }
    }
}


impl mio::Handler for ClientHandler {
    type Timeout = ();
    type Message = CommandEvent;

    fn ready(&mut self, event_loop: &mut mio::EventLoop<ClientHandler>, token: mio::Token, events: mio::EventSet) {
        match token {
            SERVER => {
                if events.is_readable() {
                    print_error(self.process_client_messsage());
                }
                else {
                    std::thread::sleep(std::time::Duration::from_millis(15));
                }
            },
            _ => panic!("unexpected token"),
        }
    }

    fn notify(&mut self, event_loop: &mut mio::EventLoop<ClientHandler>, msg: CommandEvent) {
        match msg {
            CommandEvent::Move(loc) => {
                let mut message = capnp::message::Builder::new_default();
                {
                    let mut command = message.init_root::<command_capnp::command::Builder>();
                    let mut move_command = command.init_move();
                    write_point(&mut move_command, &loc);
                }
                let mut out_buf = mio::buf::ByteBuf::mut_with_capacity(BUFFER_SIZE);
                capnp::serialize::write_message(&mut out_buf, &message).unwrap();
                self.socket.send_to(&mut out_buf.flip(), &self.server_address).unwrap();
            }
        }
    }
}

fn connect_to_server(addr: std::net::SocketAddr, scene: Arc<Mutex<Scene>>) -> mio::Sender<CommandEvent> {
    let addr_copy = addr.clone();
    let scene_copy = scene.clone();

    let mut event_loop = mio::EventLoop::new().unwrap();

    let sender = event_loop.channel();

    std::thread::spawn(move || {
        let client = match &addr_copy {
            &std::net::SocketAddr::V4(_) => mio::udp::UdpSocket::v4().unwrap(),
            &std::net::SocketAddr::V6(_) => mio::udp::UdpSocket::v6().unwrap()
        };

        event_loop.register(&client, SERVER);

        let mut handler = ClientHandler {
            socket: client,
            server_address: addr_copy,
            scene: scene_copy,
        };

        event_loop.run(&mut handler).unwrap();
    });
    sender
}
