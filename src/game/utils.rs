use super::Id;

pub struct Timer {
    tick: u32,
}

impl Timer {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
    }

    pub fn restart(&mut self) {
        self.tick = 0;
    }

    pub fn has_elapsed(&self, duration: u32) -> bool {
        self.tick >= duration
    }

    pub fn tick_and_restart_if_elapsed(&mut self, duration: u32) -> bool {
        self.tick();
        if self.has_elapsed(duration) {
            self.restart();
            true
        } else {
            false
        }
    }
}

pub struct IdGenerator {
    next_id: Id,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            next_id: Id::new(1).unwrap(),
        }
    }
}

impl FnOnce<()> for IdGenerator {
    type Output = Id;

    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        unreachable!()
    }
}

impl FnMut<()> for IdGenerator {
    extern "rust-call" fn call_mut(&mut self, _args: ()) -> Self::Output {
        let id = self.next_id;
        self.next_id = Id::new(id.get() + 1).unwrap();
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ids() {
        let mut gen_id = IdGenerator::new();
        assert_eq!(1, gen_id().get());
        assert_eq!(2, gen_id().get());
        assert_eq!(3, gen_id().get());
    }
}
