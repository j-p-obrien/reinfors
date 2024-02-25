pub trait Testing {
    type Associated;
    fn unconditional(&self);
    fn conditional_on_static(&self) -> &'static [Self::Associated]
    where
        Self::Associated: 'static;
}

static static_array: i32 = [1, 2];

pub struct Tester {
    inner: i32,
}

impl Testing for Tester {
    type Associated = i32;

    fn unconditional(&self) {
        ()
    }
}
