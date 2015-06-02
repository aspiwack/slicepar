use std::collections::VecDeque;

trait Queue<'a,A:?Sized> {
    type Error;
    fn push (&mut self,x:&'a mut A);
    fn pop (&mut self) -> Result<&'a mut A, Self::Error>;
}

fn iter_queue<'a,A:?Sized+'a,Q,F> (mut q:Q, f:F)
    where F:Fn(&mut FnMut(&'a mut A)->(),&'a mut A)->(),
          Q:Queue<'a,A> + 'a
{
    while let Ok(x) = q.pop() {
        let mut yld = |x| q.push(x);
        f(&mut yld,x);
    }
}

struct VecQueueError;

impl<'a,A:?Sized> Queue<'a,A> for VecDeque<&'a mut A> {
    type Error = VecQueueError;
    fn push(&mut self,x:&'a mut A) { self.push_front(x); }
    fn pop (&mut self) -> Result<&'a mut A, Self::Error> {
        self.pop_back().ok_or(VecQueueError)
    }
}

pub fn pool<'a,A:?Sized+'a,D> (o:&'a mut A,d:D) -> ()
    where D:Fn(&mut FnMut(&'a mut A)->(), &'a mut A) -> ()
{
    let mut queue = VecDeque::new ();
    queue.push(o);
    iter_queue (queue,d);
}

#[cfg(test)]
mod tests {

    use super::pool;
    use std::cmp::Ordering;

    trait SplitAround {
        fn split_around (&mut self,p:usize) -> (&mut Self,&mut Self);
    }

    impl<T> SplitAround for [T] {
        fn split_around (&mut self,p:usize) -> ( &mut [T] , &mut [T] ) {
            let (a,b) = self.split_at_mut(p);
            let (_,b) = b.split_at_mut(1);
            ( a , b )
        }
    }

    #[test]
    fn quick_sort() {

        fn quick_sort<A> (c:&Fn(&A,&A)->Ordering, a:&mut [A]) {
            pool(a, |yld,a| {
                if a.len() <= 1 { return; }
                let mut left=0;
                let mut right=a.len()-1;
                while left<right {
                    let next = left+1;
                    match c(&a[next],&a[left]) {
                        Ordering::Less => {
                            a.swap(left,next);
                            left += 1;
                        },
                        Ordering::Greater => {
                            a.swap(next,right);
                            right -= 1;
                        },
                        Ordering::Equal => {
                            left += 1;
                        }
                    }
                };
                let (sub_left,sub_right) = a.split_around(left);
                yld(sub_left);
                yld(sub_right);
            });
        };

        let shuffled = &mut[5,3,2,7,1,6,4];
        let sorted   = &mut[1,2,3,4,5,6,7];
        quick_sort ( &|x:&i64,y|x.cmp(y) , shuffled );
        assert_eq!(shuffled,sorted)
    }
}
