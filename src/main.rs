use rand::Rng;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("Pendulum"),
        ..Default::default()
    };
    eframe::run_native(
        "Pendulum",
        native_options,
        Box::new(|_cc| Box::new(App::default())),
    )
}

const STEPS_PER_FRAME: usize = 100;

#[derive(Debug)]
struct App {
    root_pos: egui::Pos2,
    paused: bool,
    gravity: f32,
    pendulums: Vec<PendulumArm>,
}
impl Default for App {
    fn default() -> Self {
        Self {
            root_pos: egui::Pos2::ZERO,
            paused: false,
            gravity: 0.1,
            pendulums: vec![],
        }
    }
}
impl App {
    fn physics_step(&mut self, t: f32) {
        // let mut rod_tensions = vec![0.0; self.pendulums.len()];
        // let mut weighted_com = egui::Pos2::ZERO;
        // let mut total_mass = 0.0;
        // for i in (0..self.pendulums.len()).rev() {
        //     let pend = &self.pendulums[i];
        //     weighted_com += pend.pos.to_vec2() * pend.mass;
        //     total_mass += pend.mass;
        //     let com = egui::Pos2::from(weighted_com / total_mass);
        //     rod_tensions[i] =
        // }

        let mut last_pos = self.root_pos;
        for i in 0..self.pendulums.len() {
            let pend = &mut self.pendulums[i];
            // Update position
            pend.pos += pend.vel * t;
            let pole_vector = (pend.pos - last_pos).normalized();
            pend.pos = last_pos + pole_vector * pend.length;

            // Update velocity
            pend.vel.y += self.gravity * t;
            let accel = pole_vector * pend.vel.dot(pole_vector) * pend.mass * t;
            pend.vel -= accel / pend.mass;
            last_pos = pend.pos;
            if i > 0 {
                let prev = &mut self.pendulums[i - 1];
                prev.vel += accel / prev.mass;
            }
        }
    }
    fn end(&self) -> egui::Pos2 {
        match self.pendulums.last() {
            Some(pend) => pend.pos,
            None => self.root_pos,
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();

        if !self.paused {
            // Update physics
            for _ in 0..STEPS_PER_FRAME {
                self.physics_step((STEPS_PER_FRAME as f32).recip());
            }
        }

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.heading("Simulation");
            ui.checkbox(&mut self.paused, "Pause");
            ui.label("Gravity");
            ui.add(egui::DragValue::new(&mut self.gravity).speed(0.01));
            ui.separator();
            let potential_energy: f32 = self
                .pendulums
                .iter()
                .map(|pend| pend.gravitational_potential_energy(self.gravity))
                .sum();
            let kinetic_energy: f32 = self
                .pendulums
                .iter()
                .map(|pend| pend.kinetic_energy())
                .sum();
            ui.label(format!("Potential energy: {potential_energy}"));
            ui.label(format!("Kinetic energy: {kinetic_energy}"));
            ui.label(format!(
                "Total energy: {}",
                potential_energy + kinetic_energy,
            ));
            ui.separator();
            ui.heading("Pendulums");
            if ui.button("Add pendulum").clicked() {
                self.pendulums.push(PendulumArm::new(self.end()));
            }
            self.pendulums.retain_mut(|pend| {
                ui.horizontal(|ui| {
                    ui.label("Length");
                    ui.add(egui::DragValue::new(&mut pend.length));
                    ui.label("Mass");
                    ui.add(egui::DragValue::new(&mut pend.mass));
                    ui.label("Drag");
                    ui.add(egui::DragValue::new(&mut pend.drag));
                    !ui.button("🗑").clicked()
                })
                .inner
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.root_pos = ui.available_rect_before_wrap().center_top();

            let p = ui.painter();
            let mut last_pos = self.root_pos;
            for (i, pend) in self.pendulums.iter().enumerate() {
                p.line_segment(
                    [last_pos, pend.pos],
                    egui::Stroke {
                        width: 2.0,
                        color: get_color(i),
                    },
                );
                last_pos = pend.pos
            }
            for (i, pend) in self.pendulums.iter().enumerate() {
                p.circle_filled(pend.pos, pend.mass.sqrt(), get_color(i));
            }
        });
    }
}

#[derive(Debug, Copy, Clone)]
struct PendulumArm {
    length: f32,
    mass: f32,
    drag: f32,

    pos: egui::Pos2,
    vel: egui::Vec2,
}
impl PendulumArm {
    pub fn new(root: egui::Pos2) -> Self {
        let length = 50.0;
        let angle = rand::thread_rng().gen_range(0.0..std::f32::consts::PI);
        let pos = root + egui::vec2(angle.cos(), angle.sin()) * length;

        Self {
            length,
            mass: 25.0,
            drag: 0.0,

            pos,
            vel: egui::Vec2::ZERO,
        }
    }
    pub fn kinetic_energy(&self) -> f32 {
        self.vel.length_sq() * self.mass / 2.0
    }
    pub fn gravitational_potential_energy(&self, gravity: f32) -> f32 {
        -self.pos.y * self.mass * gravity
    }
}

fn get_color(i: usize) -> egui::Color32 {
    match colorous::CATEGORY10.get(i) {
        Some(c) => egui::Color32::from_rgb(c.r, c.g, c.b),
        None => egui::Color32::GRAY,
    }
}
