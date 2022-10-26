use tch::{no_grad, Device, Tensor};

use crate::core::{StateDict, TensorCell};

/// A trait for anything that has trainable parameters.
pub trait Trainable: std::fmt::Debug + Send {
    /// Returns all trainable parameters that is not freezed.
    fn training_parameters(&self) -> Vec<TensorCell> {
        self.trainable_parameters()
            .to_vec()
            .into_iter()
            .filter(|tensor| tensor.lock().requires_grad())
            .collect()
    }

    /// Returns the size of the parameters of the module.
    fn trainable_parameter_size(&self) -> usize {
        self.training_parameters().len()
    }

    /// Load the parameters from another `StateDict`.
    fn load_trainable_parameters(&mut self, parameters: StateDict) {
        self.trainable_parameters().load(parameters);
    }

    /// Returns the trainable parameters of the module as `StateDict`.
    fn trainable_parameters(&self) -> StateDict;

    /// Return all tensors in the module, including non-trainable parameters.
    fn all_parameters(&self) -> Vec<TensorCell> {
        self.trainable_parameters().to_vec()
    }

    /// Initialize the trainable parameters of the module, with a certain distribution from `tch::nn::Init`.
    fn init(&mut self, init: tch::nn::Init) {
        no_grad(|| {
            for parameter in self.trainable_parameters().to_vec() {
                parameter.lock().init(init);
            }
        });
    }

    /// Freeze the trainable parameters of the module.
    fn freeze(&mut self) {
        for tensor in self.trainable_parameters().to_vec() {
            let mut tensor = tensor.lock();
            no_grad(|| {
                *tensor = tensor.set_requires_grad(false);
            });
        }
    }

    /// Unfreeze the trainable parameters of the module.
    fn unfreeze(&mut self) {
        for tensor in self.trainable_parameters().to_vec() {
            let mut tensor = tensor.lock();
            no_grad(|| {
                *tensor = tensor.set_requires_grad(true);
            });
        }
    }

    /// Move the parameters of the module to a certain device.
    fn to(&self, device: Device) {
        self.all_parameters().iter().for_each(|param| {
            let mut param = param.lock();
            let requires_grad = param.requires_grad();
            no_grad(|| {
                *param = param.to(device).set_requires_grad(requires_grad);
            })
        });
    }

    /// Clear the gradients of the trainable parameters of the module.
    fn zero_grad(&self) {
        self.trainable_parameters()
            .to_vec()
            .iter()
            .for_each(|param| {
                let mut param = param.lock();
                param.zero_grad();
            });
    }
}

/// A module is a neural network layer, which can be seen as a function from `Tensor` to `Tensor`, with some trainable parameters.
pub trait Module: Trainable {
    /// The forward function for Module.
    fn forward(&self, input: &Tensor) -> Tensor;
}

/// A module without trainable parameters.
pub trait NonParameterModule: Module {}

impl<T: NonParameterModule> Trainable for T {
    fn trainable_parameters(&self) -> StateDict {
        StateDict::new()
    }
}