//! Minimal C ABI. Handles are integer IDs, never dereferenced foreign pointers.
use crate::{parse_request,DeviceProfile,FeatureSet,Limits,Model,ProtocolVersion,Router,SessionState};
use std::collections::HashMap;
use std::ffi::c_void;
use std::panic::{catch_unwind,AssertUnwindSafe};
use std::slice;
use std::sync::{Mutex,OnceLock};

const MAX_FFI_INPUT:usize=64*1024;
struct Context{router:Router,profile:DeviceProfile}
fn contexts()->&'static Mutex<HashMap<usize,Context>>{static C:OnceLock<Mutex<HashMap<usize,Context>>>=OnceLock::new();C.get_or_init(||Mutex::new(HashMap::new()))}
fn next_id()->usize{use std::sync::atomic::{AtomicUsize,Ordering};static N:AtomicUsize=AtomicUsize::new(1);N.fetch_add(1,Ordering::Relaxed)}
fn profile()->DeviceProfile{DeviceProfile{instance_name:"Android TV".into(),model:Model("AirPlayReceiver1,1".into()),device_id:[0;6],protocol:ProtocolVersion{major:1,minor:0},features:FeatureSet(FeatureSet::AUDIO|FeatureSet::VIDEO)}}
fn guarded(f:impl FnOnce()->i32)->i32{catch_unwind(AssertUnwindSafe(f)).unwrap_or(-127)}

#[no_mangle] pub extern "C" fn airplay_context_create()->*mut c_void{catch_unwind(||{let id=next_id();let p=profile();contexts().lock().ok()?.insert(id,Context{router:Router::new(p.clone()),profile:p});Some(id as *mut c_void)}).ok().flatten().unwrap_or(std::ptr::null_mut())}
#[no_mangle] pub extern "C" fn airplay_context_destroy(handle:*mut c_void){let _=catch_unwind(||{if !handle.is_null(){if let Ok(mut c)=contexts().lock(){c.remove(&(handle as usize));}}});}
#[no_mangle] pub extern "C" fn airplay_context_reset(handle:*mut c_void)->i32{guarded(||{let Ok(mut c)=contexts().lock()else{return -2};let Some(x)=c.get_mut(&(handle as usize))else{return -1};x.router=Router::new(x.profile.clone());0})}
#[no_mangle] pub extern "C" fn airplay_context_state(handle:*mut c_void)->i32{guarded(||{let Ok(c)=contexts().lock()else{return -2};let Some(x)=c.get(&(handle as usize))else{return -1};state_code(x.router.state())})}
#[no_mangle] pub unsafe extern "C" fn airplay_context_handle(handle:*mut c_void,input:*const u8,input_len:usize,output:*mut u8,output_cap:usize)->isize{
 catch_unwind(AssertUnwindSafe(||{if handle.is_null()||input.is_null()||input_len>MAX_FFI_INPUT||output_cap>MAX_FFI_INPUT{return -1}let bytes=unsafe{slice::from_raw_parts(input,input_len)};let req=match parse_request(bytes,Limits::default()){Ok(x)=>x,Err(_)=>return -2};let mut c=match contexts().lock(){Ok(x)=>x,Err(_)=>return -3};let Some(ctx)=c.get_mut(&(handle as usize))else{return -1};let response=match ctx.router.route(&req){Ok(x)=>x,Err(_)=>return -4};let encoded=crate::frontend::encode_response(&response);if encoded.len()>output_cap{return encoded.len() as isize}if !encoded.is_empty(){if output.is_null(){return -1}unsafe{std::ptr::copy_nonoverlapping(encoded.as_ptr(),output,encoded.len())}}encoded.len() as isize})).unwrap_or(-127)
}
fn state_code(s:SessionState)->i32{match s{SessionState::Idle=>0,SessionState::Announced=>1,SessionState::Configured=>2,SessionState::Streaming=>3,SessionState::Paused=>4,SessionState::Closed=>5}}

#[cfg(test)]mod tests{use super::*;
 #[test]fn lifecycle_and_idempotent_destroy(){let h=airplay_context_create();assert!(!h.is_null());assert_eq!(airplay_context_state(h),0);airplay_context_destroy(h);airplay_context_destroy(h);assert_eq!(airplay_context_state(h),-1)}
 #[test]fn bounded_and_isolated(){let a=airplay_context_create();let b=airplay_context_create();let q=b"ANNOUNCE * RTSP/1.0\r\nCSeq: 1\r\n\r\n";let mut out=[0u8;256];assert!(unsafe{airplay_context_handle(a,q.as_ptr(),q.len(),out.as_mut_ptr(),out.len())}>0);assert_eq!(airplay_context_state(a),1);assert_eq!(airplay_context_state(b),0);assert_eq!(unsafe{airplay_context_handle(a,q.as_ptr(),MAX_FFI_INPUT+1,out.as_mut_ptr(),out.len())},-1);airplay_context_destroy(a);airplay_context_destroy(b)}
}
