use raddar_derive::{ArchitectureBuilder, CallableModule};
use tch::Tensor;

use crate::{
    nn::{
        AdaptiveAveragePooling2D, AdaptiveAveragePooling2DBuilder, Conv2dBuilder, DropoutBuilder,
        LinearBuilder, MaxPooling2DBuilder, Module, ReLU, Sequential, Trainable,
    },
    seq,
};

use super::{Mod, TrainableDict};

/// AlexNet architecture.
///
/// See [ImageNet Classification with Deep Convolutional Neural Networks](https://papers.nips.cc/paper/4824-imagenet-classification-with-deep-convolutional-neural-networks.pdf).
#[derive(Debug, CallableModule, ArchitectureBuilder)]
pub struct AlexNet {
    pub features: Mod<Sequential>,
    pub avgpool: Mod<AdaptiveAveragePooling2D>,
    pub classifier: Mod<Sequential>,

    #[builder(default = "1000")]
    pub num_classes: i64,

    #[builder(default = "0.5")]
    pub dropout: f64,
}

impl Trainable for AlexNet {
    fn child_modules(&self) -> TrainableDict {
        let mut result = TrainableDict::new();
        result.insert("features".to_owned(), self.features.clone());
        result.insert("classifier".to_owned(), self.classifier.clone());
        result
    }
}

impl Module for AlexNet {
    fn forward(&self, input: &Tensor) -> Tensor {
        let mut output = (self.features)(input);
        output = (self.avgpool)(&output);
        output = output.flatten(1, 3);
        output = (self.classifier)(&output);
        output
    }
}

impl AlexNet {
    pub fn new(config: AlexNetConfig) -> Self {
        let features = seq!(
            Conv2dBuilder::default()
                .in_channel(3)
                .out_channel(64)
                .kernel_size([11, 11])
                .stride([4, 4])
                .padding([2, 2])
                .build(),
            Mod::new(ReLU),
            MaxPooling2DBuilder::default()
                .kernel_size([3, 3])
                .stride([2, 2])
                .build(),
            Conv2dBuilder::default()
                .in_channel(64)
                .out_channel(192)
                .kernel_size([5, 5])
                .padding([2, 2])
                .build(),
            Mod::new(ReLU),
            MaxPooling2DBuilder::default()
                .kernel_size([3, 3])
                .stride([2, 2])
                .build(),
            Conv2dBuilder::default()
                .in_channel(192)
                .out_channel(384)
                .kernel_size([3, 3])
                .padding([1, 1])
                .build(),
            Mod::new(ReLU),
            Conv2dBuilder::default()
                .in_channel(384)
                .out_channel(256)
                .kernel_size([3, 3])
                .padding([1, 1])
                .build(),
            Mod::new(ReLU),
            Conv2dBuilder::default()
                .in_channel(256)
                .out_channel(256)
                .kernel_size([3, 3])
                .padding([1, 1])
                .build(),
            Mod::new(ReLU),
            MaxPooling2DBuilder::default()
                .kernel_size([3, 3])
                .stride([2, 2])
                .build(),
        );
        let avgpool = AdaptiveAveragePooling2DBuilder::default()
            .output_size([6, 6])
            .build();
        let classifier = seq!(
            DropoutBuilder::default().p(config.dropout).build(),
            LinearBuilder::default()
                .input_dim(9216)
                .output_dim(4096)
                .build(),
            Mod::new(ReLU),
            DropoutBuilder::default().p(config.dropout).build(),
            LinearBuilder::default()
                .input_dim(4096)
                .output_dim(4096)
                .build(),
            Mod::new(ReLU),
            LinearBuilder::default()
                .input_dim(4096)
                .output_dim(config.num_classes)
                .build(),
        );
        AlexNet {
            features,
            avgpool,
            classifier,
            num_classes: config.num_classes,
            dropout: config.dropout,
        }
    }
}
pub fn alexnet(num_classes: i64, dropout: f64, _pretrained: bool) -> Mod<AlexNet> {
    let model = AlexNetBuilder::default()
        .num_classes(num_classes)
        .dropout(dropout)
        .build();
    model
}
