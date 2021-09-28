use crate::sparse_set::SparseSet;

#[derive(Debug, Default)]
pub struct Nfa {
    states: Vec<State>,
    fragments: Vec<Fragment>,
}

impl Nfa {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn state(&self, state_id: StateId) -> &State {
        &self.states[state_id]
    }

    pub fn fragments(&self) -> &[Fragment] {
        &self.fragments
    }

    pub fn empty_closure(
        &self,
        state_id: StateId,
        state_id_set: &mut SparseSet,
        state_id_stack: &mut Vec<StateId>,
    ) {
        state_id_stack.clear();
        state_id_stack.push(state_id);
        while let Some(state_id) = state_id_stack.pop() {
            if state_id_set.contains(state_id) {
                continue;
            }
            state_id_set.insert(state_id);
            state_id_stack.extend(
                self.states[state_id]
                    .transitions
                    .iter()
                    .filter(|transition| transition.is_empty())
                    .map(|transition| transition.next_state_id),
            );
        }
    }

    pub fn add_state(&mut self) -> StateId {
        let state_id = self.states.len();
        self.states.push(State {
            matched_token: None,
            transitions: vec![],
        });
        state_id
    }

    pub fn add_transition(
        &mut self,
        state_id: StateId,
        byte_range: Option<ByteRange>,
        next_state_id: StateId,
    ) {
        let state = &mut self.states[state_id];
        state.transitions.push(Transition {
            byte_range,
            next_state_id,
        });
    }
}

pub type StateId = usize;

#[derive(Debug)]
pub struct State {
    pub matched_token: Option<usize>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Transition {
    pub byte_range: Option<ByteRange>,
    pub next_state_id: StateId,
}

impl Transition {
    pub fn is_empty(&self) -> bool {
        self.byte_range.is_none()
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct ByteRange {
    pub start: u8,
    pub end: u8,
}

impl ByteRange {
    pub fn contains(&self, byte: u8) -> bool {
        self.start <= byte && byte <= self.end
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Fragment {
    pub start_state_id: StateId,
    pub end_state_id: StateId,
}

#[derive(Debug, Default)]
pub struct Builder {
    nfa: Nfa,
    fragment_stack: Vec<Fragment>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn char(&mut self, ch: char) {
        let mut bytes = [0; 4];
        for &byte in ch.encode_utf8(&mut bytes).as_bytes() {
            self.byte_range(ByteRange {
                start: byte,
                end: byte,
            });
        }
    }

    fn byte_range(&mut self, byte_range: ByteRange) {
        let start_state_id = self.nfa.add_state();
        let end_state_id = self.nfa.add_state();
        self.nfa
            .add_transition(start_state_id, Some(byte_range), end_state_id);
        self.fragment_stack.push(Fragment {
            start_state_id,
            end_state_id,
        })
    }

    pub fn zero_or_one(&mut self) {
        let fragment = self.fragment_stack.pop().unwrap();
        let start_state_id = self.nfa.add_state();
        let end_state_id = self.nfa.add_state();
        self.nfa
            .add_transition(start_state_id, None, fragment.start_state_id);
        self.nfa.add_transition(start_state_id, None, end_state_id);
        self.nfa
            .add_transition(fragment.end_state_id, None, end_state_id);
        self.fragment_stack.push(Fragment {
            start_state_id,
            end_state_id,
        });
    }

    pub fn one_or_more(&mut self) {
        let fragment = self.fragment_stack.pop().unwrap();
        let start_state_id = self.nfa.add_state();
        let end_state_id = self.nfa.add_state();
        self.nfa
            .add_transition(start_state_id, None, fragment.start_state_id);
        self.nfa
            .add_transition(fragment.end_state_id, None, fragment.start_state_id);
        self.nfa
            .add_transition(fragment.end_state_id, None, end_state_id);
        self.fragment_stack.push(Fragment {
            start_state_id,
            end_state_id,
        });
    }

    pub fn zero_or_more(&mut self) {
        let fragment = self.fragment_stack.pop().unwrap();
        let start_state_id = self.nfa.add_state();
        let end_state_id = self.nfa.add_state();
        self.nfa
            .add_transition(start_state_id, None, fragment.start_state_id);
        self.nfa.add_transition(start_state_id, None, end_state_id);
        self.nfa
            .add_transition(fragment.end_state_id, None, fragment.start_state_id);
        self.nfa
            .add_transition(fragment.end_state_id, None, end_state_id);
        self.fragment_stack.push(Fragment {
            start_state_id,
            end_state_id,
        });
    }

    pub fn concatenate(&mut self) {
        let fragment_1 = self.fragment_stack.pop().unwrap();
        let fragment_0 = self.fragment_stack.pop().unwrap();
        self.nfa
            .add_transition(fragment_0.end_state_id, None, fragment_1.start_state_id);
        self.fragment_stack.push(Fragment {
            start_state_id: fragment_0.start_state_id,
            end_state_id: fragment_1.end_state_id,
        })
    }

    pub fn alternate(&mut self) {
        let fragment_1 = self.fragment_stack.pop().unwrap();
        let fragment_0 = self.fragment_stack.pop().unwrap();
        let start_state_id = self.nfa.add_state();
        let end_state_id = self.nfa.add_state();
        self.nfa
            .add_transition(start_state_id, None, fragment_0.start_state_id);
        self.nfa
            .add_transition(start_state_id, None, fragment_1.start_state_id);
        self.nfa
            .add_transition(fragment_0.end_state_id, None, end_state_id);
        self.nfa
            .add_transition(fragment_1.end_state_id, None, end_state_id);
        self.fragment_stack.push(Fragment {
            start_state_id,
            end_state_id,
        });
    }

    pub fn accept(&mut self, matched_token: usize) {
        let fragment = self.fragment_stack.pop().unwrap();
        self.nfa.states[fragment.end_state_id].matched_token = Some(matched_token);
        self.nfa.fragments.push(fragment);
    }

    pub fn build(self) -> Nfa {
        self.nfa
    }
}
