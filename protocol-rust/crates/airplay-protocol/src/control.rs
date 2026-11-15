//! Pure, bounded RTSP control-plane routing. No sockets, pairing crypto, or media decryption.
use crate::{DeviceProfile, Event, Method, Request, SessionMachine, SessionState};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouteLimits { pub max_cseq: u32, pub max_session_id_bytes: usize }
impl Default for RouteLimits { fn default() -> Self { Self { max_cseq: i32::MAX as u32, max_session_id_bytes: 64 } } }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response { pub status: u16, pub reason: &'static str, pub headers: BTreeMap<String,String>, pub body: Vec<u8> }
impl Response {
 fn new(status:u16, reason:&'static str, cseq:Option<u32>)->Self { let mut headers=BTreeMap::new(); if let Some(v)=cseq { headers.insert("CSeq".into(),v.to_string()); } Self{status,reason,headers,body:vec![]} }
 fn body(mut self, content_type:&str, body:Vec<u8>)->Self { self.headers.insert("Content-Type".into(),content_type.into()); self.headers.insert("Content-Length".into(),body.len().to_string()); self.body=body; self }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteError { MissingCSeq, DuplicateCSeq, InvalidCSeq, CSeqOutOfOrder { previous:u32, received:u32 }, MissingSession, InvalidSession, SessionMismatch, InvalidState }

#[derive(Clone, Debug)]
pub struct Router { profile:DeviceProfile, machine:SessionMachine, session_id:Option<String>, last_cseq:Option<u32>, limits:RouteLimits }
impl Router {
 pub fn new(profile:DeviceProfile)->Self { Self::with_limits(profile,RouteLimits::default()) }
 pub fn with_limits(profile:DeviceProfile,limits:RouteLimits)->Self { Self{profile,machine:SessionMachine::new(),session_id:None,last_cseq:None,limits} }
 pub fn state(&self)->SessionState { self.machine.state() }
 pub fn route(&mut self, req:&Request)->Result<Response,RouteError> {
  let cseq=self.cseq(req)?;
  if let Some(previous)=self.last_cseq { if cseq<=previous { return Err(RouteError::CSeqOutOfOrder{previous,received:cseq}); } }
  let mut response=match (&req.method,req.uri.as_str()) {
   (Method::Get,"/info") => Response::new(200,"OK",Some(cseq)).body("application/json",self.info_json().into_bytes()),
   (Method::Options,_) => { let mut r=Response::new(200,"OK",Some(cseq)); r.headers.insert("Public".into(),"OPTIONS, GET, ANNOUNCE, SETUP, RECORD, PAUSE, FLUSH, TEARDOWN".into()); r },
   (Method::Announce,_) => { self.no_session(req)?; self.transition(Event::Announce)?; Response::new(200,"OK",Some(cseq)) },
   (Method::Setup,_) => { self.no_session(req)?; self.transition(Event::Setup)?; let id=format!("{:08X}",cseq); self.session_id=Some(id.clone()); let mut r=Response::new(200,"OK",Some(cseq)); r.headers.insert("Session".into(),id); r },
   (Method::Record,_) => { self.require_session(req)?; self.transition(Event::Record)?; Response::new(200,"OK",Some(cseq)) },
   (Method::Pause,_) => { self.require_session(req)?; self.transition(Event::Pause)?; Response::new(200,"OK",Some(cseq)) },
   (Method::Flush,_) => { self.require_session(req)?; self.transition(Event::Flush)?; Response::new(200,"OK",Some(cseq)) },
   (Method::Teardown,_) => { self.require_session(req)?; self.transition(Event::Teardown)?; self.session_id=None; Response::new(200,"OK",Some(cseq)) },
   (Method::Other(_),_) | (Method::Get,_) => Response::new(404,"Not Found",Some(cseq)),
   _ => Response::new(501,"Not Implemented",Some(cseq)),
  };
  if let Some(id)=&self.session_id { if !matches!(req.method,Method::Setup) { response.headers.insert("Session".into(),id.clone()); } }
  self.last_cseq=Some(cseq); Ok(response)
 }
 fn cseq(&self,req:&Request)->Result<u32,RouteError> { let vals:Vec<_>=req.headers.values("CSeq").collect(); if vals.is_empty(){return Err(RouteError::MissingCSeq)} if vals.len()!=1{return Err(RouteError::DuplicateCSeq)} let n=vals[0].parse().map_err(|_|RouteError::InvalidCSeq)?; if n>self.limits.max_cseq{return Err(RouteError::InvalidCSeq)} Ok(n) }
 fn session_value<'a>(&self,req:&'a Request)->Result<Option<&'a str>,RouteError> { let vals:Vec<_>=req.headers.values("Session").collect(); if vals.len()>1{return Err(RouteError::InvalidSession)} let Some(raw)=vals.first() else{return Ok(None)}; let value=raw.split(';').next().unwrap_or("").trim(); if value.is_empty()||value.len()>self.limits.max_session_id_bytes||!value.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_'){return Err(RouteError::InvalidSession)} Ok(Some(value)) }
 fn no_session(&self,req:&Request)->Result<(),RouteError>{ if self.session_value(req)?.is_some(){Err(RouteError::InvalidSession)}else{Ok(())} }
 fn require_session(&self,req:&Request)->Result<(),RouteError>{ let got=self.session_value(req)?.ok_or(RouteError::MissingSession)?; if self.session_id.as_deref()!=Some(got){Err(RouteError::SessionMismatch)}else{Ok(())} }
 fn transition(&mut self,e:Event)->Result<(),RouteError>{self.machine.apply(e).map(|_|()).map_err(|_|RouteError::InvalidState)}
 fn info_json(&self)->String { format!("{{\"deviceID\":\"{}\",\"features\":{},\"model\":\"{}\",\"protocolVersion\":\"{}.{}\"}}",self.profile.device_id_text(),self.profile.features.0,json_escape(&self.profile.model.0),self.profile.protocol.major,self.profile.protocol.minor) }
}
fn json_escape(s:&str)->String { let mut out=String::new(); for c in s.chars(){match c{'"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),c if c.is_control()=>out.push_str(&format!("\\u{:04X}",c as u32)),c=>out.push(c)}} out }

#[cfg(test)] mod tests {
 use super::*; use crate::{parse_request,FeatureSet,Limits,Model,ProtocolVersion};
 fn profile()->DeviceProfile{DeviceProfile{instance_name:"TV".into(),model:Model("Safe\"Model".into()),device_id:[0,1,2,3,4,5],protocol:ProtocolVersion{major:1,minor:0},features:FeatureSet(FeatureSet::AUDIO)}}
 fn req(s:&str)->Request{parse_request(s.as_bytes(),Limits::default()).unwrap()}
 #[test] fn info_is_safe_and_only_declared(){let mut r=Router::new(profile());let x=r.route(&req("GET /info RTSP/1.0\r\nCSeq: 1\r\n\r\n")).unwrap();assert_eq!(x.status,200);let b=String::from_utf8(x.body).unwrap();assert!(b.contains("Safe\\\"Model"));assert!(b.contains("\"features\":1"));assert!(!b.contains("pairing"));}
 #[test] fn normal_lifecycle_and_session(){let mut r=Router::new(profile());r.route(&req("ANNOUNCE * RTSP/1.0\r\nCSeq: 1\r\n\r\n")).unwrap();let s=r.route(&req("SETUP * RTSP/1.0\r\nCSeq: 2\r\n\r\n")).unwrap().headers["Session"].clone();r.route(&req(&format!("RECORD * RTSP/1.0\r\nCSeq: 3\r\nSession: {s}\r\n\r\n"))).unwrap();assert_eq!(r.state(),SessionState::Streaming);}
 #[test] fn rejects_out_of_order_and_duplicate(){let mut r=Router::new(profile());r.route(&req("OPTIONS * RTSP/1.0\r\nCSeq: 2\r\n\r\n")).unwrap();assert!(matches!(r.route(&req("OPTIONS * RTSP/1.0\r\nCSeq: 1\r\n\r\n")),Err(RouteError::CSeqOutOfOrder{..})));assert!(matches!(r.route(&req("OPTIONS * RTSP/1.0\r\nCSeq: 2\r\n\r\n")),Err(RouteError::CSeqOutOfOrder{..})));}
 #[test] fn errors_do_not_consume_cseq(){let mut r=Router::new(profile());assert_eq!(r.route(&req("RECORD * RTSP/1.0\r\nCSeq: 1\r\n\r\n")),Err(RouteError::MissingSession));assert!(r.route(&req("OPTIONS * RTSP/1.0\r\nCSeq: 1\r\n\r\n")).is_ok());}
 #[test] fn rejects_duplicate_cseq_and_oversize_session(){let mut r=Router::with_limits(profile(),RouteLimits{max_cseq:9,max_session_id_bytes:3});assert_eq!(r.route(&req("OPTIONS * RTSP/1.0\r\nCSeq: 1\r\nCSeq: 2\r\n\r\n")),Err(RouteError::DuplicateCSeq));assert_eq!(r.route(&req("OPTIONS * RTSP/1.0\r\nCSeq: 10\r\n\r\n")),Err(RouteError::InvalidCSeq));}
 #[test] fn wrong_state_is_rejected(){let mut r=Router::new(profile());assert_eq!(r.route(&req("SETUP * RTSP/1.0\r\nCSeq: 1\r\n\r\n")),Err(RouteError::InvalidState));}
}
