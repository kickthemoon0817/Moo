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
    pub mod continuum {
        // Placeholder
    }
}

pub mod platform {
    pub mod compute {
        // Placeholder
    }
    pub mod storage {
        // Placeholder
    }
}

pub mod investigation {
    pub mod probe;
    pub mod viz;
}
