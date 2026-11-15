//! Minimal C ABI. Handles are integer IDs, never dereferenced foreign pointers.
use crate::{parse_request,DeviceProfile,FeatureSet,Limits,Model,ProtocolVersion,Router,SessionState};
use std::collections::HashMap;
use std::ffi::c_void;
use std::panic::{catch_unwind,AssertUnwindSafe};
use std::slice;
use std::sync::{Arc,Mutex,OnceLock};
use std::sync::atomic::{AtomicBool,Ordering};
use std::thread;

const MAX_FFI_INPUT:usize=64*1024;
const MAX_BIND_ADDR:usize=128;
struct Listener{stop:Arc<AtomicBool>,bound_addr:String}
struct Context{router:Router,profile:DeviceProfile,listener:Option<Listener>}
// Defense-in-depth bind validation: only explicit local interfaces are allowed.
// Accepts an IP literal with a port (e.g. 127.0.0.1:7000, [::1]:7000). Rejects
// unspecified/wildcard addresses (0.0.0.0, [::]) and non-numeric hosts, so callers
// cannot accidentally expose the receiver on all interfaces. Port 0 (ephemeral) is
// allowed for loopback verification; the real bound address is reported back.
fn validate_bind_addr(addr:&str)->Option<std::net::SocketAddr>{
 if addr.is_empty()||addr.len()>MAX_BIND_ADDR{return None}
 let s:std::net::SocketAddr=addr.parse().ok()?;
 if s.ip().is_unspecified(){return None}
 Some(s)
}
fn contexts()->&'static Mutex<HashMap<usize,Context>>{static C:OnceLock<Mutex<HashMap<usize,Context>>>=OnceLock::new();C.get_or_init(||Mutex::new(HashMap::new()))}
fn next_id()->usize{use std::sync::atomic::{AtomicUsize,Ordering};static N:AtomicUsize=AtomicUsize::new(1);N.fetch_add(1,Ordering::Relaxed)}
// Conservative default profile. This FFI is not yet wired into Android and must not
// advertise capabilities that are not actually implemented. Keep `features` aligned with the
// Android `DiscoveryConfig` default (0x0) until a capability is real; only advertise what the
// receiver can actually serve. `device_id` is a placeholder and must be replaced with a real,
// per-device, non-zero identifier (never a hard-coded constant) before any advertising.
fn profile()->DeviceProfile{DeviceProfile{instance_name:"Android TV".into(),model:Model("AirPlayReceiver1,1".into()),device_id:[0;6],protocol:ProtocolVersion{major:1,minor:0},features:FeatureSet(0)}}
fn guarded(f:impl FnOnce()->i32)->i32{catch_unwind(AssertUnwindSafe(f)).unwrap_or(-127)}

#[no_mangle] pub extern "C" fn airplay_context_create()->*mut c_void{catch_unwind(||{let id=next_id();let p=profile();contexts().lock().ok()?.insert(id,Context{router:Router::new(p.clone()),profile:p,listener:None});Some(id as *mut c_void)}).ok().flatten().unwrap_or(std::ptr::null_mut())}
#[no_mangle] pub extern "C" fn airplay_context_destroy(handle:*mut c_void){let _=catch_unwind(||{if !handle.is_null(){if let Ok(mut c)=contexts().lock(){if let Some(x)=c.remove(&(handle as usize)){if let Some(l)=x.listener{l.stop.store(true,Ordering::Relaxed);}}}}});}
#[no_mangle] pub extern "C" fn airplay_context_reset(handle:*mut c_void)->i32{guarded(||{let Ok(mut c)=contexts().lock()else{return -2};let Some(x)=c.get_mut(&(handle as usize))else{return -1};x.router=Router::new(x.profile.clone());0})}
#[no_mangle] pub extern "C" fn airplay_context_state(handle:*mut c_void)->i32{guarded(||{let Ok(c)=contexts().lock()else{return -2};let Some(x)=c.get(&(handle as usize))else{return -1};state_code(x.router.state())})}
#[no_mangle] pub unsafe extern "C" fn airplay_context_handle(handle:*mut c_void,input:*const u8,input_len:usize,output:*mut u8,output_cap:usize)->isize{
 catch_unwind(AssertUnwindSafe(||{if handle.is_null()||input.is_null()||input_len>MAX_FFI_INPUT||output_cap>MAX_FFI_INPUT{return -1}let bytes=unsafe{slice::from_raw_parts(input,input_len)};let req=match parse_request(bytes,Limits::default()){Ok(x)=>x,Err(_)=>return -2};let mut c=match contexts().lock(){Ok(x)=>x,Err(_)=>return -3};let Some(ctx)=c.get_mut(&(handle as usize))else{return -1};let response=match ctx.router.route(&req){Ok(x)=>x,Err(_)=>return -4};let encoded=crate::frontend::encode_response(&response);if encoded.len()>output_cap{return encoded.len() as isize}if !encoded.is_empty(){if output.is_null(){return -1}unsafe{std::ptr::copy_nonoverlapping(encoded.as_ptr(),output,encoded.len())}}encoded.len() as isize})).unwrap_or(-127)
}
fn state_code(s:SessionState)->i32{match s{SessionState::Idle=>0,SessionState::Announced=>1,SessionState::Configured=>2,SessionState::Streaming=>3,SessionState::Paused=>4,SessionState::Closed=>5}}

// ---- Listen control -------------------------------------------------------
// Return codes for the listen APIs (extend the existing code set):
//   -1 invalid handle / null pointer / param too large
//   -2 context lock poisoned
//   -3 invalid or wildcard bind address
//   -8 caller has NOT signalled explicit user authorization
//   -9 a listener is already running for this context
//  -10 bind/IO failure
// -127 panic caught at the FFI boundary
//   >0 success: the bound TCP port (state) or 0 (stop).
//
// `authorized` MUST be 1. This mirrors the M1-2 threat-model gate: the receiver
// opens no socket until the caller (Android UI) has obtained explicit user consent.
// The address must be an explicit local interface literal (loopback or a specific
// NIC address); wildcard binds are rejected so the receiver is never exposed on
// all interfaces by accident.
#[no_mangle] pub extern "C" fn airplay_listen_start(handle:*mut c_void,addr:*const u8,addr_len:usize,authorized:i32)->i32{
 guarded(||{
  if handle.is_null()||addr.is_null()||addr_len==0||addr_len>MAX_BIND_ADDR{return -1}
  if authorized!=1{return -8}
  let raw=unsafe{slice::from_raw_parts(addr,addr_len)};
  let s=match std::str::from_utf8(raw){Ok(x)=>x,Err(_)=>return -3};
  if validate_bind_addr(s).is_none(){return -3}
  let stop=Arc::new(AtomicBool::new(false));
  let mut c=match contexts().lock(){Ok(x)=>x,Err(_)=>return -2};
  let x=match c.get_mut(&(handle as usize)){Some(v)=>v,None=>return -1};
  if x.listener.is_some(){return -9}
  let server=match crate::frontend::Server::bind(s,x.profile.clone(),crate::frontend::ServerLimits::default()){Ok(v)=>v,Err(_)=>return -10};
  let local=match server.local_addr(){Ok(v)=>v,Err(_)=>return -10};
  let port=local.port();
  let bound_addr=local.to_string();
  let stop_thread=stop.clone();
  let _=thread::spawn(move||{let _=server.run(stop_thread);});
  x.listener=Some(Listener{stop,bound_addr});
  port as i32
 })
}
#[no_mangle] pub extern "C" fn airplay_listen_stop(handle:*mut c_void)->i32{
 guarded(||{
  if handle.is_null(){return -1}
  let mut c=match contexts().lock(){Ok(x)=>x,Err(_)=>return -2};
  let x=match c.get_mut(&(handle as usize)){Some(v)=>v,None=>return -1};
  match x.listener.take(){Some(l)=>{l.stop.store(true,Ordering::Relaxed);0},None=>-1}
 })
}
// 0 = not listening, -1 = invalid handle, >0 = bound port.
#[no_mangle] pub extern "C" fn airplay_listen_state(handle:*mut c_void)->i32{
 guarded(||{
  if handle.is_null(){return -1}
  let c=match contexts().lock(){Ok(x)=>x,Err(_)=>return -2};
  let x=match c.get(&(handle as usize)){Some(v)=>v,None=>return -1};
  match &x.listener{Some(l)=>l.bound_addr.rsplit(':').next().and_then(|p|p.parse().ok()).unwrap_or(1),None=>0}
 })
}
// Writes the actual bound address string; returns byte count (or needed capacity).
#[no_mangle] pub unsafe extern "C" fn airplay_listen_bound_addr(handle:*mut c_void,out:*mut u8,out_cap:usize)->isize{
 catch_unwind(AssertUnwindSafe(||{
  if handle.is_null()||out_cap>MAX_FFI_INPUT{return -1}
  let c=match contexts().lock(){Ok(x)=>x,Err(_)=>return -2};
  let x=match c.get(&(handle as usize)){Some(v)=>v,None=>return -1};
  let text=match &x.listener{Some(l)=>l.bound_addr.as_bytes(),None=>return -1};
  if text.len()>out_cap{return text.len() as isize}
  if out.is_null(){return -1}
  unsafe{std::ptr::copy_nonoverlapping(text.as_ptr(),out,text.len())}
  text.len() as isize
 })).unwrap_or(-127)
}

#[cfg(test)]mod tests{use super::*;
 #[test]fn lifecycle_and_idempotent_destroy(){let h=airplay_context_create();assert!(!h.is_null());assert_eq!(airplay_context_state(h),0);airplay_context_destroy(h);airplay_context_destroy(h);assert_eq!(airplay_context_state(h),-1)}
 #[test]fn bounded_and_isolated(){let a=airplay_context_create();let b=airplay_context_create();let q=b"ANNOUNCE * RTSP/1.0\r\nCSeq: 1\r\n\r\n";let mut out=[0u8;256];assert!(unsafe{airplay_context_handle(a,q.as_ptr(),q.len(),out.as_mut_ptr(),out.len())}>0);assert_eq!(airplay_context_state(a),1);assert_eq!(airplay_context_state(b),0);assert_eq!(unsafe{airplay_context_handle(a,q.as_ptr(),MAX_FFI_INPUT+1,out.as_mut_ptr(),out.len())},-1);airplay_context_destroy(a);airplay_context_destroy(b)}
 #[test]fn listen_requires_explicit_authorization(){let h=airplay_context_create();let addr=b"127.0.0.1:0";assert_eq!(airplay_listen_start(h,addr.as_ptr(),addr.len(),0),-8);assert_eq!(airplay_listen_state(h),0);airplay_context_destroy(h)}
 #[test]fn listen_rejects_wildcard_and_non_literal(){let h=airplay_context_create();for a in [&b"0.0.0.0:0"[..],&b"[::]:0"[..],&b"localhost:0"[..]]{assert_eq!(airplay_listen_start(h,a.as_ptr(),a.len(),1),-3)}let empty=b"";assert_eq!(airplay_listen_start(h,empty.as_ptr(),empty.len(),1),-1);assert_eq!(airplay_listen_state(h),0);airplay_context_destroy(h)}
 #[test]fn listen_loopback_start_stop_and_addr(){let h=airplay_context_create();let addr=b"127.0.0.1:0";let port=airplay_listen_start(h,addr.as_ptr(),addr.len(),1);if port==-10{airplay_context_destroy(h);return}assert!(port>0,"expected bound port, got {}",port);assert!(airplay_listen_state(h)>0);let mut buf=[0u8;128];let n=unsafe{airplay_listen_bound_addr(h,buf.as_mut_ptr(),buf.len())};assert!(n>0);assert!(std::str::from_utf8(&buf[..n as usize]).unwrap().starts_with("127.0.0.1:"));assert_eq!(airplay_listen_stop(h),0);assert_eq!(airplay_listen_state(h),0);assert_eq!(airplay_listen_stop(h),-1);airplay_context_destroy(h)}
 #[test]fn second_listen_is_rejected(){let h=airplay_context_create();let addr=b"127.0.0.1:0";let p=airplay_listen_start(h,addr.as_ptr(),addr.len(),1);if p==-10{airplay_context_destroy(h);return}assert!(p>0);assert_eq!(airplay_listen_start(h,addr.as_ptr(),addr.len(),1),-9);airplay_listen_stop(h);airplay_context_destroy(h)}
}
