use macroquad::{prelude::*, rand::gen_range};

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

    fn update(&mut self, dt: f32) {
        let temp_curr_pos = self.pos; // aktuelle Position speichern

        // Gravitationsbeschleunigung
        self.acceleration.y += 100.0;
        // Beschleunigung berechnen. Wird errechnet aus dem Unterschied der alten und aktuellen Position.
        let velocity = self.pos - self.old_pos;

        // Verlet Formel
        let new_pos = self.pos + velocity + self.acceleration * dt * dt; // dt^2 da acc = m / s^2

        // Positionen aktuallisiern.
        self.old_pos = self.pos;

        self.pos = new_pos;
        // Acc zurücksetzen 
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

        let dist = delta.length(); // errechnet die Länge des Vektors

        let min_dist = self.radius + other.radius;

        if dist - min_dist < 0.0 && dist > 0.0 {
            let overlap = min_dist - dist;

            let direction = delta / dist;

            let correction = direction * overlap * 0.5;

            self.pos += correction;

            other.pos -= correction;

        }
    }
    fn draw(&self, color: Color) {
        draw_circle(self.pos[0], self.pos[1], self.radius, color);
    }
    
}

fn resolve_collision(particles: &mut Vec<Particle>) {
    for i in 0..particles.len() {
        for j in i+1..particles.len() {
            let (left, right) = particles.split_at_mut(j);
            left[i].resolve_collision(&mut right[0]);
        }
    }
}

fn update_particles(particles: &mut Vec<Particle>, dt: f32) {
    for particle in particles {
        particle.update(dt);
        particle.wall_constrains(screen_width(),screen_height());
    }
}

fn draw_particles(particles: &Vec<Particle>) {
    for particle in particles {
        particle.draw(WHITE)
    }
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
    let mut particles: Vec<Particle> = vec![
        Particle::new(Vec2::new(100.0, 100.0), Vec2::new(10., 0.), 10.0),
        Particle::new(Vec2::new(200.0, 100.0), Vec2::new(-10., 0.), 5.0),
    ];

    let mut fps_history: Vec<f32> = Vec::new();

    for i in 0..10 {
        particles.push(spawn_high_particle());
    }

    const FIXED_DT: f32 = 1.0 / 60.0;
    loop {
        clear_background(BLACK);
        
        if is_key_pressed(KeyCode::Key1) {
            for i in 0..10 {
                particles.push(spawn_high_particle());
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

        // particles.push(spawn_high_particle());

        update_particles(&mut particles, FIXED_DT);

        resolve_collision(&mut particles);

        draw_particles(&particles);

        next_frame().await
    }
}