// pub mod engine;
// pub mod games;
// pub mod ui;

// New PhysicLaw Architecture
pub mod core {
    pub mod math;
    pub mod geometry;
    pub mod state;
    pub mod solve;
}

pub mod laws {
    pub mod registry;
    pub mod classical;
    pub mod continuum;
}

pub mod platform;


pub mod investigation {
    pub mod probe;
    pub mod viz;
}
