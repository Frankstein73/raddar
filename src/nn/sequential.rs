use std::ops::{Deref, DerefMut};
use raddar_derive::CallableModule;
use tch::Tensor;
use crate::core::{StateDict, TensorCell};
use crate::nn::Module;

use super::Trainable;

/// A module composed by a sequential of modules.
#[derive(Debug, CallableModule)]
pub struct Sequential(Vec<Box<dyn Module>>);

impl Deref for Sequential {
    type Target = Vec<Box<dyn Module>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Sequential {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Box<dyn Module>>> for Sequential {
    fn from(seq: Vec<Box<dyn Module>>) -> Self {
        Sequential(seq)
    }
}

impl Trainable for Sequential {
    fn trainable_parameters(&self) -> StateDict {
        let mut state_dict = StateDict::new();
        for (i, module) in self.iter().enumerate() {
            let child = module.trainable_parameters();
            state_dict.append_child(i.to_string(), child)
        }
        state_dict
    }

    fn all_parameters(&self) -> Vec<TensorCell> {
        let mut result: Vec<TensorCell> = vec![];
        for module in self.iter(){
            result.append(&mut module.all_parameters())
        }
        result
    }
}

impl Module for Sequential {
    fn forward(&self, input: &tch::Tensor) -> tch::Tensor {
        let mut x = input + 0;
        for module in self.iter(){
            x = module.forward(&x)
        }
        x
    }
}

#[macro_export]
macro_rules! seq {
    ($($x:expr),* $(,)?) => {
        {
            $crate::nn::sequential::Sequential::from(vec![$(Box::new($x) as Box<dyn $crate::nn::Module>,)*])
        }
    };
}