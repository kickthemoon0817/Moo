// pub mod engine;
// pub mod games;
// pub mod ui;

// New PhysicLaw Architecture
pub mod core {
    pub mod geometry;
    pub mod math;
    pub mod solve;
    pub mod state;
}

pub mod laws {
    pub mod classical;
    pub mod continuum;
    pub mod registry;
}

pub mod platform;

pub mod investigation {
    pub mod probe;
    pub mod viz;
}
