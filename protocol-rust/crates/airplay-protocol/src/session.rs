use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionState { Idle, Announced, Configured, Streaming, Paused, Closed }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event { Announce, Setup, Record, Pause, Resume, Flush, Teardown }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionMachine { state: SessionState }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionError { pub state: SessionState, pub event: Event }
impl fmt::Display for SessionError { fn fmt(&self, f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f,"event {:?} is illegal in state {:?}",self.event,self.state) } }
impl std::error::Error for SessionError {}

impl Default for SessionMachine { fn default()->Self { Self::new() } }
impl SessionMachine {
 pub const fn new()->Self { Self { state: SessionState::Idle } }
 pub const fn state(&self)->SessionState { self.state }
 pub fn apply(&mut self, event:Event)->Result<SessionState,SessionError> {
   use Event::*; use SessionState::*;
   let next=match (self.state,event) {
    (Idle,Announce)=>Announced, (Announced,Setup)=>Configured, (Configured,Record)=>Streaming,
    (Streaming,Pause)=>Paused, (Paused,Resume)=>Streaming,
    (Streaming,Flush)=>Configured, (Paused,Flush)=>Configured,
    (Idle|Announced|Configured|Streaming|Paused,Teardown)=>Closed,
    _=>return Err(SessionError{state:self.state,event}),
   };
   self.state=next; Ok(next)
 }
}

#[cfg(test)]
mod tests { use super::*;
 #[test] fn normal_lifecycle() { let mut s=SessionMachine::new(); for e in [Event::Announce,Event::Setup,Event::Record,Event::Pause,Event::Resume,Event::Teardown] { s.apply(e).unwrap(); } assert_eq!(s.state(),SessionState::Closed); }
 #[test] fn rejects_and_preserves_state() { let mut s=SessionMachine::new(); assert_eq!(s.apply(Event::Record),Err(SessionError{state:SessionState::Idle,event:Event::Record})); assert_eq!(s.state(),SessionState::Idle); }
 #[test] fn closed_is_terminal() { let mut s=SessionMachine::new(); s.apply(Event::Teardown).unwrap(); assert!(s.apply(Event::Announce).is_err()); }
}
