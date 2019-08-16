use mmm::{future::*, task::*};

#[derive(Default)]
struct MyFuture {
    counter: u32,
}

impl Future for MyFuture {
    type Output = i32;
    fn poll(&mut self, ctx: &Context) -> Poll<Self::Output> {
        match self.counter {
            3 => Poll::Ready(3),
            _ => {
                self.counter += 1;
                ctx.waker().wake();
                Poll::Pending
            }
        }
    }
}

struct AddOneFuture<T>(T);

impl<T> Future for AddOneFuture<T>
where
    T: Future,
    T::Output: std::ops::Add<i32, Output = i32>,
{
    type Output = i32;
    fn poll(&mut self, ctx: &Context) -> Poll<Self::Output> {
        match self.0.poll(ctx) {
            Poll::Ready(v) => Poll::Ready(v + 1),
            _ => Poll::Pending,
        }
    }
}

fn main() {
    //let my_future = MyFuture::default();
    //("output {}", run(AddOneFuture(my_future)));
    println!(
        "ready {:?}",
        block_on(
            ready(10)
                .then(|v| ready(v - 3))
                .map(|v| v + 5)
                .then(|v| ready(v * 2))
                .map(|v| v / 4)
                .then(|v| {
                    let q: Result<i32, ()> = Ok(v);
                    ready(q)
                })
                .and_then(|v| ready(Ok(v + 1)))
        )
    );
}
