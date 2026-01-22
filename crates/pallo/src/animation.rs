use std::rc::Rc;

use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct Animations {
    list: FxHashMap<AnimationId, Animation>,
    id_cursor: usize,
}

pub type AnimationId = Rc<usize>;

impl Animations {
    pub fn add_linear(&mut self, duration_ms: f32) -> AnimationId {
        let id = Rc::new(self.id_cursor);
        self.id_cursor += 1;
        self.list.insert(id.clone(), Animation::new_linear(duration_ms));
        id
    }

    pub fn add_decaying(&mut self, decay_coeff: f32) -> AnimationId {
        let id = Rc::new(self.id_cursor);
        self.id_cursor += 1;
        self.list.insert(id.clone(), Animation::new_decaying(decay_coeff));
        id
    }

    pub fn set(&mut self, id: &AnimationId, value: f32) {
        self.list.get_mut(id).unwrap().set(value);
    }

    pub fn get(&mut self, id: &AnimationId) -> f32 {
        self.list[id].get()
    }

    pub fn tick(&mut self, frame_delta_ms: f32) {
        self.garbage_collect();
        for animation in self.list.values_mut() {
            animation.tick(frame_delta_ms);
        }
    }

    pub fn garbage_collect(&mut self) {
        self.list.retain(|id, _| Rc::strong_count(id) > 1);
    }
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    (1.0 - t) * start + t * end
}

enum Animation {
    Decaying { current: f32, decay_coeff: f32 },
    Linear { start: f32, current: f32, target: f32, duration_ms: f32, elapsed: f32 },
}

impl Animation {
    fn new_decaying(decay_coeff: f32) -> Self {
        Self::Decaying { current: 0.0, decay_coeff }
    }

    fn new_linear(duration_ms: f32) -> Self {
        Self::Linear { current: 0.0, start: 0.0, target: 0.0, duration_ms, elapsed: 0.0 }
    }

    fn set(&mut self, v: f32) {
        match self {
            Animation::Decaying { current, .. } => {
                *current = v;
            }
            Animation::Linear { start, current, target, elapsed, .. } => {
                *start = *current;
                *target = v;
                *elapsed = 0.0;
            }
        }
    }

    fn get(&self) -> f32 {
        match self {
            Animation::Decaying { current, .. } => *current,
            Animation::Linear { current, .. } => *current,
        }
    }

    fn tick(&mut self, delta_ms: f32) {
        match self {
            Animation::Decaying { current, decay_coeff } => {
                *current *= decay_coeff.powf(delta_ms);
            }
            Animation::Linear { start, current, target, duration_ms, elapsed } => {
                if elapsed < duration_ms {
                    *elapsed += delta_ms;
                    let t = (*elapsed / *duration_ms).clamp(0.0, 1.0);
                    *current = lerp(*start, *target, t);
                } else {
                    *current = *target;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::Animation;

    #[test]
    fn works() {
        let mut a = Animation::new_linear(100.0);
        a.set(1.0);
        a.tick(50.0);
        assert_eq!(a.get(), 0.5);
        a.tick(25.0);
        assert_eq!(a.get(), 0.75);
        a.tick(30.0);
        assert_eq!(a.get(), 1.0);
    }

    #[test]
    fn no_overshoot_after_idle() {
        let mut a = Animation::new_linear(50.0);
        a.set(1.0);
        a.tick(1000.0);
        assert_eq!(a.get(), 1.0);
        a.set(0.0);
        a.tick(60.0 * 60.0 * 1000.0);
        assert_eq!(a.get(), 0.0);
    }
}
