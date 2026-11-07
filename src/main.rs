mod hud;
mod pipeline;
mod shaders;

use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;

use raylib::prelude::*;

use crate::hud::{Hud, HudState};
use crate::pipeline::{Framebuffer, Mesh, Vec3};
use crate::shaders::{Material, Uniforms};

const WIDTH: usize = 880;
const HEIGHT: usize = 520;
const VIEW_TOP: i32 = 112;
const FOOTER_HEIGHT: i32 = 68;
const RECORDING_FPS: f32 = 24.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scene {
    Soles,
    Scarif,
    Bespin,
    Naboo,
    Mustafar,
}

impl Scene {
    const ALL: [Self; 5] = [
        Self::Soles,
        Self::Scarif,
        Self::Bespin,
        Self::Naboo,
        Self::Mustafar,
    ];

    fn index(self) -> usize {
        self as usize
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|scene| scene.short_name().eq_ignore_ascii_case(name))
            .or_else(|| (name == "soles").then_some(Self::Soles))
    }

    fn title(self) -> &'static str {
        match self {
            Self::Soles => "SOLES DE TATOOINE",
            Self::Scarif => "SCARIF",
            Self::Bespin => "BESPIN",
            Self::Naboo => "NABOO",
            Self::Mustafar => "MUSTAFAR",
        }
    }

    fn short_name(self) -> &'static str {
        match self {
            Self::Soles => "Tatooine",
            Self::Scarif => "Scarif",
            Self::Bespin => "Bespin",
            Self::Naboo => "Naboo",
            Self::Mustafar => "Mustafar",
        }
    }

    fn kind(self) -> &'static str {
        match self {
            Self::Soles => "DOS ESTRELLAS",
            Self::Bespin => "GIGANTE GASEOSO",
            Self::Scarif | Self::Naboo | Self::Mustafar => "PLANETA ROCOSO",
        }
    }

    fn effect_name(self) -> &'static str {
        match self {
            Self::Soles => "Actividad del plasma",
            Self::Scarif => "Cobertura de nubes",
            Self::Bespin => "Intensidad del viento",
            Self::Naboo => "Cobertura de nubes altas",
            Self::Mustafar => "Actividad de la lava",
        }
    }

    fn default_effect(self) -> f32 {
        match self {
            Self::Soles => 0.5,
            Self::Scarif => 0.38,
            Self::Bespin => 0.46,
            Self::Naboo => 0.56,
            Self::Mustafar => 0.60,
        }
    }

    fn lighting(self) -> (f32, f32) {
        match self {
            Self::Soles => (0.0, 0.27),
            Self::Scarif => (0.73, 0.31),
            Self::Bespin => (-0.85, 0.52),
            Self::Naboo => (-1.02, 0.33),
            Self::Mustafar => (-0.76, 0.60),
        }
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    mesh: &Mesh,
    scene: Scene,
    time: f32,
    light_angle: f32,
    effect: f32,
    cloud_shadows: bool,
) {
    framebuffer.clear();
    let (yaw, elevation) = scene.lighting();
    let angle = yaw + light_angle;
    let uniforms = Uniforms::new(
        time,
        Vec3::new(angle.sin(), elevation, angle.cos()).normalized(),
        effect,
        cloud_shadows,
    );
    match scene {
        Scene::Soles => {
            framebuffer.draw_object(
                mesh,
                Vec3::new(-1.00, 0.15, 0.0),
                0.90,
                Material::TatooI,
                &uniforms,
            );
            framebuffer.draw_object(
                mesh,
                Vec3::new(1.08, -0.24, 0.0),
                0.70,
                Material::TatooII,
                &uniforms,
            );
        }
        Scene::Scarif | Scene::Bespin | Scene::Naboo | Scene::Mustafar => {
            let material = match scene {
                Scene::Scarif => Material::Scarif,
                Scene::Bespin => Material::Bespin,
                Scene::Naboo => Material::Naboo,
                Scene::Mustafar => Material::Mustafar,
                Scene::Soles => unreachable!(),
            };
            framebuffer.draw_object(mesh, Vec3::zero(), 1.15, material, &uniforms);
        }
    }
}

fn run_window(mesh: &Mesh, framebuffer: &mut Framebuffer) -> Result<(), Box<dyn Error>> {
    let footer = VIEW_TOP + HEIGHT as i32;
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, footer + FOOTER_HEIGHT)
        .title("Archivo estelar | Laboratorio 6")
        .build();
    rl.set_target_fps(60);

    let image = Image::gen_image_color(WIDTH as i32, HEIGHT as i32, Color::BLACK);
    let mut texture = rl.load_texture_from_image(&thread, &image)?;
    let hud = Hud::load(&mut rl, &thread)?;
    let keys = [
        KeyboardKey::KEY_ONE,
        KeyboardKey::KEY_TWO,
        KeyboardKey::KEY_THREE,
        KeyboardKey::KEY_FOUR,
        KeyboardKey::KEY_FIVE,
    ];
    let mut scene = Scene::Soles;
    let mut time = 0.0;
    let mut paused = false;
    let mut light_angle = 0.0;
    let mut effects = Scene::ALL.map(Scene::default_effect);
    let mut cloud_shadows = true;
    let mut show_help = false;

    while !rl.window_should_close() {
        let delta = rl.get_frame_time().min(0.1);
        for (key, view) in keys.into_iter().zip(Scene::ALL) {
            if rl.is_key_pressed(key) {
                scene = view;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            paused = !paused;
        }
        if !paused {
            time += delta;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            show_help = !show_help;
        }
        let direction = f32::from(rl.is_key_down(KeyboardKey::KEY_RIGHT))
            - f32::from(rl.is_key_down(KeyboardKey::KEY_LEFT));
        light_angle = (light_angle + direction * delta * 0.95).rem_euclid(std::f32::consts::TAU);
        let strength = f32::from(rl.is_key_down(KeyboardKey::KEY_UP))
            - f32::from(rl.is_key_down(KeyboardKey::KEY_DOWN));
        let effect = &mut effects[scene.index()];
        *effect = (*effect + strength * delta * 0.48).clamp(0.0, 1.0);
        if rl.is_key_pressed(KeyboardKey::KEY_S) && scene == Scene::Scarif {
            cloud_shadows = !cloud_shadows;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            time = 0.0;
            light_angle = 0.0;
            *effect = scene.default_effect();
            cloud_shadows = true;
        }

        render(
            framebuffer,
            mesh,
            scene,
            time,
            light_angle,
            *effect,
            cloud_shadows,
        );
        texture.update_texture(framebuffer.rgba())?;

        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(Color::BLACK);
        drawing.draw_texture(&texture, 0, VIEW_TOP, Color::WHITE);
        hud.draw(
            &mut drawing,
            WIDTH as i32,
            footer,
            HudState {
                scene,
                effect: *effect,
                paused,
                shadows: cloud_shadows,
                show_help,
            },
        );
    }
    Ok(())
}

fn write_frames(
    mesh: &Mesh,
    framebuffer: &mut Framebuffer,
    scene: Scene,
    count: u32,
    sweep_light: bool,
    cloud_shadows: bool,
) -> io::Result<()> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for frame in 0..count {
        let elapsed = frame as f32 / RECORDING_FPS;
        let (time, light_angle) = if sweep_light {
            let phase = frame as f32 / (count - 1).max(1) as f32;
            (0.0, -1.20 + 2.40 * phase - scene.lighting().0)
        } else {
            (elapsed, 0.0)
        };
        render(
            framebuffer,
            mesh,
            scene,
            time,
            light_angle,
            scene.default_effect(),
            cloud_shadows,
        );
        framebuffer.write_ppm(&mut output)?;
    }
    output.flush()
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mesh = Mesh::parse_obj(include_str!("../assets/sphere.obj"))?;
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    match arguments.as_slice() {
        [] => run_window(&mesh, &mut framebuffer),
        [flag, scene, count, rest @ ..] if flag == "--frames" => {
            let scene = Scene::from_name(scene).ok_or("Vista desconocida")?;
            let count: u32 = count.parse()?;
            if count == 0 {
                return Err("El número de cuadros debe ser positivo".into());
            }
            let mut sweep_light = false;
            let mut cloud_shadows = true;
            for option in rest {
                match option.as_str() {
                    "--sweep-light" if !sweep_light => sweep_light = true,
                    "--without-cloud-shadows" if cloud_shadows => cloud_shadows = false,
                    _ => return Err(format!("Opción inválida: {option}").into()),
                }
            }
            write_frames(&mesh, &mut framebuffer, scene, count, sweep_light, cloud_shadows)?;
            Ok(())
        }
        _ => Err("Uso: graf-laboratorio6 [--frames <soles|scarif|bespin|naboo|mustafar> <cantidad> [--sweep-light] [--without-cloud-shadows]]".into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
