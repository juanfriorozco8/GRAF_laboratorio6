mod pipeline;
mod shaders;

use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;

use raylib::prelude::*;

use crate::pipeline::{Framebuffer, Mesh, Vec3};
use crate::shaders::{Material, Uniforms};

const WIDTH: usize = 880;
const HEIGHT: usize = 520;
const INITIAL_LIGHT: f32 = -0.52;
const INITIAL_COVERAGE: f32 = 0.36;
const RECORDING_FPS: f32 = 24.0;

#[derive(Clone, Copy)]
enum Scene {
    Soles,
    Scarif,
    Bespin,
    Naboo,
}

impl Scene {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "soles" => Some(Self::Soles),
            "scarif" => Some(Self::Scarif),
            "bespin" => Some(Self::Bespin),
            "naboo" => Some(Self::Naboo),
            _ => None,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Soles => "1   /   LOS SOLES DE TATOOINE",
            Self::Scarif => "2   /   SCARIF",
            Self::Bespin => "3   /   BESPIN",
            Self::Naboo => "4   /   NABOO",
        }
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    mesh: &Mesh,
    scene: Scene,
    time: f32,
    light_angle: f32,
    cloud_coverage: f32,
    cloud_shadows: bool,
) {
    framebuffer.clear();
    let uniforms = Uniforms::new(
        time,
        Vec3::new(light_angle.sin(), 0.27, light_angle.cos()).normalized(),
        cloud_coverage,
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
        Scene::Scarif => {
            framebuffer.draw_object(mesh, Vec3::zero(), 1.03, Material::Scarif, &uniforms);
        }
        Scene::Bespin => {
            framebuffer.draw_object(mesh, Vec3::zero(), 1.03, Material::Bespin, &uniforms);
        }
        Scene::Naboo => {
            framebuffer.draw_object(mesh, Vec3::zero(), 1.03, Material::Naboo, &uniforms);
        }
    }
}

fn run_window(mesh: &Mesh, framebuffer: &mut Framebuffer) -> Result<(), Box<dyn Error>> {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Laboratorio 6 | Planetas y estrellas")
        .build();
    rl.set_target_fps(60);

    let image = Image::gen_image_color(WIDTH as i32, HEIGHT as i32, Color::BLACK);
    let mut texture = rl.load_texture_from_image(&thread, &image)?;
    let mut scene = Scene::Soles;
    let mut time = 0.0;
    let mut paused = false;
    let mut light_angle = INITIAL_LIGHT;
    let mut cloud_coverage = INITIAL_COVERAGE;
    let mut cloud_shadows = true;

    while !rl.window_should_close() {
        let delta = rl.get_frame_time().min(0.1);
        for (key, view) in [
            (KeyboardKey::KEY_ONE, Scene::Soles),
            (KeyboardKey::KEY_TWO, Scene::Scarif),
            (KeyboardKey::KEY_THREE, Scene::Bespin),
            (KeyboardKey::KEY_FOUR, Scene::Naboo),
        ] {
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
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            light_angle -= delta * 0.95;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            light_angle += delta * 0.95;
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            cloud_coverage = (cloud_coverage + delta * 0.4).min(0.72);
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            cloud_coverage = (cloud_coverage - delta * 0.4).max(0.08);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_S) {
            cloud_shadows = !cloud_shadows;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            time = 0.0;
            light_angle = INITIAL_LIGHT;
            cloud_coverage = INITIAL_COVERAGE;
            cloud_shadows = true;
        }

        render(
            framebuffer,
            mesh,
            scene,
            time,
            light_angle,
            cloud_coverage,
            cloud_shadows,
        );
        texture.update_texture(framebuffer.rgba())?;

        let mut drawing = rl.begin_drawing(&thread);
        drawing.draw_texture(&texture, 0, 0, Color::WHITE);
        drawing.draw_text(scene.title(), 24, 22, 19, Color::RAYWHITE);
        let time_status = if paused { "   |   Pausa" } else { "" };
        let shadow_status = if matches!(scene, Scene::Scarif) && !cloud_shadows {
            "   |   Sin sombras"
        } else {
            ""
        };
        drawing.draw_text(
            &format!(
                "1-4: vistas   |   Flechas: luz / nubes   |   Espacio: pausa   S: sombras   R: reiniciar{time_status}{shadow_status}"
            ),
            24,
            HEIGHT as i32 - 28,
            14,
            Color::LIGHTGRAY,
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
            (0.0, -1.20 + 2.40 * phase)
        } else {
            (elapsed, INITIAL_LIGHT)
        };
        render(
            framebuffer,
            mesh,
            scene,
            time,
            light_angle,
            INITIAL_COVERAGE,
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
        _ => Err("Uso: graf-laboratorio6 [--frames <soles|scarif|bespin|naboo> <cantidad> [--sweep-light] [--without-cloud-shadows]]".into()),
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
