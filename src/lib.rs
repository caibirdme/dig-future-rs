use std::cell::RefCell;

thread_local!(static NOTIFY: RefCell<bool> = RefCell::new(true));

pub mod task {
    use crate::NOTIFY;

    pub struct Context<'a> {
        waker: &'a Waker,
    }

    impl<'a> Context<'a> {
        pub fn from_waker(waker: &'a Waker) -> Self {
            Self {
                waker,
            }
        }

        pub fn waker(&self) -> &'a Waker {
            self.waker
        }
    }

    pub struct Waker;

    impl Waker {
        pub fn wake(&self) {
            NOTIFY.with(|v| *v.borrow_mut() = true);
        }
    }
}

pub mod future {
    use crate::task::*;
    use crate::NOTIFY;

    pub enum Poll<T> {
        Pending,
        Ready(T),
    }

    pub trait Future {
        type Output;
        fn poll(&mut self, ctx: &Context) -> Poll<Self::Output>;
        fn map<U,F>(self, f: F) -> Map<Self, F>
        where
            F: FnOnce(Self::Output) -> U,
            Self: Sized,
        {
            Map{
                fut: self,
                f: Some(f),
            }
        }
        fn then<Fut,F>(self, f: F) -> Then<Self, F>
        where
            F: FnOnce(Self::Output) -> Fut,
            Fut: Future,
            Self: Sized,
        {
            Then{
                fut: self,
                f: Some(f),
            }
        }
    }

    pub struct Then<Fut,F> {
        fut: Fut,
        f: Option<F>,
    }

    impl<Fut,NextFut,F> Future for Then<Fut,F>
    where
        Fut: Future,
        NextFut: Future,
        F: FnOnce(Fut::Output) -> NextFut,
    {
        type Output = NextFut::Output;
        fn poll(&mut self, ctx: &Context) -> Poll<Self::Output> {
            match self.fut.poll(ctx) {
                Poll::Ready(v) => {
                    let f = self.f.take().unwrap();
                    f(v).poll(ctx)
                },
                _ => Poll::Pending,
            }
        }
    }

    pub struct Map<Fut,F> {
        fut: Fut,
        f: Option<F>,
    }

    impl<Fut,F,T> Future for Map<Fut,F>
    where
        Fut: Future,
        F: FnOnce(Fut::Output) -> T,
    {
        type Output = T;
        fn poll(&mut self, ctx: &Context) -> Poll<Self::Output> {
            match self.fut.poll(ctx) {
                Poll::Ready(v) => {
                    let cb = self.f.take().unwrap();
                    Poll::Ready(cb(v))
                },
                _ => Poll::Pending,
            }
        }
    }

    pub fn block_on<F>(mut f: F) -> F::Output
        where
            F: Future
    {
        NOTIFY.with(|v| {
            loop {
                if *v.borrow() {
                    *v.borrow_mut() = false;
                    let ctx = Context::from_waker(&Waker);
                    if let Poll::Ready(r) = f.poll(&ctx) {
                        return r;
                    }
                }
            }
        })
    }

    pub struct Ready<T>(T);

    impl<T: Copy> Future for Ready<T> {
        type Output = T;
        fn poll(&mut self, _: &Context) -> Poll<Self::Output> {
            Poll::Ready(self.0)
        }
    }

    pub fn ready<T>(v: T) -> Ready<T> {
        Ready(v)
    }
}
