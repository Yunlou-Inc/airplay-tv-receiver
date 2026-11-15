use crate::profile::DeviceProfile;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind { AirPlay, Raop }
impl ServiceKind { pub fn service_type(self) -> &'static str { match self { Self::AirPlay => "_airplay._tcp", Self::Raop => "_raop._tcp" } } }

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TxtRecord(BTreeMap<String, String>);
impl TxtRecord {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<(), DiscoveryError> {
        let (key, value) = (key.into(), value.into());
        if key.is_empty() || key.len() > 63 || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') { return Err(DiscoveryError::InvalidTxtKey); }
        if key.len() + 1 + value.len() > 255 { return Err(DiscoveryError::TxtEntryTooLong); }
        self.0.insert(key, value); Ok(())
    }
    pub fn get(&self, key: &str) -> Option<&str> { self.0.get(key).map(String::as_str) }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> { self.0.iter().map(|(k,v)| (k.as_str(),v.as_str())) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryRecord { pub kind: ServiceKind, pub instance: String, pub port: u16, pub txt: TxtRecord }

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveryError { EmptyInstance, InvalidPort, InvalidTxtKey, TxtEntryTooLong }
impl fmt::Display for DiscoveryError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "discovery record error: {self:?}") } }
impl std::error::Error for DiscoveryError {}

impl DiscoveryRecord {
    pub fn from_profile(profile: &DeviceProfile, kind: ServiceKind, port: u16) -> Result<Self, DiscoveryError> {
        if profile.instance_name.trim().is_empty() { return Err(DiscoveryError::EmptyInstance); }
        if port == 0 { return Err(DiscoveryError::InvalidPort); }
        let mut txt = TxtRecord::default();
        txt.insert("model", &profile.model.0)?;
        txt.insert("deviceid", profile.device_id_text())?;
        txt.insert("features", format!("0x{:X}", profile.features.0))?;
        txt.insert("srcvers", format!("{}.{}", profile.protocol.major, profile.protocol.minor))?;
        let instance = match kind { ServiceKind::AirPlay => profile.instance_name.clone(), ServiceKind::Raop => format!("{}@{}", profile.device_id_text().replace(':', ""), profile.instance_name) };
        Ok(Self { kind, instance, port, txt })
    }
}

#[cfg(test)]
mod tests {
 use super::*; use crate::profile::*;
 fn profile() -> DeviceProfile { DeviceProfile { instance_name:"TV".into(), model:Model("Receiver1,1".into()), device_id:[0,1,2,3,4,5], protocol:ProtocolVersion{major:1,minor:0}, features:FeatureSet(FeatureSet::AUDIO) } }
 #[test] fn builds_both_service_records() { let a=DiscoveryRecord::from_profile(&profile(),ServiceKind::AirPlay,7000).unwrap(); let r=DiscoveryRecord::from_profile(&profile(),ServiceKind::Raop,5000).unwrap(); assert_eq!(a.kind.service_type(),"_airplay._tcp"); assert_eq!(r.kind.service_type(),"_raop._tcp"); assert_eq!(a.txt.get("deviceid"),Some("00:01:02:03:04:05")); }
 #[test] fn rejects_zero_port() { assert_eq!(DiscoveryRecord::from_profile(&profile(),ServiceKind::AirPlay,0),Err(DiscoveryError::InvalidPort)); }
}

#[cfg(test)]
mod boundary_tests {
 use super::*;
 #[test] fn txt_rejects_invalid_and_oversize_inputs() {
  let mut txt=TxtRecord::default();
  assert_eq!(txt.insert("bad_key","x"),Err(DiscoveryError::InvalidTxtKey));
  assert_eq!(txt.insert("k","x".repeat(254)),Err(DiscoveryError::TxtEntryTooLong));
 }
 #[test] fn duplicate_txt_key_replaces_deterministically() {
  let mut txt=TxtRecord::default(); txt.insert("model","one").unwrap(); txt.insert("model","two").unwrap();
  assert_eq!(txt.get("model"),Some("two")); assert_eq!(txt.iter().count(),1);
 }
}
