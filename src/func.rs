pub struct Func<'a, T> {
    func: Box<dyn Fn(&T) -> f64 + 'a>,
}

impl<'a, T> Func<'a, T> {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(&T) -> f64 + 'a,
    {
        Self {
            func: Box::new(func),
        }
    }

    pub fn calculate(&self, value: &T) -> f64 {
        (self.func)(value)
    }
}
