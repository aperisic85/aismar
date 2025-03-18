#[derive(Debug)]
pub enum RaconStatus {
    NotFitted,
    NotMonitored,
    Operational,
    Test,
    Unknown,
}

#[derive(Debug)]
pub enum LightStatus {
    None,
    On,
    Off,
    Reserved,
    Unknown,
}
