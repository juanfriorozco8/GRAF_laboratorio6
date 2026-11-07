use raylib::prelude::*;

use crate::Scene;

const GOLD: Color = Color::new(222, 181, 103, 255);
const INK: Color = Color::new(239, 239, 235, 255);
const MUTED: Color = Color::new(163, 169, 177, 255);
const RULE: Color = Color::new(40, 44, 51, 255);
const PANEL: Color = Color::new(13, 16, 21, 252);
const MARGIN: i32 = 34;

pub struct Hud {
    font: Font,
}

#[derive(Clone, Copy)]
pub struct HudState {
    pub scene: Scene,
    pub effect: f32,
    pub paused: bool,
    pub shadows: bool,
    pub show_help: bool,
}

impl Hud {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<Self, raylib::error::Error> {
        let font = rl.load_font_from_memory(
            thread,
            ".ttf",
            include_bytes!("../assets/fonts/Lato-Medium.ttf"),
            48,
            None,
        )?;
        font.texture()
            .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        Ok(Self { font })
    }

    pub fn draw(&self, drawing: &mut impl RaylibDraw, width: i32, footer: i32, state: HudState) {
        let scene = state.scene;
        let right = width - MARGIN;
        let number = format!("ARCHIVO ESTELAR   /   {:02}", scene.index() + 1);
        self.text(drawing, &number, (MARGIN, 19), (12.0, 2.4), GOLD);
        self.text(drawing, scene.title(), (MARGIN, 39), (35.0, 3.0), INK);
        self.text(drawing, scene.kind(), (MARGIN, 80), (13.0, 1.3), MUTED);
        drawing.draw_line(MARGIN, 106, MARGIN + 32, 106, GOLD);
        let status = if state.paused { "EN PAUSA" } else { "EN VIVO" };
        let status_x = right - self.font.measure_text(status, 12.0, 1.5).x.ceil() as i32;
        drawing.draw_circle(status_x - 10, 28, 2.0, GOLD);
        self.text(drawing, status, (status_x, 20), (12.0, 1.5), MUTED);

        drawing.draw_line(MARGIN, footer, right, footer, RULE);
        let tab_width = (right - MARGIN) / Scene::ALL.len() as i32;
        for tab in Scene::ALL {
            let x = MARGIN + tab_width * tab.index() as i32;
            let active = tab == scene;
            if active {
                drawing.draw_line(x, footer, x + tab_width - 20, footer, GOLD);
            }
            self.text(
                drawing,
                &format!("{:02}", tab.index() + 1),
                (x, footer + 12),
                (12.0, 0.5),
                if active { GOLD } else { MUTED },
            );
            self.text(
                drawing,
                tab.short_name(),
                (x + 27, footer + 10),
                (16.0, 0.3),
                if active { INK } else { MUTED },
            );
        }
        self.text(
            drawing,
            &format!(
                "ARRIBA / ABAJO     {}   {:02}%",
                scene.effect_name(),
                (state.effect * 100.0).round() as u8
            ),
            (MARGIN, footer + 41),
            (12.0, 0.8),
            MUTED,
        );
        let help = if state.show_help {
            "[H]  CERRAR AYUDA"
        } else {
            "[H]  CONTROLES"
        };
        let help_x = right - self.font.measure_text(help, 12.0, 0.8).x.ceil() as i32;
        self.text(drawing, help, (help_x, footer + 41), (12.0, 0.8), GOLD);

        if state.show_help {
            self.draw_help(drawing, right, state);
        }
    }

    fn draw_help(&self, drawing: &mut impl RaylibDraw, right: i32, state: HudState) {
        let width = 398;
        let left = right - width;
        let top = 118;
        let border = Rectangle::new(left as f32, top as f32, width as f32, 254.0);
        drawing.draw_rectangle_rec(border, PANEL);
        drawing.draw_rectangle_lines_ex(border, 1.0, RULE);
        drawing.draw_rectangle(left, top, 38, 1, GOLD);
        self.text(
            drawing,
            "CONTROLES DE VISTA",
            (left + 18, top + 14),
            (15.0, 1.7),
            GOLD,
        );
        let light = if state.scene == Scene::Soles {
            "Los soles emiten luz propia"
        } else {
            "Girar la luz a ambos lados"
        };
        let shadows = if state.shadows {
            "Sombras de Scarif: activas"
        } else {
            "Sombras de Scarif: apagadas"
        };
        let rows = [
            ("1 - 5", "Cambiar de cuerpo"),
            ("IZQ / DER", light),
            ("ARRIBA / ABAJO", state.scene.effect_name()),
            ("ESPACIO", "Pausar o reanudar el material"),
            ("S", shadows),
            ("R", "Restaurar luz, tiempo y efecto"),
            ("ESC", "Cerrar el visor"),
        ];
        for (index, (key, description)) in rows.into_iter().enumerate() {
            let y = top + 47 + index as i32 * 26;
            self.text(drawing, key, (left + 18, y), (11.0, 0.6), MUTED);
            self.text(drawing, description, (left + 145, y - 2), (15.0, 0.1), INK);
        }
    }

    fn text(
        &self,
        drawing: &mut impl RaylibDraw,
        text: &str,
        (x, y): (i32, i32),
        (size, spacing): (f32, f32),
        color: Color,
    ) {
        drawing.draw_text_ex(
            &self.font,
            text,
            Vector2::new(x as f32, y as f32),
            size,
            spacing,
            color,
        );
    }
}
