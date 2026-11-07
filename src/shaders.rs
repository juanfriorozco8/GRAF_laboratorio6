use crate::pipeline::Vec3;

#[derive(Clone, Copy)]
pub enum Material {
    TatooI,
    TatooII,
    Scarif,
    Bespin,
    Naboo,
}

#[derive(Clone, Copy)]
pub struct Fragment {
    pub normal: Vec3,
    pub view: Vec3,
}

pub struct Uniforms {
    pub time: f32,
    pub light: Vec3,
    cloud_coverage: f32,
    cloud_rotation: (f32, f32),
    cloud_shadows: bool,
}

impl Uniforms {
    pub fn new(time: f32, light: Vec3, cloud_coverage: f32, cloud_shadows: bool) -> Self {
        Self {
            time,
            light,
            cloud_coverage,
            cloud_rotation: (time * CLOUD_SPEED).sin_cos(),
            cloud_shadows,
        }
    }
}

const PLASMA_SPEED: f32 = 0.33;
const NIGHT_LIGHT_STRENGTH: f32 = 1.15;
const WATER_SPECULAR_POWER: f32 = 32.0;
const CLOUD_SPEED: f32 = 0.13;
const CLOUD_HEIGHT: f32 = 0.052;
const VORTEX_RADIUS: f32 = 0.38;

impl Material {
    pub fn shade(self, fragment: Fragment, uniforms: &Uniforms) -> [u8; 4] {
        let color = match self {
            Self::TatooI => tatoo_i(fragment, uniforms),
            Self::TatooII => tatoo_ii(fragment, uniforms),
            Self::Scarif => scarif(fragment, uniforms),
            Self::Bespin => bespin(fragment, uniforms),
            Self::Naboo => naboo(fragment, uniforms),
        };
        let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        [channel(color.x), channel(color.y), channel(color.z), 255]
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Vec3 {
    Vec3 {
        x: r as f32 / 255.0,
        y: g as f32 / 255.0,
        z: b as f32 / 255.0,
    }
}

fn mix<T>(from: T, to: T, amount: f32) -> T
where
    T: std::ops::Mul<f32, Output = T> + std::ops::Add<Output = T>,
{
    from * (1.0 - amount) + to * amount
}

fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lattice(x: i32, y: i32, z: i32) -> f32 {
    let mut hash = (x as u32).wrapping_mul(0x9e37_79b1)
        ^ (y as u32).wrapping_mul(0x85eb_ca77)
        ^ (z as u32).wrapping_mul(0xc2b2_ae3d);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb_352d);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846c_a68b);
    ((hash ^ (hash >> 16)) >> 8) as f32 / 16_777_215.0
}

fn noise(p: Vec3) -> f32 {
    let x = p.x.floor() as i32;
    let y = p.y.floor() as i32;
    let z = p.z.floor() as i32;
    let fade = |t: f32| t * t * (3.0 - 2.0 * t);
    let u = fade(p.x - x as f32);
    let v = fade(p.y - y as f32);
    let w = fade(p.z - z as f32);
    let layer = |z| {
        mix(
            mix(lattice(x, y, z), lattice(x + 1, y, z), u),
            mix(lattice(x, y + 1, z), lattice(x + 1, y + 1, z), u),
            v,
        )
    };
    mix(layer(z), layer(z + 1), w)
}

fn fractal_noise(p: Vec3) -> f32 {
    0.56 * noise(p) + 0.28 * noise(p * 2.07 + 8.1) + 0.16 * noise(p * 4.19 - 9.3)
}

fn tatoo_i(fragment: Fragment, uniforms: &Uniforms) -> Vec3 {
    let p = fragment.normal;
    let time = uniforms.time * PLASMA_SPEED;
    let flow = noise(p * 4.2 + Vec3::new(time, -time * 0.3, 7.0));
    let plasma = fractal_noise(p * 10.0 + Vec3::new(flow * 2.3, time, -flow * 2.0));
    let granules = noise(p * 56.0 + Vec3::new(-time, time * 0.6, 3.0));
    let spots = smoothstep(0.60, 0.81, fractal_noise(p * 3.6 + 13.0));
    let filaments = 1.0 - smoothstep(0.015, 0.12, (plasma - 0.52).abs());
    let limb = smoothstep(0.0, 0.72, p.dot(fragment.view));
    let heat = (0.14 + plasma * 0.60 + granules * 0.19 + filaments * 0.21) * (1.0 - spots * 0.66);
    let ember = mix(
        rgb(142, 29, 14),
        rgb(255, 144, 36),
        smoothstep(0.18, 0.65, heat),
    );
    let gold = mix(ember, rgb(255, 246, 174), smoothstep(0.52, 0.90, heat));
    mix(rgb(239, 73, 25), gold, 0.65 + 0.35 * limb)
}

fn tatoo_ii(fragment: Fragment, uniforms: &Uniforms) -> Vec3 {
    let p = fragment.normal;
    let time = uniforms.time * PLASMA_SPEED;
    let shear = noise(p * 6.1 + Vec3::new(-time * 1.3, 12.0, time));
    let storm = fractal_noise(p * Vec3::new(7.5, 15.0, 7.5) + Vec3::new(shear * 2.8, time, 20.0));
    let veins = 1.0 - smoothstep(0.010, 0.145, (storm - 0.55).abs());
    let granules = noise(p * 63.0 + Vec3::new(5.0, -time, 2.0));
    let arc = smoothstep(0.40, 0.73, noise(p * 2.9 + 31.0));
    let heat = storm * 0.46 + veins * 0.26 + granules * 0.18 + arc * 0.21;
    let red = mix(
        rgb(91, 17, 31),
        rgb(245, 62, 31),
        smoothstep(0.16, 0.62, heat),
    );
    let bright = mix(red, rgb(255, 179, 96), smoothstep(0.49, 0.87, heat));
    let rim = (1.0 - smoothstep(0.0, 0.57, p.dot(fragment.view))) * 0.45;
    mix(bright, rgb(255, 65, 35), rim)
}

fn water_reflection(normal: Vec3, view: Vec3, light: Vec3, water: f32) -> f32 {
    let half = (view + light).normalized();
    let daylight = smoothstep(0.02, 0.20, normal.dot(light));
    let highlight = normal.dot(half).max(0.0).powf(WATER_SPECULAR_POWER);
    water * daylight * highlight
}

fn coastal_lights(p: Vec3, height: f32, illumination: f32) -> Vec3 {
    let shore = smoothstep(0.491, 0.526, height) * (1.0 - smoothstep(0.58, 0.64, height));
    let settlements = smoothstep(0.46, 0.69, noise(p * 12.0 + 55.0));
    let lights = smoothstep(0.49, 0.78, noise(p * 74.0 - 39.0));
    let night = 1.0 - smoothstep(-0.15, 0.015, illumination);
    rgb(255, 171, 80) * (shore * settlements * lights * night * NIGHT_LIGHT_STRENGTH)
}

fn cloud_mask(p: Vec3, uniforms: &Uniforms) -> f32 {
    let (sin, cos) = uniforms.cloud_rotation;
    let drift = Vec3::new(cos * p.x + sin * p.z, p.y, -sin * p.x + cos * p.z);
    let density = fractal_noise(drift * Vec3::new(5.6, 9.0, 5.6) + 19.0);
    let threshold = 0.78 - 0.60 * uniforms.cloud_coverage;
    smoothstep(threshold, threshold + 0.16, density)
}

fn cloud_shadow(p: Vec3, illumination: f32, uniforms: &Uniforms) -> f32 {
    if !uniforms.cloud_shadows || illumination <= 0.0 {
        return 0.0;
    }
    // Proyectamos hacia la luz hasta alcanzar la capa matemática de nubes.
    let radius = 1.0 + CLOUD_HEIGHT;
    let distance = (illumination * illumination + radius * radius - 1.0).sqrt() - illumination;
    let projected = (p + uniforms.light * distance) * radius.recip();
    0.68 * cloud_mask(projected, uniforms) * smoothstep(0.0, 0.20, illumination)
}

fn scarif(fragment: Fragment, uniforms: &Uniforms) -> Vec3 {
    let p = fragment.normal;
    let distortion = noise(p * 4.0 + 22.5) - 0.5;
    let height = fractal_noise(p * 4.8 + Vec3::new(distortion, 0.24, -distortion * 0.73));
    let land = smoothstep(0.485, 0.525, height);
    let shallow = smoothstep(0.30, 0.475, height).powi(2);
    let sand = smoothstep(0.512, 0.531, height) * (1.0 - smoothstep(0.539, 0.561, height));
    let ocean = mix(rgb(10, 39, 94), rgb(22, 185, 167), shallow);
    let jungle = mix(rgb(21, 72, 46), rgb(81, 124, 61), noise(p * 26.0 + 34.0));
    let ground = mix(ocean, mix(jungle, rgb(216, 190, 141), sand * 0.84), land);
    let illumination = p.dot(uniforms.light);
    let direct = illumination.max(0.0);
    let occlusion = 1.0 - cloud_shadow(p, illumination, uniforms);
    let reflection = water_reflection(p, fragment.view, uniforms.light, 1.0 - land);
    let surface = ground * (0.11 + direct * 0.89 * occlusion)
        + rgb(230, 248, 244) * (reflection * occlusion * 0.66)
        + coastal_lights(p, height, illumination);
    let clouds = rgb(228, 246, 250) * (0.10 + 0.90 * direct.sqrt());
    let color = mix(surface, clouds, cloud_mask(p, uniforms) * 0.88);
    atmosphere(color, fragment, rgb(67, 163, 223), illumination, 0.38)
}

fn bespin_vortex(p: Vec3) -> (Vec3, f32) {
    let center = Vec3::new(0.26, -0.23, 0.938).normalized();
    let radius = (p - center).length();
    if radius >= VORTEX_RADIUS {
        return (p, 0.0);
    }
    let falloff = 1.0 - smoothstep(0.03, VORTEX_RADIUS, radius);
    let (sin, cos) = (falloff * 4.45).sin_cos();
    let warped = p * cos + center.cross(p) * sin + center * (center.dot(p) * (1.0 - cos));
    (warped, falloff)
}

fn bespin(fragment: Fragment, uniforms: &Uniforms) -> Vec3 {
    let p = fragment.normal;
    let (warped, vortex) = bespin_vortex(p);
    let eddies = fractal_noise(warped * Vec3::new(4.1, 11.0, 4.1) + 44.0);
    let latitude = warped.y + 0.095 * (eddies - 0.5);
    let broad = (latitude * 26.0 + 0.55 * (latitude * 12.0).sin()).sin() * 0.5 + 0.5;
    let streaks = noise(warped * Vec3::new(7.0, 29.0, 7.0) + 57.0);
    let wisps = (latitude * 106.0 + eddies * 3.0 + streaks * 1.4).sin() * 0.5 + 0.5;
    let rose = mix(rgb(122, 77, 132), rgb(204, 130, 152), broad);
    let banded = mix(
        rose,
        rgb(251, 219, 186),
        smoothstep(0.43, 0.96, broad + wisps * 0.19),
    );
    let eye = smoothstep(0.63, 0.97, vortex);
    let color = mix(
        banded,
        mix(rgb(109, 64, 118), rgb(239, 174, 189), wisps),
        eye * 0.85,
    ) * (0.89 + 0.14 * wisps + 0.07 * streaks);
    let illumination = p.dot(uniforms.light);
    let cloud_light = 0.24 + 0.76 * illumination.max(0.0).sqrt();
    atmosphere(
        color * cloud_light,
        fragment,
        rgb(212, 145, 189),
        illumination,
        0.27,
    )
}

fn naboo(fragment: Fragment, uniforms: &Uniforms) -> Vec3 {
    let p = fragment.normal;
    let folds = noise(p * 5.0 + 3.0);
    let terrain = fractal_noise(p * 3.3 + Vec3::new(-9.0, 7.0, 5.0));
    let grain = fractal_noise(p * 21.0 + 38.0);
    let elevation = terrain + 0.14 * (folds - 0.5) + 0.095 * (grain - 0.5);
    let land = smoothstep(0.43, 0.47, elevation);
    let plains = mix(
        rgb(26, 66, 53),
        rgb(169, 184, 100),
        smoothstep(0.45, 0.66, elevation),
    );
    let ridge = 1.0 - smoothstep(0.02, 0.18, (grain - 0.49).abs());
    let alpine = smoothstep(0.60, 0.79, elevation) * ridge;
    let ice = smoothstep(0.73, 0.91, p.y.abs() + 0.10 * (folds - 0.5));
    let ground = mix(
        rgb(22, 65, 105),
        mix(plains * (0.72 + 0.50 * grain), rgb(205, 213, 185), alpine),
        land,
    );
    let base = mix(ground, rgb(198, 225, 224), ice * 0.87);
    let illumination = p.dot(uniforms.light);
    let direct = illumination.max(0.0);
    let reflection = water_reflection(p, fragment.view, uniforms.light, (1.0 - land) * (1.0 - ice));
    let surface = base * (0.18 + 0.82 * direct) + rgb(229, 239, 223) * (reflection * 0.37);
    atmosphere(surface, fragment, rgb(124, 183, 218), illumination, 0.34)
}

fn atmosphere(
    color: Vec3,
    fragment: Fragment,
    tint: Vec3,
    illumination: f32,
    strength: f32,
) -> Vec3 {
    let rim = 1.0 - smoothstep(0.0, 0.56, fragment.normal.dot(fragment.view));
    let daylight = 0.15 + 0.85 * smoothstep(-0.15, 0.65, illumination);
    mix(color, tint, rim * daylight * strength)
}
