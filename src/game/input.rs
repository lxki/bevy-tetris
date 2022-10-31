use super::{utils::Timer, REPEAT_DURATION, WAIT_DURATION};

enum RepeatedActionState {
    Inactive,
    Wait,
    Repeat,
}

struct RepeatedAction {
    state: RepeatedActionState,
    timer: Timer,
    active: bool,
    wait_duration: u32,
    repeat_duration: u32,
}

impl RepeatedAction {
    fn new(wait_duration: u32, repeat_duration: u32) -> Self {
        Self {
            state: RepeatedActionState::Inactive,
            timer: Timer::new(),
            active: false,
            wait_duration,
            repeat_duration,
        }
    }

    fn tick(&mut self, active: bool) {
        if !active {
            self.state = RepeatedActionState::Inactive;
            self.active = false;
        } else {
            (self.state, self.active) = match self.state {
                RepeatedActionState::Inactive => {
                    self.timer.restart();
                    (RepeatedActionState::Wait, true)
                }
                RepeatedActionState::Wait => {
                    if self.timer.tick_and_restart_if_elapsed(self.wait_duration) {
                        (RepeatedActionState::Repeat, true)
                    } else {
                        (RepeatedActionState::Wait, false)
                    }
                }
                RepeatedActionState::Repeat => {
                    let active = self.timer.tick_and_restart_if_elapsed(self.repeat_duration);
                    (RepeatedActionState::Repeat, active)
                }
            };
        }
    }

    fn active(&self) -> bool {
        self.active
    }
}

pub trait Input {
    fn move_left(&self) -> bool;
    fn move_right(&self) -> bool;
    fn rotate(&self) -> bool;
    fn fast_drop(&self) -> bool;
    fn instant_drop(&self) -> bool;
}

pub struct SmartInput {
    move_left: RepeatedAction,
    move_right: RepeatedAction,
    rotate: RepeatedAction,
    fast_drop: bool,
    instant_drop: bool,
}

impl SmartInput {
    pub fn new() -> Self {
        Self {
            move_left: RepeatedAction::new(WAIT_DURATION, REPEAT_DURATION),
            move_right: RepeatedAction::new(WAIT_DURATION, REPEAT_DURATION),
            rotate: RepeatedAction::new(WAIT_DURATION, REPEAT_DURATION),
            fast_drop: false,
            instant_drop: false,
        }
    }

    pub fn tick(&mut self, input: &dyn Input) {
        self.move_left.tick(input.move_left());
        self.move_right.tick(input.move_right());
        self.rotate.tick(input.rotate());
        self.fast_drop = input.fast_drop();
        self.instant_drop = input.instant_drop();
    }
}

impl Input for SmartInput {
    fn move_left(&self) -> bool {
        self.move_left.active()
    }

    fn move_right(&self) -> bool {
        self.move_right.active()
    }

    fn rotate(&self) -> bool {
        self.rotate.active()
    }

    fn fast_drop(&self) -> bool {
        self.fast_drop
    }

    fn instant_drop(&self) -> bool {
        self.instant_drop
    }
}
