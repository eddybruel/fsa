use std::{
    iter::{self, Cloned, Enumerate, Zip},
    slice::{Chunks, Iter},
};

#[derive(Clone, Debug)]
pub struct Dfa {
    states: Vec<Option<usize>>,
    transitions: Vec<StateId>,
    token_count: usize,
}

impl Dfa {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn token_count(&self) -> usize {
        self.token_count
    }

    pub fn states(&self) -> States {
        States {
            iter: self
                .states
                .iter()
                .cloned()
                .zip(self.transitions.chunks(u8::MAX as usize))
                .enumerate(),
        }
    }

    pub fn longest_match<'a, B>(&self, mut bytes: B) -> Option<(usize, B)>
    where
        B: Clone + Iterator<Item = u8>,
    {
        let mut longest_match = None;
        let mut state_id = start_state_id();
        while let Some(byte) = bytes.next() {
            let offset = state_id * (u8::MAX as usize + 1) + byte as usize;
            state_id = self.transitions[offset];
            if state_id == dead_state_id() {
                break;
            }
            if let Some(token) = self.states[state_id] {
                longest_match = Some((token, bytes.clone()));
            }
        }
        longest_match
    }

    pub fn add_state(&mut self, matched_token: Option<usize>) -> StateId {
        let state_id = self.states.len();
        self.states.push(matched_token);
        self.transitions
            .extend(iter::repeat(dead_state_id()).take((u8::MAX as usize) + 1));
        if let Some(matched_token) = matched_token {
            self.token_count = self.token_count.max(matched_token + 1);
        }
        state_id
    }

    pub fn add_transition(&mut self, state_id: StateId, byte: u8, next_state_id: StateId) {
        let offset = state_id * (u8::MAX as usize + 1) + byte as usize;
        self.transitions[offset] = next_state_id;
    }
}

impl Default for Dfa {
    fn default() -> Self {
        let mut dfa = Self {
            states: Vec::new(),
            transitions: Vec::new(),
            token_count: 0,
        };
        dfa.add_state(None);
        dfa
    }
}

pub type StateId = usize;

#[derive(Debug)]
pub struct States<'a> {
    iter: Enumerate<Zip<Cloned<Iter<'a, Option<usize>>>, Chunks<'a, StateId>>>,
}

impl<'a> Iterator for States<'a> {
    type Item = (StateId, State<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (state_id, (matched_token, transitions)) = self.iter.next()?;
        Some((
            state_id,
            State {
                matched_token,
                transitions,
            },
        ))
    }
}

#[derive(Debug)]
pub struct State<'a> {
    matched_token: Option<usize>,
    transitions: &'a [StateId],
}

impl<'a> State<'a> {
    pub fn matched_token(&self) -> Option<usize> {
        self.matched_token
    }

    pub fn transitions(&self) -> Transitions<'a> {
        Transitions {
            iter: self.transitions.iter().cloned().enumerate(),
        }
    }
}

#[derive(Debug)]
pub struct Transitions<'a> {
    iter: Enumerate<Cloned<Iter<'a, StateId>>>,
}

impl<'a> Iterator for Transitions<'a> {
    type Item = Transition;

    fn next(&mut self) -> Option<Self::Item> {
        let (byte, next_state_id) = self.iter.next()?;
        Some(Transition {
            byte: byte as u8,
            next_state_id,
        })
    }
}

#[derive(Debug)]
pub struct Transition {
    pub byte: u8,
    pub next_state_id: StateId,
}

pub fn dead_state_id() -> StateId {
    0
}

pub fn start_state_id() -> StateId {
    1
}
