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
}

pub mod simulation;
pub mod control;

#[cfg(feature = "grpc")]
pub mod grpc {
    tonic::include_proto!("khemoo.moo.v1");
}

#[cfg(feature = "grpc")]
pub mod server;

#[cfg(feature = "python")]
pub mod python;

