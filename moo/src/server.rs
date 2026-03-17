use std::sync::Arc;
use std::sync::atomic::Ordering;
use tonic::{Request, Response, Status};
use crate::control::{CommandSender, SimCommand, SimMetrics};
use crate::grpc::simulation_control_server::SimulationControl;
use crate::grpc::{Empty, StepRequest, StateSnapshot, Status as SimStatus, ParamUpdate};

pub struct MooServer {
    sender: CommandSender,
    metrics: Arc<SimMetrics>,
}

impl MooServer {
    pub fn new(sender: CommandSender, metrics: Arc<SimMetrics>) -> Self {
        Self { sender, metrics }
    }

    fn send_cmd(&self, cmd: SimCommand) -> Result<(), Status> {
        self.sender.send(cmd).map_err(|_| Status::internal("Simulation loop disconnected"))
    }
}

#[tonic::async_trait]
impl SimulationControl for MooServer {
    async fn start(&self, _request: Request<Empty>) -> Result<Response<SimStatus>, Status> {
        self.send_cmd(SimCommand::Resume)?;
        Ok(Response::new(SimStatus { success: true, message: "Simulation started".into() }))
    }

    async fn pause(&self, _request: Request<Empty>) -> Result<Response<SimStatus>, Status> {
        self.send_cmd(SimCommand::Pause)?;
        Ok(Response::new(SimStatus { success: true, message: "Simulation paused".into() }))
    }

    async fn resume(&self, _request: Request<Empty>) -> Result<Response<SimStatus>, Status> {
        self.send_cmd(SimCommand::Resume)?;
        Ok(Response::new(SimStatus { success: true, message: "Simulation resumed".into() }))
    }

    async fn reset(&self, _request: Request<Empty>) -> Result<Response<SimStatus>, Status> {
        self.send_cmd(SimCommand::Reset)?;
        Ok(Response::new(SimStatus { success: true, message: "Simulation reset".into() }))
    }

    async fn step(&self, request: Request<StepRequest>) -> Result<Response<StateSnapshot>, Status> {
        let req = request.into_inner();
        self.send_cmd(SimCommand::Step(req.steps))?;
        Ok(Response::new(StateSnapshot {
            step_count: self.metrics.step_count.load(Ordering::Relaxed),
            particle_count: self.metrics.particle_count.load(Ordering::Relaxed),
        }))
    }

    async fn set_params(&self, request: Request<ParamUpdate>) -> Result<Response<SimStatus>, Status> {
        let req = request.into_inner();
        if let Some(dt) = req.dt {
            self.send_cmd(SimCommand::SetDt(dt))?;
        }
        if let Some(gy) = req.gravity_y {
            self.send_cmd(SimCommand::SetGravity(0.0, gy))?;
        }
        Ok(Response::new(SimStatus { success: true, message: "Params updated".into() }))
    }

    async fn get_state(&self, _request: Request<Empty>) -> Result<Response<StateSnapshot>, Status> {
        Ok(Response::new(StateSnapshot {
            step_count: self.metrics.step_count.load(Ordering::Relaxed),
            particle_count: self.metrics.particle_count.load(Ordering::Relaxed),
        }))
    }
}

pub async fn start_server(addr: std::net::SocketAddr, sender: CommandSender, metrics: Arc<SimMetrics>) -> Result<(), Box<dyn std::error::Error>> {
    let server = MooServer::new(sender, metrics);
    tonic::transport::Server::builder()
        .add_service(crate::grpc::simulation_control_server::SimulationControlServer::new(server))
        .serve(addr)
        .await?;
    Ok(())
}
