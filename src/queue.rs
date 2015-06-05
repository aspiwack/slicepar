//! Types implementing the Queue trait can be used as queues. The
//! order in which elements are inserted and taken out is not
//! specified.

use std::collections::VecDeque;

pub trait Queue<A> {
    fn push (&mut self,x:A);
    fn pop (&mut self) -> Option<A>;
    fn is_empty(&self) -> bool;
}


impl<A> Queue<A> for VecDeque<A> {
    fn push(&mut self,x:A) { self.push_front(x); }
    fn pop (&mut self) -> Option<A> { self.pop_back() }
    fn is_empty(&self) -> bool { self.is_empty() }
}
