use graphics::math::{add, mul_scalar, Vec2d};
use piston_window::*;
use rand::prelude::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::time::Instant;

struct ReportingAllocator;

#[global_allocator]
static ALLOCATOR: ReportingAllocator = ReportingAllocator;

/// ## Custom allocator
/// Implementing the GlobalAlloc trait will help us to
/// inpect how long it takes to allocate memory on the heap.
/// We are not doing anything tricky here, just timing and defering
/// the actual allocation to the system default memory allocator
unsafe impl GlobalAlloc for ReportingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start = Instant::now();
        let ptr = System.alloc(layout);
        let end = Instant::now();
        let time_taken = end - start;
        let bytes_requested = layout.size();

        eprintln!("{}\t{}", bytes_requested, time_taken.as_nanos());
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

struct Particle {
    height: f64,
    width: f64,
    position: Vec2d<f64>,
    velocity: Vec2d<f64>,
    acceleration: Vec2d<f64>,
    color: [f32; 4],
}

impl Particle {
    fn new(world: &World) -> Self {
        let mut rng = thread_rng();
        let x = rng.gen_range(0.0..=world.width);
        let y = world.height;
        let x_velocity = 0.0;
        let y_velocity = rng.gen_range(-2.0..0.0);
        let x_acceleration = 0.0;
        let y_acceleration = rng.gen_range(0.0..0.15);

        Self {
            height: 4.0,
            width: 4.0,
            position: [x, y].into(),
            velocity: [x_velocity, y_velocity].into(),
            acceleration: [x_acceleration, y_acceleration].into(),
            color: [1.0, 1.0, 1.0, 0.99],
        }
    }

    fn update(&mut self) {
        self.velocity = add(self.velocity, self.acceleration);
        self.position = add(self.position, self.velocity);
        self.acceleration = mul_scalar(self.acceleration, 0.7);
        self.color[3] *= 0.995;
    }
}

struct World {
    current_turn: u64,
    particles: Vec<Box<Particle>>,
    height: f64,
    width: f64,
    rng: ThreadRng,
}

impl World {
    fn new(width: f64, height: f64) -> Self {
        Self {
            current_turn: 0,
            particles: Vec::<Box<Particle>>::new(),
            height,
            width,
            rng: thread_rng(),
        }
    }

    fn add_particles(&mut self, quantity: i32) {
        for _ in 0..quantity.abs() {
            // creates the particle on the stack
            let particle = Particle::new(&self);
            // moves the particle ownership to the heap
            // through the Box smart pointer and keep the
            // pointer reference on the stack
            let boxed_particle = Box::new(particle);
            // push the particle reference to our list of particles
            self.particles.push(boxed_particle);
        }
    }

    fn remove_particles(&mut self, n: i32) {
        for _ in 0..n.abs() {
            let mut to_delete = None;

            let iter = self.particles.iter().enumerate();
            for (i, particle) in iter {
                if particle.color[3] < 0.02 {
                    to_delete = Some(i);
                    break;
                }
            }

            // particles that are fading away,
            // remove them one by one.
            if let Some(i) = to_delete {
                self.particles.remove(i);
            } else {
                // otherwise, remove the oldest particle
                self.particles.remove(0);
            }
        }
    }

    fn update(&mut self) {
        let n = self.rng.gen_range(-3..=3);
        if n > 3 {
            self.add_particles(n);
        } else {
            self.remove_particles(n);
        }

        self.particles.shrink_to_fit();
        for particle in &mut self.particles {
            particle.update();
        }

        self.current_turn += 1;
    }
}
fn main() {
    let (width, height) = (1280.0, 960.0);
    let mut window: PistonWindow = WindowSettings::new("Particles", [width, height])
        .exit_on_esc(true)
        .build()
        .expect("Could not create window.");

    let mut world = World::new(width, height);
    world.add_particles(1000);

    while let Some(event) = window.next() {
        world.update();

        window.draw_2d(&event, |ctx, renderer, device| {
            clear([0.15, 0.17, 0.19, 0.9], renderer);

            for p in &mut world.particles {
                let size = [p.position[0], p.position[1], p.width, p.height];
                rectangle(p.color, size, ctx.transform, renderer);
            }
        });
    }
}
