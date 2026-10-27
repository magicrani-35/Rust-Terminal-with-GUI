use std::collections::HashMap;

use super::jobs::Job;

#[derive(Default)]
pub(crate) struct ShellState {
    pub(super) jobs: Vec<Job>,
    pub(super) command_history: Vec<String>,
    pub(super) history_append_from: usize,
    pub(super) variables: HashMap<String, String>,
}