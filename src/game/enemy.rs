#[derive(Debug, Default)]
pub struct Enemy {
    pub threat_level: u8,
}

impl Enemy {
    pub fn new(threat_level: u8) -> Self {
        Self { threat_level }
    }
}
