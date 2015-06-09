use std::collections::VecDeque;
mod queue;
use queue::Queue;
mod pool;
use pool::ThreadPool;

// Sequential

fn iter_queue<A,Q,F> (mut q:Q, f:F)
    where F:Fn(&mut FnMut(A)->(),A)->(),
          Q:Queue<A>
{
    while let Some(x) = q.pop() {
        let mut yld = |x| q.push(x);
        f(&mut yld,x);
    }
}

pub fn seq<A,D> (o:A,d:D) -> ()
    where D:Fn(&mut FnMut(A)->(), A) -> ()
{
    let mut queue = VecDeque::new ();
    queue.push(o);
    iter_queue (queue,d);
}


// Concurrent

fn rec_pool<A> (pool:&'static ThreadPool,s:A,d:&'static (Fn(&Fn(A)->(), A) -> ()+Sync)) -> ()
    where A:Send+'static
{
    pool.execute(move || {
        let yld = move |x| rec_pool(pool,x,d);
        d (&yld , s)
    })
}


// Constant because of lazyness
const NTHREADS :usize = 4;

pub fn par<A> (o:A,d:&'static (Fn(&Fn(A)->(), A) -> () + Sync)) -> ()
    where A:Send+'static,
{
    let pool = ThreadPool::new(NTHREADS);
    // Does not type check because of the `'static` constraint.
    // rec_pool(&pool,o,d);
    unimplemented!();
}

#[cfg(test)]
mod tests {

    use super::*;
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
    fn quick_sort_seq() {

        fn quick_sort<A> (c:&Fn(&A,&A)->Ordering, a:&mut [A]) {
            seq(a, |yld,a| {
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
