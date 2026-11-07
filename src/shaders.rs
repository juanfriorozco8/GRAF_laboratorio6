use crate::pipeline::Vec3;

#[derive(Clone, Copy)]
pub enum Material {
    TatooI,
    TatooII,
    Scarif,
    Bespin,
    Naboo,
    Mustafar,
}

#[derive(Clone, Copy)]
pub struct Fragment {
    pub normal: Vec3,
    pub view: Vec3,
}

pub struct Uniforms {
    pub time: f32,
    pub light: Vec3,
    effect: f32,
    cloud_rotation: (f32, f32),
    naboo_rotation: (f32, f32),
    wind_rotation: (f32, f32),
    cloud_shadows: bool,
}

impl Uniforms {
    pub fn new(time: f32, light: Vec3, effect: f32, cloud_shadows: bool) -> Self {
        Self {
            time,
            light,
            effect,
            cloud_rotation: (time * CLOUD_SPEED).sin_cos(),
            naboo_rotation: (time * NABOO_CLOUD_SPEED).sin_cos(),
            wind_rotation: (time * BESPIN_WIND_SPEED).sin_cos(),
            cloud_shadows,
        }
    }
}

const PLASMA_SPEED: f32 = 0.33;
const NIGHT_LIGHT_STRENGTH: f32 = 1.15;
const WATER_SPECULAR_POWER: f32 = 32.0;
const CLOUD_SPEED: f32 = 0.13;
const CLOUD_HEIGHT: f32 = 0.052;
const VORTEX_RADIUS: f32 = 0.35;
const BESPIN_WIND_SPEED: f32 = 0.30;
const NABOO_CLOUD_SPEED: f32 = -0.115;
const LAVA_FLOW_SPEED: f32 = 0.57;

impl Material {
    pub fn shade(self, fragment: Fragment, uniforms: &Uniforms) -> [u8; 4] {
        let color = match self {
            Self::TatooI => tatoo_i(fragment, uniforms),
            Self::TatooII => tatoo_ii(fragment, uniforms),
            Self::Scarif => scarif(fragment, uniforms),
            Self::Bespin => bespin(fragment, uniforms),
            Self::Naboo => naboo(fragment, uniforms),
            Self::Mustafar => mustafar(fragment, uniforms),
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
    let activity = 0.56 + 0.85 * uniforms.effect;
    let flow = noise(p * 4.2 + Vec3::new(time, -time * 0.3, 7.0));
    let plasma = fractal_noise(p * 10.0 + Vec3::new(flow * 2.3, time, -flow * 2.0));
    let granules = noise(p * 56.0 + Vec3::new(-time, time * 0.6, 3.0));
    let spots = smoothstep(0.60, 0.81, fractal_noise(p * 3.6 + 13.0));
    let filaments = 1.0 - smoothstep(0.015, 0.12, (plasma - 0.52).abs());
    let limb = smoothstep(0.0, 0.72, p.dot(fragment.view));
    let heat = (0.14 + plasma * 0.60 + granules * 0.19 + filaments * 0.21 * activity)
        * (1.0 - spots * 0.66);
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
    let activity = 0.56 + 0.85 * uniforms.effect;
    let shear = noise(p * 6.1 + Vec3::new(-time * 1.3, 12.0, time));
    let storm = fractal_noise(p * Vec3::new(7.5, 15.0, 7.5) + Vec3::new(shear * 2.8, time, 20.0));
    let veins = 1.0 - smoothstep(0.010, 0.145, (storm - 0.55).abs());
    let granules = noise(p * 63.0 + Vec3::new(5.0, -time, 2.0));
    let arc = smoothstep(0.40, 0.73, noise(p * 2.9 + 31.0));
    let heat = storm * 0.46 + veins * 0.26 * activity + granules * 0.18 + arc * 0.21;
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
    let shore = smoothstep(0.548, 0.575, height) * (1.0 - smoothstep(0.61, 0.66, height));
    let settlements = smoothstep(0.46, 0.69, noise(p * 12.0 + 55.0));
    let lights = smoothstep(0.49, 0.78, noise(p * 74.0 - 39.0));
    let night = 1.0 - smoothstep(-0.15, 0.015, illumination);
    rgb(255, 171, 80) * (shore * settlements * lights * night * NIGHT_LIGHT_STRENGTH)
}

fn rotate_y(p: Vec3, (sin, cos): (f32, f32)) -> Vec3 {
    Vec3::new(cos * p.x + sin * p.z, p.y, -sin * p.x + cos * p.z)
}

fn cloud_mask(p: Vec3, uniforms: &Uniforms) -> f32 {
    let drift = rotate_y(p, uniforms.cloud_rotation);
    let front = noise(drift * 4.9 + 21.4) - 0.5;
    let density =
        fractal_noise(drift * Vec3::new(10.2, 14.0, 10.2) + Vec3::new(19.0, front * 4.2, -7.3));
    let threshold = 0.81 - 0.65 * uniforms.effect;
    smoothstep(threshold, threshold + 0.18, density)
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
    let warp = fractal_noise(p * 4.1 + 22.5) - 0.5;
    let islands = fractal_noise(p * 6.0 + Vec3::new(warp * 1.8, 0.24, -warp * 1.4));
    let detail = fractal_noise(p * 32.0 + 10.3);
    let height = islands + 0.14 * (detail - 0.5);
    let land = smoothstep(0.545, 0.572, height);
    let shallow = smoothstep(0.424, 0.551, height).powi(2);
    let sand = smoothstep(0.550, 0.571, height) * (1.0 - smoothstep(0.573, 0.601, height));
    let deep_water = mix(rgb(13, 48, 93), rgb(41, 103, 159), islands);
    let reef = mix(rgb(43, 130, 154), rgb(110, 221, 207), detail);
    let ocean = mix(deep_water, reef, shallow);
    let jungle = mix(rgb(38, 70, 68), rgb(105, 123, 96), detail);
    let ground = mix(ocean, mix(jungle, rgb(173, 169, 138), sand * detail), land);
    let illumination = p.dot(uniforms.light);
    let direct = illumination.max(0.0);
    let occlusion = 1.0 - cloud_shadow(p, illumination, uniforms);
    let reflection = water_reflection(p, fragment.view, uniforms.light, 1.0 - land);
    let surface = ground * (0.09 + direct.powf(0.78) * 0.91 * occlusion)
        + rgb(195, 227, 251) * (reflection * occlusion * 0.70)
        + coastal_lights(p, height, illumination);
    let clouds = rgb(220, 235, 243) * (0.07 + 0.93 * direct.sqrt());
    let color = mix(surface, clouds, cloud_mask(p, uniforms) * 0.85);
    atmosphere(color, fragment, rgb(103, 207, 245), illumination, 0.62)
}

fn bespin_vortex(p: Vec3, intensity: f32) -> (Vec3, f32) {
    let center = Vec3::new(0.13, -0.23, 0.96).normalized();
    let radius = (p - center).length();
    if radius >= VORTEX_RADIUS {
        return (p, 0.0);
    }
    let falloff = 1.0 - smoothstep(0.03, VORTEX_RADIUS, radius);
    let (sin, cos) = (falloff * (2.45 + 1.20 * intensity)).sin_cos();
    let warped = p * cos + center.cross(p) * sin + center * (center.dot(p) * (1.0 - cos));
    (warped, falloff)
}

fn bespin(fragment: Fragment, uniforms: &Uniforms) -> Vec3 {
    let p = fragment.normal;
    let (warped, vortex) = bespin_vortex(p, uniforms.effect);
    // Las corrientes siguen un eje inclinado respecto a la imagen.
    let tilted = Vec3::new(
        0.89 * warped.x + 0.456 * warped.y,
        0.89 * warped.y - 0.456 * warped.x,
        warped.z,
    );
    let drift = rotate_y(tilted, uniforms.wind_rotation);
    let gusts = fractal_noise(drift * Vec3::new(3.3, 12.5, 3.3) + 43.0);
    let fine = fractal_noise(drift * Vec3::new(11.0, 46.0, 11.0) + 56.1);
    let latitude = tilted.y + 0.13 * (gusts - 0.5);
    let broad = 0.5 + 0.5 * (latitude * 15.0 + 0.7 * (latitude * 7.0).sin()).sin();
    let wisps = 0.5 + 0.5 * (latitude * 54.0 + gusts * 4.0 + fine * 1.7).sin();
    let wisps = smoothstep(0.65, 0.98, wisps) * (0.11 + uniforms.effect * 0.30);
    let tint = smoothstep(0.19, 0.90, broad * 0.57 + gusts * 0.38 + vortex * 0.06);
    let color = mix(rgb(139, 65, 40), rgb(237, 168, 97), tint);
    let color = mix(color, rgb(255, 198, 132), wisps * (0.49 + fine * 0.51));
    let illumination = p.dot(uniforms.light);
    let cloud_light = 0.062 + 0.94 * illumination.max(0.0).powf(0.70);
    atmosphere(
        color * cloud_light,
        fragment,
        rgb(199, 153, 121),
        illumination,
        0.24,
    )
}

fn naboo(fragment: Fragment, uniforms: &Uniforms) -> Vec3 {
    let p = fragment.normal;
    let grain = fractal_noise(p * 25.0 + 38.0);
    let terrain = fractal_noise(p * 4.0 + Vec3::new(-9.0, 7.0, 5.0));
    let elevation = terrain + 0.18 * (grain - 0.5);
    let land = smoothstep(0.43, 0.49, elevation);
    let plains = mix(rgb(31, 77, 73), rgb(89, 135, 107), grain);
    let river = 1.0 - smoothstep(0.007, 0.038, (elevation - 0.51).abs());
    let water = mix(rgb(21, 59, 81), rgb(47, 104, 117), grain);
    let base = mix(water, plains, land * (1.0 - river * 0.72));

    let drift = rotate_y(p, uniforms.naboo_rotation);
    let curl = noise(drift * 4.0 + 2.0) - 0.5;
    let fronts = drift * Vec3::new(6.5, 11.5, 6.5) + Vec3::new(9.7, curl * 4.0, -curl * 2.5);
    let density = fractal_noise(fronts) * 0.76 + noise(drift * 54.0 - 10.0) * 0.24;
    let threshold = 0.78 - 0.58 * uniforms.effect;
    let cirrus = smoothstep(threshold, threshold + 0.17, density);
    let illumination = p.dot(uniforms.light);
    let direct = illumination.max(0.0);
    let reflection = water_reflection(p, fragment.view, uniforms.light, 1.0 - land);
    let surface =
        base * (0.10 + 0.90 * direct.powf(0.7)) + rgb(170, 206, 212) * (reflection * 0.28);
    let cloud_color = mix(rgb(21, 41, 48), rgb(218, 237, 239), direct.sqrt());
    let color = mix(surface, cloud_color, cirrus * 0.93);
    atmosphere(color, fragment, rgb(140, 213, 230), illumination, 0.52)
}

fn mustafar(fragment: Fragment, uniforms: &Uniforms) -> Vec3 {
    let p = fragment.normal;
    let warp = fractal_noise(p * 4.1 + 60.0) - 0.5;
    let terrain = p * 6.0 + Vec3::new(warp * 2.6, warp * -1.4, 17.0);
    let crust = fractal_noise(terrain);
    let grit = fractal_noise(p * 48.0 + 13.0);
    let provinces = fractal_noise(p * 3.7 - 31.0);
    let fault = (crust + 0.12 * (grit - 0.5) - 0.50).abs();
    let vents = smoothstep(0.48, 0.67, provinces);
    let rivers = (1.0 - smoothstep(0.008, 0.055, fault)) * vents;
    let glow = (1.0 - smoothstep(0.02, 0.21, fault)) * vents;
    let time = uniforms.time * LAVA_FLOW_SPEED;
    let current = noise(terrain * 4.8 + Vec3::new(time, -time * 0.58, 5.4));
    let activity = 0.25 + 0.95 * uniforms.effect;
    let heat = mix(
        rgb(180, 30, 9),
        rgb(255, 177, 56),
        smoothstep(0.31, 0.76, current),
    );
    let basalt = mix(
        rgb(34, 16, 19),
        rgb(143, 66, 46),
        crust * 0.63 + grit * 0.37,
    );
    let illumination = p.dot(uniforms.light);
    let direct = illumination.max(0.0);
    let ash_density = fractal_noise(p * 7.0 + Vec3::new(11.6, warp * 2.1, -time * 0.24));
    let ash = smoothstep(0.54, 0.77, ash_density) * (0.31 + 0.58 * direct);
    let surface = basalt * (0.25 + direct * 0.78)
        + rgb(185, 40, 22) * (glow * 0.62 * activity)
        + heat * (rivers * activity);
    let color = mix(surface, rgb(211, 157, 133) * (0.11 + direct * 0.95), ash);
    atmosphere(color, fragment, rgb(174, 57, 39), illumination, 0.32)
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
