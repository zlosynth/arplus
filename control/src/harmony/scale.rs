use heapless::Vec;

use super::tonic::Tonic;

pub type Step = usize;

const Q: Step = 1;
const S: Step = 2;
const T: Step = 4;

pub struct Scale {
    tonic: Tonic,
    ascending: Vec<Step, 7>,
}
