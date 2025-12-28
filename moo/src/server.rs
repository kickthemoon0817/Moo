use tonic::{Request, Response, Status};
use crate::control::{CommandSender, SimCommand};
use crate::grpc::simulation_control_server::SimulationControl;
use crate::grpc::{Empty, StepRequest, StateSnapshot, Status as SimStatus, ParamUpdate};

pub struct MooServer {
    sender: CommandSender,
}

impl MooServer {
    pub fn new(sender: CommandSender) -> Self {
        Self { sender }
    }
}

#[tonic::async_trait]
impl SimulationControl for MooServer {
    async fn start(&self, _request: Request<Empty>) -> Result<Response<SimStatus>, Status> {
        self.sender.send(SimCommand::Resume);
        Ok(Response::new(SimStatus { success: true, message: "Simulation started".into() }))
    }

    async fn pause(&self, _request: Request<Empty>) -> Result<Response<SimStatus>, Status> {
        self.sender.send(SimCommand::Pause);
        Ok(Response::new(SimStatus { success: true, message: "Simulation paused".into() }))
    }

    async fn resume(&self, _request: Request<Empty>) -> Result<Response<SimStatus>, Status> {
        self.sender.send(SimCommand::Resume);
        Ok(Response::new(SimStatus { success: true, message: "Simulation resumed".into() }))
    }

    async fn reset(&self, _request: Request<Empty>) -> Result<Response<SimStatus>, Status> {
        self.sender.send(SimCommand::Reset);
        Ok(Response::new(SimStatus { success: true, message: "Simulation reset".into() }))
    }

    async fn step(&self, request: Request<StepRequest>) -> Result<Response<StateSnapshot>, Status> {
        let req = request.into_inner();
        self.sender.send(SimCommand::Step(req.steps));
        // Note: Returning snapshot immediately is tricky as step is async on another thread.
        // For now, return empty snapshot or last known state.
        Ok(Response::new(StateSnapshot { 
            step_count: 0, 
            particle_count: 0 
        }))
    }

    async fn set_params(&self, request: Request<ParamUpdate>) -> Result<Response<SimStatus>, Status> {
        let req = request.into_inner();
        if let Some(dt) = req.dt {
            self.sender.send(SimCommand::SetDt(dt));
        }
        if let Some(gy) = req.gravity_y {
            self.sender.send(SimCommand::SetGravity(0.0, gy));
        }
        Ok(Response::new(SimStatus { success: true, message: "Params updated".into() }))
    }

    async fn get_state(&self, _request: Request<Empty>) -> Result<Response<StateSnapshot>, Status> {
        // Limitation: One-way command queue. Cannot query state easily without a return channel or shared memory.
        // Phase 17 limitation acknowledgment.
        Ok(Response::new(StateSnapshot { 
            step_count: 0, 
            particle_count: 0 
        }))
    }
}

pub async fn start_server(addr: std::net::SocketAddr, sender: CommandSender) -> Result<(), Box<dyn std::error::Error>> {
    let server = MooServer::new(sender);
    tonic::transport::Server::builder()
        .add_service(crate::grpc::simulation_control_server::SimulationControlServer::new(server))
        .serve(addr)
        .await?;
    Ok(())
}
