use std::io::{self, Write};

use raylib::prelude::Matrix;
pub use raylib::prelude::Vector3 as Vec3;

use crate::shaders::{Fragment, Material, Uniforms};

pub const CAMERA: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 4.2,
};

pub struct Mesh {
    positions: Vec<Vec3>,
    triangles: Vec<[usize; 3]>,
}

impl Mesh {
    pub fn parse_obj(source: &str) -> Result<Self, String> {
        let mut positions = Vec::new();
        let mut triangles = Vec::new();
        for (line_number, line) in source.lines().enumerate() {
            let invalid = || {
                format!(
                    "Geometría inválida en sphere.obj, línea {}",
                    line_number + 1
                )
            };
            let mut fields = line.split_whitespace();
            match fields.next() {
                Some("v") => {
                    let mut coords = [0.0; 3];
                    for coordinate in &mut coords {
                        *coordinate = fields
                            .next()
                            .and_then(|field| field.parse::<f32>().ok())
                            .filter(|value| value.is_finite())
                            .ok_or_else(invalid)?;
                    }
                    positions.push(Vec3::new(coords[0], coords[1], coords[2]));
                }
                Some("f") => {
                    let mut triangle = [0; 3];
                    for index in &mut triangle {
                        *index = fields
                            .next()
                            .and_then(|field| field.split('/').next())
                            .and_then(|index| index.parse::<usize>().ok())
                            .and_then(|index| index.checked_sub(1))
                            .filter(|&index| index < positions.len())
                            .ok_or_else(invalid)?;
                    }
                    if fields.next().is_some() {
                        return Err(invalid());
                    }
                    triangles.push(triangle);
                }
                _ => {}
            }
        }
        if triangles.is_empty() {
            return Err("sphere.obj no tiene caras triangulares".into());
        }
        Ok(Self {
            positions,
            triangles,
        })
    }
}

pub struct Framebuffer {
    width: usize,
    height: usize,
    color: Vec<u8>,
    depth: Vec<f32>,
    projected: Vec<ProjectedVertex>,
    view: Matrix,
    focal: f32,
}

#[derive(Clone, Copy)]
struct ProjectedVertex {
    x: f32,
    y: f32,
    inverse_w: f32,
    local: Vec3,
    world: Vec3,
}

fn edge(a: ProjectedVertex, b: ProjectedVertex, x: f32, y: f32) -> f32 {
    (b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x)
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            color: vec![0; width * height * 4],
            depth: vec![0.0; width * height],
            projected: Vec::new(),
            view: Matrix::look_at(CAMERA, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0)),
            focal: height as f32 * 0.5 / 16.0_f32.to_radians().tan(),
        }
    }

    pub fn clear(&mut self) {
        self.depth.fill(0.0);
        for pixel in self.color.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        }
    }

    pub fn draw_object(
        &mut self,
        mesh: &Mesh,
        translation: Vec3,
        scale: f32,
        material: Material,
        uniforms: &Uniforms,
    ) {
        let mut model = Matrix::scale(scale, scale, scale);
        model.m12 = translation.x;
        model.m13 = translation.y;
        model.m14 = translation.z;
        self.projected.clear();
        self.projected.extend(mesh.positions.iter().map(|&local| {
            let world = local.transform_with(model);
            let eye = world.transform_with(self.view);
            let inverse_w = -eye.z.recip();
            ProjectedVertex {
                x: self.width as f32 * 0.5 + eye.x * self.focal * inverse_w,
                y: self.height as f32 * 0.5 - eye.y * self.focal * inverse_w,
                inverse_w,
                local,
                world,
            }
        }));

        for &[i, j, k] in &mesh.triangles {
            let vertices = [self.projected[i], self.projected[j], self.projected[k]];
            self.rasterize_triangle(vertices, material, uniforms);
        }
    }

    fn rasterize_triangle(
        &mut self,
        [a, b, c]: [ProjectedVertex; 3],
        material: Material,
        uniforms: &Uniforms,
    ) {
        // Las vistas mantienen la malla completa delante del plano cercano.
        let area = edge(a, b, c.x, c.y);
        if area >= -f32::EPSILON {
            return;
        }
        let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as usize;
        let max_x = (a.x.max(b.x).max(c.x).ceil() as usize).min(self.width);
        let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as usize;
        let max_y = (a.y.max(b.y).max(c.y).ceil() as usize).min(self.height);
        let inverse_area = area.recip();
        for y in min_y..max_y {
            for x in min_x..max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let u = edge(b, c, px, py) * inverse_area;
                let v = edge(c, a, px, py) * inverse_area;
                let w = edge(a, b, px, py) * inverse_area;
                if u < -1.0e-6 || v < -1.0e-6 || w < -1.0e-6 {
                    continue;
                }
                let u = u * a.inverse_w;
                let v = v * b.inverse_w;
                let w = w * c.inverse_w;
                let inverse_w = u + v + w;
                let index = y * self.width + x;
                if inverse_w <= self.depth[index] {
                    continue;
                }
                let perspective = inverse_w.recip();
                let local = (a.local * u + b.local * v + c.local * w) * perspective;
                let world = (a.world * u + b.world * v + c.world * w) * perspective;
                let normal = local.normalized();
                let fragment = Fragment {
                    normal,
                    view: (CAMERA - world).normalized(),
                };
                let color = material.shade(fragment, uniforms);
                self.depth[index] = inverse_w;
                self.color[index * 4..index * 4 + 4].copy_from_slice(&color);
            }
        }
    }

    pub fn rgba(&self) -> &[u8] {
        &self.color
    }

    pub fn write_ppm(&self, writer: &mut impl Write) -> io::Result<()> {
        write!(writer, "P6\n{} {}\n255\n", self.width, self.height)?;
        for pixel in self.color.chunks_exact(4) {
            writer.write_all(&pixel[..3])?;
        }
        Ok(())
    }
}
