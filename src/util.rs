pub trait TakeWhileOkExt: Iterator + Sized {
    fn take_while_ok(self) -> TakeWhileOk<Self> {
        TakeWhileOk {
            iter: self,
            done: false,
        }
    }
}

// Add other iterator types here if you want to use this
impl<I: Iterator, F, B> TakeWhileOkExt for std::iter::Map<I, F> where F: FnMut(I::Item) -> B {}

pub struct TakeWhileOk<I> {
    iter: I,
    done: bool,
}

impl<I, T, E> Iterator for TakeWhileOk<I>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = Result<T, E>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.done {
            let item = self.iter.next();
            match item {
                Some(item) => Some(match item {
                    Ok(item) => Ok(item),
                    Err(err) => {
                        self.done = true;
                        Err(err)
                    }
                }),
                None => None,
            }
        } else {
            None
        }
    }
}
