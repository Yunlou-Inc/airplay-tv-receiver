//! Bounded blocking TCP RTSP frontend. Each connection owns its Router.
use crate::{parse_request, DeviceProfile, Limits, Response, Router};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct ServerLimits {
    pub protocol: Limits,
    pub max_connections: usize,
    pub max_requests_per_connection: usize,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
}
impl Default for ServerLimits { fn default()->Self { Self { protocol:Limits::default(),max_connections:16,max_requests_per_connection:128,read_timeout:Duration::from_secs(15),write_timeout:Duration::from_secs(10) } } }

pub struct Server { listener:TcpListener, profile:DeviceProfile, limits:ServerLimits }
impl Server {
 pub fn bind(addr:&str,profile:DeviceProfile,limits:ServerLimits)->io::Result<Self>{let listener=TcpListener::bind(addr)?;Ok(Self{listener,profile,limits})}
 pub fn local_addr(&self)->io::Result<std::net::SocketAddr>{self.listener.local_addr()}
 pub fn run(self,stop:Arc<AtomicBool>)->io::Result<()> {
  self.listener.set_nonblocking(true)?;
  let active=Arc::new(Mutex::new(0usize));
  while !stop.load(Ordering::Relaxed) {
   match self.listener.accept() {
    Ok((stream,_))=>{let mut n=active.lock().map_err(|_|io::Error::other("connection counter poisoned"))?;if *n>=self.limits.max_connections { drop(n); reject_busy(stream,self.limits.write_timeout); continue } *n+=1;drop(n);
     let profile=self.profile.clone();let limits=self.limits;let count=active.clone();thread::spawn(move||{let _guard=ConnectionGuard(count);let _=serve_stream(stream,profile,limits);});}
    Err(e) if e.kind()==io::ErrorKind::WouldBlock=>thread::sleep(Duration::from_millis(10)),
    Err(e)=>return Err(e),
   }
  } Ok(())
 }
}
struct ConnectionGuard(Arc<Mutex<usize>>);
impl Drop for ConnectionGuard {fn drop(&mut self){if let Ok(mut n)=self.0.lock(){*n=n.saturating_sub(1)}}}
fn reject_busy(mut s:TcpStream,t:Duration){let _=s.set_write_timeout(Some(t));let _=s.write_all(b"RTSP/1.0 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");}

pub fn serve_stream(mut stream:TcpStream,profile:DeviceProfile,limits:ServerLimits)->io::Result<()> {
 stream.set_read_timeout(Some(limits.read_timeout))?;stream.set_write_timeout(Some(limits.write_timeout))?;
 let mut router=Router::new(profile);let mut pending=Vec::new();
 for _ in 0..limits.max_requests_per_connection {
  let frame=match read_frame(&mut stream,&mut pending,limits.protocol){Ok(Some(x))=>x,Ok(None)=>return Ok(()),Err(FrameError::Io(e))=>return Err(e),Err(_)=>{write_status(&mut stream,400,"Bad Request")?;return Ok(())}};
  let response=match parse_request(&frame,limits.protocol){Ok(req)=>match router.route(&req){Ok(r)=>r,Err(_)=>{write_status(&mut stream,400,"Bad Request")?;continue}},Err(_)=>{write_status(&mut stream,400,"Bad Request")?;continue}};
  write_response(&mut stream,&response)?;
 }
 write_status(&mut stream,429,"Too Many Requests")
}
#[derive(Debug)] enum FrameError{Io(io::Error),TooLarge,InvalidLength}
fn read_frame<R:Read>(r:&mut R,pending:&mut Vec<u8>,limits:Limits)->Result<Option<Vec<u8>>,FrameError>{
 loop {
  if let Some(end)=pending.windows(4).position(|w|w==b"\r\n\r\n") {
   let head_end=end+4;if head_end>limits.max_request_bytes{return Err(FrameError::TooLarge)}
   let head=std::str::from_utf8(&pending[..end]).map_err(|_|FrameError::InvalidLength)?;let mut length=0usize;let mut seen=false;
   for line in head.split("\r\n").skip(1){if let Some((n,v))=line.split_once(':'){if n.eq_ignore_ascii_case("content-length"){if seen{return Err(FrameError::InvalidLength)}seen=true;length=v.trim().parse().map_err(|_|FrameError::InvalidLength)?;}}}
   if length>limits.max_body_bytes{return Err(FrameError::TooLarge)}let total=head_end.checked_add(length).ok_or(FrameError::TooLarge)?;if total>limits.max_request_bytes{return Err(FrameError::TooLarge)}
   if pending.len()>=total{return Ok(Some(pending.drain(..total).collect()))}
  }
  if pending.len()>=limits.max_request_bytes{return Err(FrameError::TooLarge)}let room=(limits.max_request_bytes-pending.len()).min(4096);let mut buf=[0u8;4096];match r.read(&mut buf[..room]){Ok(0) if pending.is_empty()=>return Ok(None),Ok(0)=>return Err(FrameError::InvalidLength),Ok(n)=>pending.extend_from_slice(&buf[..n]),Err(e)=>return Err(FrameError::Io(e))}
 }
}
pub fn encode_response(r:&Response)->Vec<u8>{let mut out=format!("RTSP/1.0 {} {}\r\n",r.status,r.reason).into_bytes();for(k,v)in &r.headers{out.extend_from_slice(format!("{k}: {v}\r\n").as_bytes())}if !r.headers.keys().any(|k|k.eq_ignore_ascii_case("content-length")){out.extend_from_slice(format!("Content-Length: {}\r\n",r.body.len()).as_bytes())}out.extend_from_slice(b"\r\n");out.extend_from_slice(&r.body);out}
fn write_response<W:Write>(w:&mut W,r:&Response)->io::Result<()>{w.write_all(&encode_response(r))}
fn write_status<W:Write>(w:&mut W,status:u16,reason:&str)->io::Result<()>{w.write_all(format!("RTSP/1.0 {status} {reason}\r\nContent-Length: 0\r\n\r\n").as_bytes())}

#[cfg(test)] mod tests {use super::*;use crate::{FeatureSet,Model,ProtocolVersion};use std::io::Cursor;
 fn profile()->DeviceProfile{DeviceProfile{instance_name:"TV".into(),model:Model("M".into()),device_id:[0;6],protocol:ProtocolVersion{major:1,minor:0},features:FeatureSet(1)}}
 #[test]fn pipelined_frames_are_separated(){let mut c=Cursor::new(b"OPTIONS * RTSP/1.0\r\nCSeq: 1\r\n\r\nOPTIONS * RTSP/1.0\r\nCSeq: 2\r\n\r\n");let mut p=vec![];assert!(read_frame(&mut c,&mut p,Limits::default()).unwrap().is_some());assert!(read_frame(&mut c,&mut p,Limits::default()).unwrap().is_some())}
 #[test]fn oversized_header_is_rejected(){let mut l=Limits::default();l.max_request_bytes=16;let mut c=Cursor::new(vec![b'x';32]);assert!(matches!(read_frame(&mut c,&mut vec![],l),Err(FrameError::TooLarge)))}
 #[test]fn distinct_routers_do_not_share_state(){let mut a=Router::new(profile());let mut b=Router::new(profile());let q=parse_request(b"OPTIONS * RTSP/1.0\r\nCSeq: 1\r\n\r\n",Limits::default()).unwrap();a.route(&q).unwrap();assert!(b.route(&q).is_ok())}
}
