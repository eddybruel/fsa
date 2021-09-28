use {
    crate::{
        dfa::{self, Dfa, StateId},
        nfa::{self, Nfa},
        sparse_set::SparseSet,
    },
    std::{collections::HashMap, mem, rc::Rc},
};

pub struct Determinizer<'a> {
    inner: DeterminizerInner<'a>,
    nfa_state_id_set: SparseSet,
    nfa_state_id_stack: Vec<nfa::StateId>,
    nfa_state_ids: Vec<nfa::StateId>,
    matched_tokens: Vec<usize>,
}

impl<'a> Determinizer<'a> {
    pub fn new(nfa: &'a Nfa) -> Self {
        Self {
            inner: DeterminizerInner::new(nfa),
            nfa_state_id_set: SparseSet::new(nfa.state_count()),
            nfa_state_id_stack: Vec::new(),
            nfa_state_ids: Vec::new(),
            matched_tokens: Vec::new(),
        }
    }

    pub fn determinize(mut self) -> Dfa {
        self.inner.determinize(
            &mut self.nfa_state_id_set,
            &mut self.nfa_state_id_stack,
            &mut self.nfa_state_ids,
            &mut self.matched_tokens,
        )
    }
}

struct DeterminizerInner<'a> {
    nfa: &'a Nfa,
    dfa: Dfa,
    states: Vec<Rc<State>>,
    state_ids_by_state: HashMap<Rc<State>, StateId>,
}

impl<'a> DeterminizerInner<'a> {
    fn new(nfa: &'a Nfa) -> Self {
        let dead_state = Rc::new(State::default());
        let mut state_ids_by_state = HashMap::new();
        state_ids_by_state.insert(dead_state.clone(), dfa::dead_state_id());
        Self {
            nfa,
            dfa: Dfa::new(),
            states: vec![dead_state],
            state_ids_by_state,
        }
    }

    fn determinize(
        mut self,
        nfa_state_id_set: &mut SparseSet,
        nfa_state_id_stack: &mut Vec<nfa::StateId>,
        nfa_state_ids: &mut Vec<nfa::StateId>,
        matched_tokens: &mut Vec<usize>,
    ) -> Dfa {
        let mut state_id_stack =
            vec![self.create_start_state(nfa_state_id_set, nfa_state_id_stack)];
        while let Some(state_id) = state_id_stack.pop() {
            for byte in 0..=u8::MAX {
                let (next_state_id, is_new) = self.get_or_create_next_state(
                    state_id,
                    byte,
                    nfa_state_id_set,
                    nfa_state_id_stack,
                    nfa_state_ids,
                    matched_tokens,
                );
                self.dfa.add_transition(state_id, byte, next_state_id);
                if is_new {
                    state_id_stack.push(next_state_id);
                }
            }
        }
        self.dfa
    }

    fn create_start_state(
        &mut self,
        nfa_state_id_set: &mut SparseSet,
        nfa_state_id_stack: &mut Vec<nfa::StateId>,
    ) -> StateId {
        let start_state_id = self.dfa.add_state(None);
        self.start_nfa_state_id_set(nfa_state_id_set, nfa_state_id_stack);
        let start_state = Rc::new(State {
            matched_token: None,
            nfa_state_ids: nfa_state_id_set.iter().collect::<Vec<_>>(),
        });
        self.states.push(start_state.clone());
        self.state_ids_by_state.insert(start_state, start_state_id);
        start_state_id
    }

    fn start_nfa_state_id_set(
        &mut self,
        nfa_state_id_set: &mut SparseSet,
        nfa_state_id_stack: &mut Vec<nfa::StateId>,
    ) {
        nfa_state_id_set.clear();
        for &fragment in self.nfa.fragments() {
            self.nfa.empty_closure(
                fragment.start_state_id,
                nfa_state_id_set,
                nfa_state_id_stack,
            );
        }
    }

    fn get_or_create_next_state(
        &mut self,
        state_id: StateId,
        byte: u8,
        nfa_state_id_set: &mut SparseSet,
        nfa_state_id_stack: &mut Vec<nfa::StateId>,
        nfa_state_ids: &mut Vec<nfa::StateId>,
        matched_tokens: &mut Vec<usize>,
    ) -> (StateId, bool) {
        self.next_nfa_state_id_set(state_id, byte, nfa_state_id_set, nfa_state_id_stack);
        self.get_or_create_state(nfa_state_id_set, nfa_state_ids, matched_tokens)
    }

    fn next_nfa_state_id_set(
        &mut self,
        state_id: StateId,
        byte: u8,
        nfa_state_id_set: &mut SparseSet,
        nfa_state_id_stack: &mut Vec<nfa::StateId>,
    ) {
        let state = &self.states[state_id];
        nfa_state_id_set.clear();
        for &nfa_state_id in &state.nfa_state_ids {
            for transition in &self.nfa.state(nfa_state_id).transitions {
                if transition
                    .byte_range
                    .as_ref()
                    .map_or(false, |byte_range| byte_range.contains(byte))
                {
                    self.nfa.empty_closure(
                        transition.next_state_id,
                        nfa_state_id_set,
                        nfa_state_id_stack,
                    );
                }
            }
        }
    }

    fn get_or_create_state(
        &mut self,
        nfa_state_id_set: &SparseSet,
        nfa_state_ids: &mut Vec<StateId>,
        matched_tokens: &mut Vec<usize>,
    ) -> (StateId, bool) {
        nfa_state_ids.clear();
        nfa_state_ids.extend(&*nfa_state_id_set);
        matched_tokens.clear();
        matched_tokens.extend(nfa_state_id_set.iter().filter_map({
            let nfa = &self.nfa;
            move |nfa_state_id| nfa.state(nfa_state_id).matched_token
        }));
        assert!(matched_tokens.len() <= 1);
        let matched_token = matched_tokens.first().cloned();
        let state = State {
            nfa_state_ids: mem::replace(nfa_state_ids, Vec::new()),
            matched_token,
        };
        match self.state_ids_by_state.get(&state) {
            Some(&state_id) => {
                *nfa_state_ids = state.nfa_state_ids;
                (state_id, false)
            }
            None => {
                let state = Rc::new(state);
                let state_id = self.dfa.add_state(matched_token);
                self.states.push(state.clone());
                self.state_ids_by_state.insert(state, state_id);
                (state_id, true)
            }
        }
    }
}

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq)]
struct State {
    nfa_state_ids: Vec<nfa::StateId>,
    matched_token: Option<usize>,
}
