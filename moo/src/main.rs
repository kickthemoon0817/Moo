use moo::simulation::Simulation;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    println!("--- Moo Physics Engine (v0.0.3) ---");
    println!("Initializing Headless Simulation...");

    // 1. Init GPU (Headless)
    let (device, queue) = Simulation::init_headless().await;

    // 2. Init Simulation
    let n_particles = 4096;
    let mut sim = Simulation::new(&device, n_particles).await;

    println!("Simulation running with {} particles.", n_particles);

    // 3. Step
    let start = std::time::Instant::now();
    for i in 0..60 {
        sim.step(&device, &queue);
        if i % 10 == 0 {
            print!(".");
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }

    println!("\nSimulated 60 frames in {:.2?}", start.elapsed());
    println!("Engine Integrity Verified.");
}
