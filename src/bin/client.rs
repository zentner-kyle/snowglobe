#[macro_use]
extern crate glium;
extern crate image;
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

use glium::backend::glutin_backend::GlutinFacade;
use glium::{Surface};

type Display = glium::backend::glutin_backend::GlutinFacade;

use std::fs::File;
use std::io::Read;
use std::collections::{HashMap};
use std::rc::Rc;
use std::collections::hash_map::Entry::{Occupied, Vacant};

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

struct AppearanceCache {
    loaded: HashMap<Rc<String>, Appearance>,
}

impl AppearanceCache {
    fn new() -> Self {
        AppearanceCache {
            loaded: HashMap::new()
        }
    }

    fn get<'a>(&'a mut self, display: &Display, name: Rc<String>) -> LoadResult<&'a Appearance> {
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
    appearance: Rc<String>
}

impl EntityInfo {
    fn new(location: [f32; 3], appearance: Rc<String>) -> EntityInfo {
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

    let server = matches.value_of("server").unwrap_or("127.0.0.1:4410");

    println!("Connecting to server at {}", server);

    let display = glium::glutin::WindowBuilder::new()
                        .with_depth_buffer(24)
                        .build_glium().unwrap();
    let mut scene = Scene {
        view_position: [0.0, 0.0, -10.0],
        light: [1.4, 0.4, 0.7f32],
        entities: HashMap::new()
    };

    scene.entities.insert(0, EntityInfo::new([0.0, 0.0, 0.0f32], Rc::new("simple".to_owned())));
    scene.entities.insert(1, EntityInfo::new([2.0, 0.0, 0.0f32], Rc::new("simple".to_owned())));

    let mut appearance_cache = AppearanceCache::new();

    loop {
        scene.render(&display, &mut appearance_cache).unwrap();

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
