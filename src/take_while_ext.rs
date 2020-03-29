/// The standard rust `take_while` does not return
/// the last item it consumes. This makes it virtually
/// impossible to halt an iterator based on some condition,
/// and also return the last item consumed before halting.
/// A bit of a pain when doing error handling. Itertools
/// has a bit of a janky solution, but it seemed more elegant
/// to write a new one. I present: `TakeWhileLast`, a `take_while`
/// that includes the last element processed by the closure.
pub trait TakeWhileLastExt: Iterator + Sized {
    fn take_while_last<P>(self, predicate: P) -> TakeWhileLast<Self, P>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        TakeWhileLast {
            iter: self,
            done: false,
            predicate,
        }
    }
}

// Add other iterator types here if you want to use this
impl<I: Iterator, F, B> TakeWhileLastExt for std::iter::Map<I, F> where F: FnMut(I::Item) -> B {}
impl<I: Iterator, F, B> TakeWhileLastExt for std::iter::FilterMap<I, F> where
    F: FnMut(I::Item) -> Option<B>
{
}
impl<T> TakeWhileLastExt for std::slice::Iter<'_, T> {}
impl<'a, T: Iterator<Item = &'a I>, I: 'a + Clone> TakeWhileLastExt for std::iter::Cloned<T> {}

pub struct TakeWhileLast<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    iter: I,
    done: bool,
    predicate: P,
}

impl<I, P> Iterator for TakeWhileLast<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        if !self.done {
            let item = self.iter.next();
            match item {
                Some(item) => {
                    self.done = !(self.predicate)(&item);
                    Some(item)
                }
                None => {
                    self.done = true;
                    None
                }
            }
        } else {
            None
        }
    }
}

#[test]
fn test_take_while_last() {
    assert_eq!(
        [1, 2, 3, 4]
            .iter()
            .cloned()
            .take_while_last(|i| (*i) < 3)
            .last()
            .unwrap(),
        3
    );
    assert_eq!(
        [1, 2].iter().cloned().take_while_last(|i| (*i) < 0).next(),
        Some(1)
    );
    assert_eq!(
        (&[])
            .iter()
            .cloned()
            .take_while_last(|i: &u8| (*i) != 0)
            .next(),
        None
    );
    assert_eq!(
        [1, 2].iter().cloned().take_while_last(|i| (*i) < 3).last(),
        Some(2)
    );
}
