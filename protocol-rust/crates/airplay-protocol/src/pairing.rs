//! Vendor-neutral pairing authorization framework.
//! Apple-private proof/key-derivation steps are intentionally delegated to `PairingProtocol`.
use std::collections::{BTreeMap, VecDeque};

pub type PeerId = String;
pub type RequestId = u64;
pub type SessionId = [u8; 16];

pub trait Clock { fn now_ms(&self) -> u64; }
pub trait Entropy { fn fill(&mut self, out: &mut [u8]) -> Result<(), PairingError>; }

/// Opaque long-term identity. Production adapters should keep private operations in Android Keystore.
pub trait DeviceIdentity {
    fn public_id(&self) -> &[u8];
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, PairingError>;
}

/// Test-only identity. This is not cryptographically secure and must not be used in production.
#[derive(Clone, Debug)]
pub struct InMemoryIdentity { public: Vec<u8>, secret: Vec<u8> }
impl InMemoryIdentity {
    pub fn for_tests(public: Vec<u8>, secret: Vec<u8>) -> Self { Self { public, secret } }
}
impl DeviceIdentity for InMemoryIdentity {
    fn public_id(&self) -> &[u8] { &self.public }
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, PairingError> {
        let mut out = self.secret.clone(); out.extend_from_slice(message); Ok(out)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolError { Unsupported, InvalidProof }
/// Adapter for legitimately sourced AirPlay pairing details; no private algorithm is guessed here.
pub trait PairingProtocol {
    fn verify(&self, peer: &PeerId, challenge: &[u8], proof: &[u8], identity: &dyn DeviceIdentity)
        -> Result<Vec<u8>, ProtocolError>;
}
#[derive(Clone, Debug, Default)]
pub struct UnsupportedAirPlayProtocol;
impl PairingProtocol for UnsupportedAirPlayProtocol {
    fn verify(&self, _: &PeerId, _: &[u8], _: &[u8], _: &dyn DeviceIdentity) -> Result<Vec<u8>, ProtocolError> { Err(ProtocolError::Unsupported) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairingState { PendingUser, Confirmed, Paired, Revoked, Expired }
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairingRequest { pub id: RequestId, pub peer: PeerId, pub challenge: Vec<u8>, pub expires_at_ms: u64, pub state: PairingState }
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PairingError { Busy, NotFound, WrongState, WrongPin, Expired, Revoked, UnsupportedProtocol, InvalidProof, EntropyFailure, UnknownPeer }

#[derive(Clone, Debug)]
struct Grant { credential: Vec<u8>, expires_at_ms: u64, revoked: bool }
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Credential { pub peer: PeerId, pub value: Vec<u8>, pub expires_at_ms: u64 }

pub struct PairingManager { next_id: RequestId, active: Option<PairingRequest>, grants: BTreeMap<PeerId, Grant>, request_ttl_ms: u64, credential_ttl_ms: u64 }
impl PairingManager {
    pub fn new(request_ttl_ms: u64, credential_ttl_ms: u64) -> Self { Self { next_id: 1, active: None, grants: BTreeMap::new(), request_ttl_ms, credential_ttl_ms } }
    pub fn request(&mut self, peer: PeerId, now: u64, entropy: &mut dyn Entropy) -> Result<PairingRequest, PairingError> {
        self.expire(now);
        if self.active.is_some() { return Err(PairingError::Busy); }
        let mut challenge=vec![0;32]; entropy.fill(&mut challenge)?;
        let r=PairingRequest{id:self.next_id,peer,challenge,expires_at_ms:now.saturating_add(self.request_ttl_ms),state:PairingState::PendingUser};
        self.next_id=self.next_id.saturating_add(1); self.active=Some(r.clone()); Ok(r)
    }
    pub fn confirm(&mut self, id: RequestId, entered_pin: &str, displayed_pin: &str, now:u64) -> Result<(),PairingError>{
        self.expire(now); let r=self.active.as_mut().ok_or(PairingError::NotFound)?;
        if r.id!=id{return Err(PairingError::NotFound)} if r.state!=PairingState::PendingUser{return Err(PairingError::WrongState)}
        if !pin_eq(entered_pin.as_bytes(),displayed_pin.as_bytes()){return Err(PairingError::WrongPin)} r.state=PairingState::Confirmed; Ok(())
    }
    pub fn verify(&mut self,id:RequestId,proof:&[u8],now:u64,protocol:&dyn PairingProtocol,identity:&dyn DeviceIdentity)->Result<Credential,PairingError>{
        self.expire(now); let r=self.active.as_ref().ok_or(PairingError::NotFound)?;
        if r.id!=id{return Err(PairingError::NotFound)} if r.state!=PairingState::Confirmed{return Err(PairingError::WrongState)}
        let peer=r.peer.clone(); let c=protocol.verify(&peer,&r.challenge,proof,identity).map_err(|e|match e{ProtocolError::Unsupported=>PairingError::UnsupportedProtocol,ProtocolError::InvalidProof=>PairingError::InvalidProof})?;
        let expires=now.saturating_add(self.credential_ttl_ms); self.grants.insert(peer.clone(),Grant{credential:c.clone(),expires_at_ms:expires,revoked:false}); self.active=None; Ok(Credential{peer,value:c,expires_at_ms:expires})
    }
    pub fn reconnect(&mut self,peer:&str,credential:&[u8],now:u64)->Result<(),PairingError>{
        let g=self.grants.get(peer).ok_or(PairingError::UnknownPeer)?; if g.revoked{return Err(PairingError::Revoked)} if now>=g.expires_at_ms{return Err(PairingError::Expired)} if !pin_eq(&g.credential,credential){return Err(PairingError::InvalidProof)} Ok(())
    }
    pub fn revoke(&mut self,peer:&str){if let Some(g)=self.grants.get_mut(peer){g.revoked=true;g.credential.fill(0)}}
    pub fn expire(&mut self,now:u64){if self.active.as_ref().is_some_and(|r|now>=r.expires_at_ms){if let Some(r)=self.active.as_mut(){r.challenge.fill(0);r.state=PairingState::Expired} self.active=None}}
}
fn pin_eq(a:&[u8],b:&[u8])->bool{let mut d=a.len()^b.len();let n=a.len().max(b.len());for i in 0..n{d|=(a.get(i).copied().unwrap_or(0)^b.get(i).copied().unwrap_or(0)) as usize}d==0}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayError { WrongSession, TooOld, Replay }
/// Sliding replay window. Call only after message authentication; failed authentication must not advance it.
#[derive(Clone, Debug)]
pub struct ReplayWindow { session: SessionId, highest: Option<u64>, seen: u64 }
impl ReplayWindow {
 pub fn new(session:SessionId)->Self{Self{session,highest:None,seen:0}}
 pub fn accept(&mut self,session:SessionId,seq:u64)->Result<(),ReplayError>{
  if session!=self.session{return Err(ReplayError::WrongSession)}
  match self.highest{None=>{self.highest=Some(seq);self.seen=1;Ok(())},Some(h) if seq>h=>{let shift=seq-h;self.seen=if shift>=64{1}else{(self.seen<<shift)|1};self.highest=Some(seq);Ok(())},Some(h)=>{let delta=h-seq;if delta>=64{return Err(ReplayError::TooOld)}let bit=1u64<<delta;if self.seen&bit!=0{return Err(ReplayError::Replay)}self.seen|=bit;Ok(())}}
 }
}

/// Registry makes concurrent session checks independent and bounds retained replay state.
pub struct SessionRegistry { max:usize, sessions:BTreeMap<SessionId,ReplayWindow>, order:VecDeque<SessionId> }
impl SessionRegistry {
 pub fn new(max:usize)->Self{Self{max,sessions:BTreeMap::new(),order:VecDeque::new()}}
 pub fn open(&mut self,id:SessionId)->Result<(),PairingError>{if self.sessions.contains_key(&id){return Err(PairingError::InvalidProof)}if self.sessions.len()>=self.max{if let Some(old)=self.order.pop_front(){self.sessions.remove(&old);}}self.order.push_back(id);self.sessions.insert(id,ReplayWindow::new(id));Ok(())}
 pub fn accept(&mut self,id:SessionId,seq:u64)->Result<(),ReplayError>{self.sessions.get_mut(&id).ok_or(ReplayError::WrongSession)?.accept(id,seq)}
 pub fn close(&mut self,id:SessionId){self.sessions.remove(&id);self.order.retain(|x|*x!=id)}
}

#[cfg(test)] mod tests {
 use super::*;
 struct E(u8); impl Entropy for E{fn fill(&mut self,o:&mut[u8])->Result<(),PairingError>{o.fill(self.0);Ok(())}}
 struct P; impl PairingProtocol for P{fn verify(&self,_:&PeerId,c:&[u8],p:&[u8],_:&dyn DeviceIdentity)->Result<Vec<u8>,ProtocolError>{if p==c{Ok(vec![7,8])}else{Err(ProtocolError::InvalidProof)}}}
 fn setup()->(PairingManager,E,InMemoryIdentity){(PairingManager::new(10,20),E(4),InMemoryIdentity::for_tests(vec![1],vec![2]))}
 #[test]fn wrong_pin_and_concurrent_request(){let(mut m,mut e,_)=setup();let r=m.request("a".into(),0,&mut e).unwrap();assert_eq!(m.request("b".into(),0,&mut e),Err(PairingError::Busy));assert_eq!(m.confirm(r.id,"0000","1234",1),Err(PairingError::WrongPin));}
 #[test]fn unsupported_is_explicit(){let(mut m,mut e,i)=setup();let r=m.request("a".into(),0,&mut e).unwrap();m.confirm(r.id,"1","1",1).unwrap();assert_eq!(m.verify(r.id,b"x",2,&UnsupportedAirPlayProtocol,&i),Err(PairingError::UnsupportedProtocol));}
 #[test]fn revoke_expire_and_reconnect(){let(mut m,mut e,i)=setup();let r=m.request("a".into(),0,&mut e).unwrap();m.confirm(r.id,"1","1",1).unwrap();let c=m.verify(r.id,&r.challenge,2,&P,&i).unwrap();m.reconnect("a",&c.value,3).unwrap();m.revoke("a");assert_eq!(m.reconnect("a",&c.value,4),Err(PairingError::Revoked));let r=m.request("b".into(),4,&mut e).unwrap();m.confirm(r.id,"1","1",5).unwrap();let c=m.verify(r.id,&r.challenge,6,&P,&i).unwrap();assert_eq!(m.reconnect("b",&c.value,26),Err(PairingError::Expired));}
 #[test]fn request_expires(){let(mut m,mut e,_)=setup();let r=m.request("a".into(),0,&mut e).unwrap();assert_eq!(m.confirm(r.id,"1","1",10),Err(PairingError::NotFound));}
 #[test]fn replay_window_and_cross_session(){let a=[1;16];let b=[2;16];let mut w=ReplayWindow::new(a);w.accept(a,5).unwrap();assert_eq!(w.accept(a,5),Err(ReplayError::Replay));w.accept(a,4).unwrap();assert_eq!(w.accept(b,6),Err(ReplayError::WrongSession));w.accept(a,70).unwrap();assert_eq!(w.accept(a,5),Err(ReplayError::TooOld));}
 #[test]fn concurrent_sessions_are_isolated(){let a=[1;16];let b=[2;16];let mut s=SessionRegistry::new(2);s.open(a).unwrap();s.open(b).unwrap();s.accept(a,1).unwrap();s.accept(b,1).unwrap();assert_eq!(s.accept(a,1),Err(ReplayError::Replay));s.accept(b,2).unwrap();}
}
