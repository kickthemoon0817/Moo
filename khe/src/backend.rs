use moo::simulation::Simulation;

#[cfg(feature = "grpc")]
use moo::grpc::simulation_control_client::SimulationControlClient;
#[cfg(feature = "grpc")]
use tonic::transport::Channel;

pub enum SimBackend {
    Local(Simulation),
    #[cfg(feature = "grpc")]
    Remote(SimulationControlClient<Channel>),
}

impl SimBackend {
    pub fn is_local(&self) -> bool {
        match self {
            SimBackend::Local(_) => true,
            #[cfg(feature = "grpc")]
            SimBackend::Remote(_) => false,
        }
    }

    pub fn unwrap_local(&mut self) -> &mut Simulation {
        match self {
            SimBackend::Local(sim) => sim,
            #[cfg(feature = "grpc")]
            SimBackend::Remote(_) => panic!("Backend is Remote, cannot access Local Simulation!"),
        }
    }

    // Abstractions for common commands could go here
    // e.g. fn pause(&mut self) -> Result<(), ...>
}
