use derive_builder::Builder;
use raddar_derive::CallableModule;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tch::{no_grad, Device, Kind, Tensor};

use crate::core::StateDict;

use super::{Module, Trainable};
#[derive(Debug, CallableModule)]
pub struct Conv1d {
    pub conv_weight: Arc<Mutex<Tensor>>,
    pub conv_bias: Option<Arc<Mutex<Tensor>>>,
    pub config: Conv1dConfig,
}
#[derive(Builder, Debug)]
#[builder(pattern = "owned")]
pub struct Conv1dConfig {
    pub kernel_size: [i64; 1],
    pub in_channel: i64,
    pub out_channel: i64,
    #[builder(default = "[1]")]
    pub stride: [i64; 1],
    #[builder(default = "[0]")]
    pub padding: [i64; 1],
    #[builder(default = "[1]")]
    pub dilation: [i64; 1],
    #[builder(default = "0")]
    pub groups: i64,
    #[builder(default = "true")]
    pub bias: bool,
}

impl Trainable for Conv1d {
    fn trainable_parameters(&self) -> StateDict {
        let mut result = BTreeMap::new();
        result.insert("weight".to_owned(), self.conv_weight.clone());
        if let Some(bias) = &self.conv_bias {
            result.insert("bias".to_owned(), bias.clone());
        }
        StateDict::from_map(result)
    }
}
impl Module for Conv1d {
    fn forward(&self, input: &Tensor) -> Tensor {
        let weight = &self.conv_weight.lock().unwrap();
        if let Some(bias) = &self.conv_bias {
            let bias = bias.lock().unwrap();
            input.conv1d(
                &weight,
                Some(&*bias),
                &self.config.stride,
                &self.config.padding,
                &self.config.dilation,
                self.config.groups,
            )
        } else {
            input.conv1d::<&Tensor>(
                &weight,
                None,
                &self.config.stride,
                &self.config.padding,
                &self.config.dilation,
                self.config.groups,
            )
        }
    }
}
impl Conv1d {
    pub fn new(config: Conv1dConfig) -> Conv1d {
        let size: [i64; 3] = [config.out_channel, config.in_channel, config.kernel_size[0]];
        let mut conv_weight =
            Tensor::empty(&size, (Kind::Double, Device::Cpu)).set_requires_grad(true);
        let mut conv_bias = Tensor::empty(&[config.out_channel], (Kind::Double, Device::Cpu))
            .set_requires_grad(true);
        no_grad(|| {
            conv_weight.init(tch::nn::Init::KaimingUniform);
            conv_bias.init(tch::nn::Init::KaimingUniform);
        });
        Conv1d {
            conv_weight: Arc::new(Mutex::new(conv_weight)),
            conv_bias: if config.bias {
                Some(Arc::new(Mutex::new(conv_bias)))
            } else {
                None
            },
            config,
        }
    }
}