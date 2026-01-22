use std::{
    cell::{Ref, RefCell, RefMut},
    ops::Deref,
    rc::Rc,
};

use rustc_hash::FxHashSet;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct ComputedId(usize);

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
struct SignalId(usize);

impl SignalId {
    fn new(rt: &Rc<Runtime>) -> Self {
        let mut signals = rt.signals.borrow_mut();
        SignalId(if let Some(slot_idx) = signals.iter().position(|i| i.is_none()) {
            signals[slot_idx] = Some(SignalData { dependents: Default::default() });
            slot_idx
        } else {
            let idx = signals.len();
            signals.push(Some(SignalData { dependents: Default::default() }));
            idx
        })
    }
}

struct SignalData {
    dependents: FxHashSet<ComputedId>,
}

struct ComputedData {
    dependencies: FxHashSet<SignalId>,
    dirty: bool,
}

impl ComputedData {
    fn new(rt: &Rc<Runtime>) -> ComputedId {
        let mut computeds = rt.computeds.borrow_mut();
        ComputedId(if let Some(slot_idx) = computeds.iter().position(|i| i.is_none()) {
            computeds[slot_idx] = Some(Self { dependencies: Default::default(), dirty: true });
            slot_idx
        } else {
            let idx = computeds.len();
            computeds.push(Some(Self { dependencies: Default::default(), dirty: true }));
            idx
        })
    }
}

#[derive(Default)]
pub struct Runtime {
    computeds: RefCell<Vec<Option<ComputedData>>>,
    signals: RefCell<Vec<Option<SignalData>>>,
    current_computed_id: RefCell<Option<ComputedId>>,
}

pub struct SignalCx {
    rt: Rc<Runtime>,
}

impl Default for SignalCx {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalCx {
    pub fn new() -> Self {
        Self { rt: Default::default() }
    }

    pub fn signal<T: 'static>(&self, initial_value: T) -> Signal<T> {
        Signal::new(self.rt.clone(), initial_value)
    }

    pub fn signal_default<T: Default + 'static>(&self) -> Signal<T> {
        Signal::new(self.rt.clone(), Default::default())
    }

    pub fn computed<T: Clone + 'static>(&self, cb: impl Fn() -> T + 'static) -> Computed<T> {
        Computed::new(self.rt.clone(), cb)
    }

    pub fn computed_static<T: Clone + 'static>(&self, value: T) -> Computed<T> {
        Computed::new_static(value)
    }
}

pub enum Computed<T> {
    Dynamic { id: ComputedId, getter: Rc<dyn Fn() -> T>, rt: Rc<Runtime> },
    Static { value: T, has_supplied_once: RefCell<bool> },
}

impl<T: Default + Clone + 'static> Default for Computed<T> {
    fn default() -> Self {
        Computed::new_static(T::default())
    }
}

impl<T: Clone + 'static> From<T> for Computed<T> {
    fn from(value: T) -> Self {
        Self::new_static(value)
    }
}

impl<T: Clone + 'static> From<Signal<T>> for Computed<T> {
    fn from(value: Signal<T>) -> Self {
        value.as_computed()
    }
}

impl<T: Clone> Clone for Computed<T> {
    fn clone(&self) -> Self {
        match self {
            Computed::Dynamic { getter, rt, .. } => {
                let id = ComputedData::new(rt);
                Self::Dynamic { id, getter: getter.clone(), rt: rt.clone() }
            }
            Computed::Static { value, .. } => {
                Self::Static { value: value.clone(), has_supplied_once: RefCell::new(false) }
            }
        }
    }
}

impl<T: Clone + 'static> Computed<T> {
    fn new(rt: Rc<Runtime>, getter: impl Fn() -> T + 'static) -> Self {
        let id = ComputedData::new(&rt);
        Self::Dynamic { rt, getter: Rc::new(getter), id }
    }

    pub fn new_static(value: T) -> Self {
        Self::Static { value, has_supplied_once: RefCell::new(false) }
    }

    #[inline]
    pub fn get(&self) -> T {
        match self {
            Computed::Dynamic { getter, .. } => (getter)(),
            Computed::Static { value, .. } => value.clone(),
        }
    }

    pub fn map<O: Clone + 'static>(&self, mapper: impl Fn(T) -> O + 'static) -> Computed<O> {
        match self {
            Computed::Dynamic { getter, rt, .. } => {
                let getter = getter.clone();
                Computed::new(rt.clone(), move || (mapper)((getter)()))
            }
            Computed::Static { value, .. } => Computed::new_static((mapper)(value.clone())),
        }
    }

    pub fn next(&self) -> Option<T> {
        match self {
            Computed::Dynamic { id, rt, .. } => {
                if rt.computeds.borrow()[id.0].as_ref().unwrap().dirty {
                    let prev_computed = {
                        let temp = &mut rt.computeds.borrow_mut()[id.0];
                        let computed = temp.as_mut().unwrap();
                        let mut signals = rt.signals.borrow_mut();
                        for signal_id in computed.dependencies.drain() {
                            signals[signal_id.0].as_mut().unwrap().dependents.remove(id);
                        }
                        computed.dirty = false;

                        let mut current_computed = rt.current_computed_id.borrow_mut();
                        let prev_computed = *current_computed;
                        *current_computed = Some(*id);
                        prev_computed
                    };

                    let output = Some(self.get());

                    *rt.current_computed_id.borrow_mut() = prev_computed;

                    output
                } else {
                    None
                }
            }
            Computed::Static { has_supplied_once, value } => {
                if !*has_supplied_once.borrow() {
                    *has_supplied_once.borrow_mut() = true;
                    Some(value.clone())
                } else {
                    None
                }
            }
        }
    }

    pub fn into_memo(self) -> Memo<T> {
        Memo { computed: self, last_value: RefCell::new(None) }
    }
}

pub trait MapComputed<I, O> {
    fn map(self, mapper: impl Fn(I) -> O + 'static) -> Computed<O>;
}

impl<A: Clone + 'static, B: Clone + 'static, O: Clone + 'static> MapComputed<(A, B), O> for (Computed<A>, Computed<B>) {
    fn map(self, mapper: impl Fn((A, B)) -> O + 'static) -> Computed<O> {
        self.0.map(move |a| (mapper)((a, self.1.get())))
    }
}

impl<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static, O: Clone + 'static> MapComputed<(A, B, C), O>
    for (Computed<A>, Computed<B>, Computed<C>)
{
    fn map(self, mapper: impl Fn((A, B, C)) -> O + 'static) -> Computed<O> {
        self.0.map(move |a| (mapper)((a, self.1.get(), self.2.get())))
    }
}

impl<T> Drop for Computed<T> {
    fn drop(&mut self) {
        if let Self::Dynamic { rt, id, .. } = self {
            rt.computeds.borrow_mut()[id.0] = None;
        }
    }
}

pub struct Memo<T> {
    computed: Computed<T>,
    last_value: RefCell<Option<T>>,
}

impl<T: Clone> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self { computed: self.computed.clone(), last_value: RefCell::new(None) }
    }
}

impl<T: Clone + PartialEq + 'static> Memo<T> {
    pub fn next(&self) -> Option<T> {
        self.computed.next().and_then(|value| {
            let v = Some(value);
            let mut last_value = self.last_value.borrow_mut();
            if v != *last_value {
                last_value.clone_from(&v);
                v
            } else {
                None
            }
        })
    }

    pub fn get(&self) -> T {
        self.next();
        (*self.last_value.borrow()).clone().unwrap() // TODO: verify that unwrapping is always ok
    }

    pub fn get_ref(&self) -> Ref<'_, T> {
        self.next();
        Ref::map(self.last_value.borrow(), |v| v.as_ref().unwrap())
    }
}

impl<T> Deref for Memo<T> {
    type Target = Computed<T>;

    fn deref(&self) -> &Self::Target {
        &self.computed
    }
}

impl<T: Clone + 'static> From<Memo<T>> for Computed<T> {
    fn from(val: Memo<T>) -> Self {
        val.computed
    }
}

pub struct Signal<T> {
    id: SignalId,
    value: Rc<RefCell<T>>,
    rt: Rc<Runtime>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self { id: self.id, value: self.value.clone(), rt: self.rt.clone() }
    }
}

impl<T: 'static> Signal<T> {
    fn new(rt: Rc<Runtime>, initial_value: T) -> Self {
        Self { id: SignalId::new(&rt), value: Rc::new(RefCell::new(initial_value)), rt }
    }

    pub fn set(&self, value: T) {
        self.mark_depending_computeds_dirty();
        *(*self.value).borrow_mut() = value;
    }

    pub fn mutate<R>(&self, mutator: impl FnOnce(RefMut<'_, T>) -> R) {
        self.mark_depending_computeds_dirty();
        (mutator)((*self.value).borrow_mut());
    }

    pub fn get_ref(&self) -> Ref<'_, T> {
        self.update_current_computed_dependencies();
        self.value.borrow()
    }

    pub fn get_ref_fast(&self) -> Ref<'_, T> {
        self.value.borrow()
    }

    pub fn cx(&self) -> SignalCx {
        SignalCx { rt: self.rt.clone() }
    }

    fn mark_depending_computeds_dirty(&self) {
        let mut computeds = self.rt.computeds.borrow_mut();
        let signals = self.rt.signals.borrow();
        for computed_id in &signals[self.id.0].as_ref().unwrap().dependents {
            if let Some(c) = computeds[computed_id.0].as_mut() {
                c.dirty = true;
            }
        }
    }

    fn update_current_computed_dependencies(&self) {
        if let Some(id) = *self.rt.current_computed_id.borrow() {
            let mut computeds = self.rt.computeds.borrow_mut();
            let mut signals = self.rt.signals.borrow_mut();
            computeds[id.0].as_mut().unwrap().dependencies.insert(self.id);
            signals[self.id.0].as_mut().unwrap().dependents.insert(id);
        }
    }
}

impl<T: PartialEq + 'static> Signal<T> {
    pub fn set_if_changed(&self, value: T) -> bool {
        if *self.value.borrow() != value {
            self.mark_depending_computeds_dirty();
            *(*self.value).borrow_mut() = value;
            true
        } else {
            false
        }
    }
}

impl<T: Clone + 'static> Signal<T> {
    pub fn as_computed(&self) -> Computed<T> {
        let s = self.clone();
        Computed::new(self.rt.clone(), move || s.get())
    }

    pub fn map<O: Clone + 'static>(&self, mapper: impl Fn(T) -> O + 'static) -> Computed<O> {
        self.as_computed().map(mapper)
    }

    pub fn get(&self) -> T {
        self.update_current_computed_dependencies();
        self.value.borrow().clone()
    }

    pub fn get_fast(&self) -> T {
        self.value.borrow().clone()
    }
}

#[cfg(test)]
mod test {
    use crate::SignalCx;

    #[test]
    fn computed_returns_value_when_either_dependency_changes() {
        let cx = SignalCx::new();
        let a = cx.signal(1.0);
        let b = cx.signal(10.0);
        let eff = {
            let a = a.clone();
            let b = b.clone();
            cx.computed(move || a.get() * b.get())
        };
        assert_eq!(eff.next(), Some(10.0));
        assert_eq!(eff.next(), None);
        a.set(2.0);
        assert_eq!(eff.next(), Some(20.0));
        assert_eq!(eff.next(), None);
        b.set(2.0);
        assert_eq!(eff.next(), Some(4.0));
        assert_eq!(eff.next(), None);
    }

    #[test]
    fn computed_returns_new_value_only_once_after_many_dependencies_change() {
        let cx = SignalCx::new();
        let a = cx.signal(1.0);
        let b = cx.signal(10.0);
        let eff = {
            let a = a.clone();
            let b = b.clone();
            cx.computed(move || a.get() * b.get())
        };
        a.set(3.0);
        b.set(5.0);
        assert_eq!(eff.next(), Some(15.0));
        assert_eq!(eff.next(), None);
    }

    #[test]
    fn nested_computeds_update_as_expected() {
        let cx = SignalCx::new();
        let a = cx.signal(1.0);
        let b = cx.signal(10.0);
        let contains_a = {
            let a = a.clone();
            cx.computed(move || a.get() * 2.0)
        };
        let eff = {
            let contains_a = contains_a.clone();
            let b = b.clone();
            cx.computed(move || contains_a.get() + b.get())
        };
        assert_eq!(eff.next(), Some(12.0));
        assert_eq!(eff.next(), None);
        a.set(2.0);
        assert_eq!(eff.next(), Some(14.0));
        assert_eq!(eff.next(), None);
        b.set(2.0);
        assert_eq!(contains_a.next(), Some(4.0));
        assert_eq!(contains_a.next(), None);
        assert_eq!(eff.next(), Some(6.0));
        assert_eq!(eff.next(), None);
    }
}
