#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolVersion { pub major: u16, pub minor: u16 }

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FeatureSet(pub u64);
impl FeatureSet {
    pub const AUDIO: u64 = 1 << 0;
    pub const VIDEO: u64 = 1 << 1;
    pub const PHOTO: u64 = 1 << 2;
    pub fn contains(self, feature: u64) -> bool { self.0 & feature == feature }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceProfile {
    pub instance_name: String,
    pub model: Model,
    pub device_id: [u8; 6],
    pub protocol: ProtocolVersion,
    pub features: FeatureSet,
}

impl DeviceProfile {
    pub fn device_id_text(&self) -> String { self.device_id.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(":") }
}
