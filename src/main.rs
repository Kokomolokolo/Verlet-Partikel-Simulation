use macroquad::{prelude::*, rand::gen_range};

//use std::collections::HashMap;
use rustc_hash::FxHashMap as HashMap; //schnellere Hashmap

struct Particle {
    pos: Vec2,
    old_pos: Vec2,
    acceleration: Vec2,
    radius: f32
}
impl Particle {
    fn new(pos: Vec2, velocity: Vec2, radius: f32) -> Self {
        Self {
            pos: pos,
            old_pos: pos - velocity, // alte Position geändert für eine Startgeschwindigkeit
            acceleration: Vec2::ZERO,
            radius,
        }
    }
    fn apply_foce(&mut self, f: Vec2) {
        self.acceleration += f;
    }

    fn update(&mut self, dt: f32, gravity: bool) {
        // Gravitationsbeschleunigung
        if gravity { self.apply_foce(vec2(0., 100.)); } else { self.apply_foce(vec2(0., 0.)); } // nur Gravitation
        self.apply_foce(vec2(0., 100.));
        // Beschleunigung berechnen. Wird errechnet aus dem Unterschied der alten und aktuellen Position.
        let velocity = self.pos - self.old_pos;

        // Verlet Formel
        let new_pos = self.pos + velocity + self.acceleration * dt * dt; // dt^2 da acc = m / s^2

        // Positionen aktuallisiern.
        self.old_pos = self.pos;

        self.pos = new_pos;

        self.acceleration = vec2(0., 0.)
    }
    
    fn wall_constrains(&mut self, width: f32, height: f32) {
        let damping = 0.5;
        
        // Boden
        if self.pos.y + self.radius > height {
            self.pos.y = height - self.radius;
            self.old_pos.y = self.pos.y + (self.pos.y - self.old_pos.y) * damping;
        }
        
        // Decke
        if self.pos.y - self.radius < 0.0 {
            self.pos.y = self.radius;
            self.old_pos.y = self.pos.y + (self.pos.y - self.old_pos.y) * damping;
        }
        
        // Rechte Wand
        if self.pos.x + self.radius > width {
            self.pos.x = width - self.radius;
            self.old_pos.x = self.pos.x + (self.pos.x - self.old_pos.x) * damping;
        }
        
        // Linke Wand
        if self.pos.x - self.radius < 0.0 {
            self.pos.x = self.radius;
            self.old_pos.x = self.pos.x + (self.pos.x - self.old_pos.x) * damping;
        }
    }

    fn resolve_collision(&mut self, other: &mut Particle) {
        let delta = self.pos - other.pos; // Wie weit sind die beiden auseinander?

        let dist_squared = delta.length_squared(); // errechnet die Länge des Vektors

        let min_dist = self.radius + other.radius;
        let min_dist_squared = min_dist * min_dist;

        if dist_squared - min_dist_squared < 0.0 && dist_squared > 0.0 {
            let dist = dist_squared.sqrt();
            
            let overlap = min_dist - dist;

            let direction = delta / dist;

            let correction = direction * overlap * 0.5;

            self.pos += correction;

            other.pos -= correction;

        }
    }
    fn draw(&self) {
        let velocity = self.pos - self.old_pos;
        let speed_squared = velocity.length_squared();  // ← Nur x² + y²

        let color = speed_to_color(speed_squared);

        draw_circle(self.pos[0], self.pos[1], self.radius, color);
    }
    fn calm(&mut self) {
        self.old_pos = self.pos;
    }
    
}
// veraltet aber lustig weil es viel weniger Performant ist
fn resolve_collision(particles: &mut Vec<Particle>) {
    for i in 0..particles.len() {
        for j in i+1..particles.len() {
            let (left, right) = particles.split_at_mut(j);
            left[i].resolve_collision(&mut right[0]);
        }
    }
}

fn fill_grid<'a>(hashie: &'a mut HashMap<(i32, i32), Vec<usize>>, particles: &Vec<Particle>, cell_size: f32) -> &'a mut HashMap<(i32, i32), Vec<usize>> {
    //let mut hashie: HashMap<(i32, i32), Vec<usize>> = HashMap::default();

    for (index, particle) in particles.iter().enumerate() {
        let cell_x = (particle.pos.x / cell_size) as i32;
        let cell_y = (particle.pos.y / cell_size) as i32;
        let cell_key = (cell_x, cell_y);

        hashie.entry(cell_key)
            .or_insert_with(|| Vec::with_capacity(8))
            .push(index);
    }
    hashie
}

fn resolve_collision_with_grid(particles: &mut Vec<Particle>, grid: &HashMap<(i32, i32), Vec<usize>>) {
    for (cell_key, cell_indices) in grid.iter() { // Über jede Celle wird iteriert.
        for i in 0..cell_indices.len() { // Über alle Particel in einer Cell werden iteriert und verglichen.
            for j in i+1..cell_indices.len() {
                let idx_a = cell_indices[i];  // Konvertiert denn cellindex in einen echten index.
                let idx_b = cell_indices[j];  
                // Finde den größeren und kleineren Index
                let (small_idx, large_idx) = if idx_a < idx_b {
                    (idx_a, idx_b)
                } else {
                    (idx_b, idx_a)
                };
                
                let (left, right) = particles.split_at_mut(large_idx);
                left[small_idx].resolve_collision(&mut right[0]);
            }
        }
        let neighbors = [ // da wir von oben rechts nach unten links gehen, werden nur diese gechekckt, um dopplungen zu vermeiden. Claude <3
            (cell_key.0 + 1, cell_key.1), // rechts
            (cell_key.0 + 1, cell_key.1 + 1), // unten rechts
            (cell_key.0, cell_key.1 + 1), // unten
            (cell_key.0 - 1, cell_key.1 + 1) // links unten
        ];
        for neighbor_key in neighbors.iter() {
            if let Some(neighbor_indicies) = grid.get(neighbor_key) { // wenn die Nachbarzelle existiert
                for &idx_a in cell_indices.iter() {
                    for &idx_b in neighbor_indicies.iter() {
                        if idx_a == idx_b { continue; } // wenn es die gleichen sind: überspringen
                        let (small_idx, large_idx) = if idx_a < idx_b {
                            (idx_a, idx_b)
                        } else {
                            (idx_b, idx_a)
                        };
                        
                        let (left, right) = particles.split_at_mut(large_idx);
                        left[small_idx].resolve_collision(&mut right[0]);
                    }
                }
            }
        }
    }
}

fn update_particles(particles: &mut Vec<Particle>, dt: f32, bool_gravity: bool) {
    for particle in particles {
        particle.update(dt, bool_gravity);
        particle.wall_constrains(screen_width(),screen_height());
    }
}

fn draw_particles(particles: &Vec<Particle>) {
    for particle in particles {
        particle.draw()
    }
}
fn speed_to_color(speed: f32) -> Color {
    let normalized = (speed / 25.0).min(1.0);
    let r = normalized.powf(1.5);
    let g = (1.0 - normalized).powf(2.0);
    Color::new(r, g, 1.0 - r, 1.0)
}

fn mouse_push_force(particles: &mut Vec<Particle>, mouse_pos: Vec2) { // geht nicht
    let force = 3000.;
    let force_radius = 100.;

    for particle in particles.iter_mut() {
        let delta = particle.pos - mouse_pos;
        let distance = delta.length();

        if distance < force_radius && distance > 0.1 {
            // Je näher, desto stärker.
            let force_strength = 20. * force / distance;
            let direction = delta / distance;

            particle.apply_foce(direction * force_strength);
        }
    }
}
fn mouse_spawn_particle(particles: &mut Vec<Particle>, mouse_pos: Vec2) {
    let particle = Particle::new(
        Vec2::new(mouse_pos.x, mouse_pos.y),
        Vec2::new(gen_range(-2., 2.), gen_range(-2., 2.)),
        gen_range(2.0, 7.0)
    );
    particles.push(particle);
}
fn spawn_high_particle() -> Particle {
    Particle::new(
        Vec2::new(gen_range(0., screen_width()), gen_range(0., 100.)),
        Vec2::new(gen_range(-2., 2.), gen_range(-2., 2.)),
        gen_range(2.0, 7.0)
    )
}
#[macroquad::main("Verlet Partikel")]
async fn main() {
    let mut particles: Vec<Particle> = Vec::with_capacity(5000);

    let mut fps_history: Vec<f32> = Vec::new();

    let mut bool_gravity = true;

    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::default();

    for _i in 0..1 {
        particles.push(spawn_high_particle());
    }

    const FIXED_DT: f32 = 1.0 / 60.0;
    loop {
        //clear_background(BLACK);
        draw_rectangle(0., 0., screen_width(), screen_height(), Color::new(0., 0., 0., 0.1));
        // Key Inputs
        if is_key_pressed(KeyCode::Key1) {
            for _i in 0..100 {
                particles.push(spawn_high_particle());
            }
        }
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = Vec2::new(mouse_position().0, mouse_position().1);
            mouse_spawn_particle(&mut particles, mouse_pos);
        }
        if is_key_down(KeyCode::P) {
            let mouse_pos = Vec2::new(mouse_position().0, mouse_position().1);
            mouse_push_force(&mut particles, mouse_pos);
        }
        if is_key_pressed(KeyCode::R) {
            particles = vec![];
        }
        if is_key_pressed(KeyCode::G) {
            if bool_gravity {
                bool_gravity = false;
            } else {
                bool_gravity = true;
            }
        }
        if is_key_pressed(KeyCode::C) {
            for p in &mut particles {
                p.calm()
            }       
        }
        // HUD
        let fps = get_fps() as f32;
        fps_history.push(fps);
        if fps_history.len() > 30 {
            fps_history.remove(0);
        }
        let avg_fps = fps_history.iter().sum::<f32>() / fps_history.len() as f32;
        let fps_color = if avg_fps > 55.0 { GREEN } else if avg_fps > 30.0 { YELLOW } else { RED };
        draw_text(
            &format!("FPS: {:.1} | Partikel: {}", avg_fps, particles.len()),
            10.0, 20.0, 24.0, fps_color
        );
        
        // let mut substeps = 4;
        // if avg_fps < 50. {
        //     substeps = 2;
        // }

        let substeps = if particles.len() > 80000 { 2 } else { 4 }; // mehere Substeps für mehr Stabilität. Weniger Substeps für bessere performance bei hoher Partikelanzahl.
        let sub_dt = FIXED_DT / substeps as f32;
        for _ in 0..substeps {
            update_particles(&mut particles, sub_dt, bool_gravity);

            grid.clear();
            let cell_size = 14.0;
            fill_grid(&mut grid, &particles, cell_size);

            //let grid = build_particle_hashmap(&particles, cell_size);
            resolve_collision_with_grid(&mut particles, &grid);
        }
        // resolve_collision(&mut particles);

        draw_particles(&particles);

        next_frame().await
    }
}