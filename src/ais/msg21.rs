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
    NoLightOrNotMonitored,
    On,
    Off,
    FailOrReducedRange,
    Unknown,
}

#[derive(Debug)]
pub enum GeneralHealth {
    Good,
    Alarm,
    Unknown,
}
