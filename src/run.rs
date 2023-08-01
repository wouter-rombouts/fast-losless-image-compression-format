use std::{ops::Range, iter::Rev};

#[derive(Clone)]
pub struct RunCountdown
{ pub color : u8,
  pub run : Rev<Range<usize>>
}


    impl PartialEq for RunCountdown
{
    fn eq(&self, other: &Self) -> bool {
        self.color == other.color
    }
}
