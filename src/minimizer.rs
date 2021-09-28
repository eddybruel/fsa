use {
    crate::{
        dfa::{Dfa, StateId},
        vec_set::VecSet,
    },
    std::mem,
};

pub struct Minimizer {
    incoming_transitions: Vec<Vec<StateId>>,
    partitions: Vec<VecSet<StateId>>,
    partition_id_stack: Vec<PartitionId>,
}

impl Minimizer {
    pub fn new(dfa: &Dfa) -> Self {
        Self {
            incoming_transitions: incoming_transitions(dfa),
            partitions: initial_partitions(dfa),
            partition_id_stack: vec![0],
        }
    }

    pub fn minimize(
        &mut self,
        previous_state_id_set: &mut VecSet<StateId>,
        state_id_set_0: &mut VecSet<StateId>,
        state_id_set_1: &mut VecSet<StateId>,
    ) {
        while let Some(partition_id) = self.partition_id_stack.pop() {
            for byte in 0..u8::MAX {
                self.previous_states(partition_id, byte, previous_state_id_set);
                // TODO: Only iterate over those partitions that contain a previous state
                for partition_id in 0..self.partitions.len() {
                    self.partitions[partition_id]
                        .intersection(&previous_state_id_set)
                        .into_vec_set(state_id_set_0);
                    if state_id_set_0.is_empty() {
                        continue;
                    }
                    self.partitions[partition_id]
                        .difference(&previous_state_id_set)
                        .into_vec_set(state_id_set_1);
                    if state_id_set_1.is_empty() {
                        continue;
                    }
                    mem::swap(&mut self.partitions[partition_id], state_id_set_0);
                    self.partitions.push(state_id_set_1.clone());
                }
            }
        }
    }

    pub fn previous_states(
        &mut self,
        partition_id: PartitionId,
        byte: u8,
        previous_state_id_set: &mut VecSet<StateId>,
    ) {
        let mut previous_state_ids = mem::replace(previous_state_id_set, VecSet::new()).into_vec();
        previous_state_ids.clear();
        for &state_id in &self.partitions[partition_id] {
            let offset = state_id * (u8::MAX as usize + 1) + byte as usize;
            previous_state_ids.extend(&self.incoming_transitions[offset]);
        }
        previous_state_ids.sort();
        previous_state_ids.dedup();
        *previous_state_id_set = unsafe { VecSet::from_vec_unchecked(previous_state_ids) };
    }
}

type PartitionId = usize;

fn incoming_transitions(dfa: &Dfa) -> Vec<Vec<StateId>> {
    let mut incoming_transitions = vec![Vec::new(); dfa.state_count() * u8::MAX as usize];
    for (state_id, state) in dfa.states() {
        for transition in state.transitions() {
            let offset = state_id * (u8::MAX as usize + 1) + transition.byte as usize;
            incoming_transitions[offset].push(state_id);
        }
    }
    incoming_transitions
}

fn initial_partitions(dfa: &Dfa) -> Vec<VecSet<StateId>> {
    let mut partitions = vec![Vec::new(); dfa.token_count() + 1];
    for (state_id, state) in dfa.states() {
        match state.matched_token() {
            Some(matched_token) => &mut partitions[matched_token + 1],
            None => &mut partitions[0],
        }
        .push(state_id)
    }
    partitions
        .into_iter()
        .map(|partitions| unsafe { VecSet::from_vec_unchecked(partitions) })
        .collect::<Vec<_>>()
}
