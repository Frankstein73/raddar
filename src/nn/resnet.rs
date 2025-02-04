use std::{fmt::Debug, marker::PhantomData};

use raddar_derive::{ArchitectureBuilder, CallableModule};
use tch::Tensor;

use crate::{nn::ReLU, seq};

use super::{
    AdaptiveAveragePooling2DBuilder, BatchNorm2dBuilder, Conv2d, Conv2dBuilder, LinearBuilder,
    MaxPooling2DBuilder, Mod, Module, Sequential, Trainable, TrainableDict,
};

pub trait Block<U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy>: Module {
    fn expansion() -> i64;
    fn new_block(
        inplanes: i64,
        planes: i64,
        stride: [i64; 2],
        groups: i64,
        base_width: i64,
        dilation: [i64; 2],
        downsample: Option<Mod<Sequential>>,
        norm_layer: U,
    ) -> Mod<Self>;
}

pub fn conv3x3(
    in_planes: i64,
    out_planes: i64,
    stride: [i64; 2],
    groups: i64,
    dilation: [i64; 2],
) -> Mod<Conv2d> {
    Conv2dBuilder::default()
        .kernel_size([3, 3])
        .in_channel(in_planes)
        .out_channel(out_planes)
        .stride(stride)
        .groups(groups)
        .dilation(dilation)
        .bias(false)
        .padding(dilation)
        .build()
}

pub fn conv1x1(in_planes: i64, out_planes: i64, stride: [i64; 2]) -> Mod<Conv2d> {
    Conv2dBuilder::default()
        .kernel_size([1, 1])
        .in_channel(in_planes)
        .out_channel(out_planes)
        .stride(stride)
        .bias(false)
        .build()
}

pub fn batchnorm2d(num_features: i64) -> Mod<Sequential> {
    seq!(BatchNorm2dBuilder::default()
        .num_features(num_features)
        .build())
}

#[derive(Debug, CallableModule)]
pub struct BasicBlock {
    pub block: Mod<Sequential>,
    pub downsample: Option<Mod<Sequential>>,
}

impl Trainable for BasicBlock {
    fn child_modules(&self) -> TrainableDict {
        let mut result = TrainableDict::new();
        result.insert("block".to_owned(), self.block.clone());
        if let Some(ref downsample) = self.downsample {
            result.insert("downsample".to_owned(), downsample.clone());
        }
        result
    }
}

impl Module for BasicBlock {
    fn forward(&self, input: &Tensor) -> Tensor {
        let mut identity = input.copy();
        let mut output = (self.block)(input);
        if let Some(downsample) = &self.downsample {
            identity = (*downsample)(&identity);
        }
        output += identity;
        let relu = seq!(Mod::new(ReLU));
        (relu)(&output)
    }
}

impl<U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy> Block<U> for BasicBlock {
    fn expansion() -> i64 {
        1
    }

    fn new_block(
        in_planes: i64,
        planes: i64,
        stride: [i64; 2],
        groups: i64,
        base_width: i64,
        dilation: [i64; 2],
        downsample: Option<Mod<Sequential>>,
        norm_layer: U,
    ) -> Mod<Self> {
        assert!(groups == 1 && base_width == 64 && dilation == [1, 1]);
        let mut block: Sequential = Sequential::default();
        block.push(conv3x3(in_planes, planes, stride, groups, dilation));
        block.push(norm_layer(planes));
        block.push(Mod::new(ReLU));
        block.push(conv3x3(planes, planes, [1, 1], groups, dilation));

        block.push(norm_layer(planes));
        Mod::new(Self {
            block: Mod::new(block),
            downsample,
        })
    }
}

#[derive(Debug, CallableModule)]
pub struct BottleNeck {
    pub block: Mod<Sequential>,
    pub downsample: Option<Mod<Sequential>>,
}

impl Trainable for BottleNeck {
    fn child_modules(&self) -> TrainableDict {
        let mut result = TrainableDict::new();
        result.insert("block".to_owned(), self.block.clone());
        if let Some(downsample) = &self.downsample {
            result.insert("downsample".to_owned(), downsample.clone());
        }
        result
    }
}

impl Module for BottleNeck {
    fn forward(&self, input: &Tensor) -> Tensor {
        let mut identity = input.copy();
        let mut output = (self.block)(input);
        if let Some(downsample) = &self.downsample {
            identity = (*downsample)(&identity);
        }
        output += identity;
        let relu = seq!(Mod::new(ReLU));
        (relu)(&output)
    }
}

impl<U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy> Block<U> for BottleNeck {
    fn expansion() -> i64 {
        4
    }
    fn new_block(
        inplanes: i64,
        planes: i64,
        stride: [i64; 2],
        groups: i64,
        base_width: i64,
        dilation: [i64; 2],
        downsample: Option<Mod<Sequential>>,
        norm_layer: U,
    ) -> Mod<Self> {
        let width = (((planes as f64) * (base_width as f64) / 64.0) as i64) * groups;
        let mut block = Sequential::default();
        block.push(conv1x1(inplanes, width, [1, 1]));
        block.push(norm_layer(width));
        block.push(Mod::new(ReLU));
        block.push(conv3x3(width, width, stride, groups, dilation));
        block.push(norm_layer(width));
        block.push(Mod::new(ReLU));
        block.push(conv1x1(
            width,
            planes * <BottleNeck as Block<U>>::expansion(),
            [1, 1],
        ));
        block.push(norm_layer(planes * <BottleNeck as Block<U>>::expansion()));
        Mod::new(Self {
            block: Mod::new(block),
            downsample,
        })
    }
}

/// A ResNet model
///
/// See [Deep Residual Learning for Image Recognition](https://arxiv.org/abs/1512.03385).
#[derive(Debug, CallableModule, ArchitectureBuilder)]
pub struct ResNet<
    T: Block<U> + 'static,
    U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy + 'static,
> {
    #[builder(default = "64")]
    pub base_width: i64,
    #[builder(default = "1000")]
    pub num_classes: i64,
    #[builder]
    pub layers: [i64; 4],
    pub net: Mod<Sequential>,
    pub fc: Mod<Sequential>,
    #[builder(default = "[false, false, false]")]
    pub replace_stride_with_dilation: [bool; 3],
    #[builder(default = "1")]
    pub groups: i64,
    #[builder(default = "Self::default_norm_layer()")]
    pub norm_layer: U,
    #[builder(default = "[1, 1]")]
    pub dilation: [i64; 2],
    #[builder(default = "64")]
    pub inplanes: i64,
    #[builder(default = "PhantomData::<T>")]
    _phantom: PhantomData<T>,
}

trait DefaultNormLayer<U: Fn(i64) -> Mod<Sequential>> {
    fn default_norm_layer() -> U;
}

impl<T: Block<U>, U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy> DefaultNormLayer<U>
    for ResNetBuilder<T, U>
{
    default fn default_norm_layer() -> U {
        panic!("Norm layer should be set!")
    }
}

impl<T: Block<fn(i64) -> Mod<Sequential>>> DefaultNormLayer<fn(i64) -> Mod<Sequential>>
    for ResNetBuilder<T, fn(i64) -> Mod<Sequential>>
{
    fn default_norm_layer() -> fn(i64) -> Mod<Sequential> {
        return batchnorm2d;
    }
}

impl<T: Block<U>, U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy> Trainable for ResNet<T, U> {
    fn child_modules(&self) -> TrainableDict {
        let mut result = TrainableDict::new();
        result.insert("net".to_owned(), self.net.clone());
        result.insert("fc".to_owned(), self.fc.clone());
        result
    }
}

impl<T: Block<U>, U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy> Module for ResNet<T, U> {
    fn forward(&self, input: &Tensor) -> Tensor {
        let mut output = (self.net)(input);
        output = output.flatten(1, 3);
        output = (self.fc)(&output);
        output
    }
}

impl<T: Block<U> + 'static, U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy> ResNet<T, U> {
    fn new(config: ResNetConfig<T, U>) -> ResNet<T, U> {
        let mut config = config;
        let mut net = Sequential::default();
        net.push(
            Conv2dBuilder::default()
                .kernel_size([7, 7])
                .in_channel(3)
                .out_channel(64)
                .stride([2, 2])
                .padding([3, 3])
                .bias(false)
                .build(),
        );
        net.push((config.norm_layer)(64));
        net.push(Mod::new(ReLU));
        net.push(
            MaxPooling2DBuilder::default()
                .kernel_size([3, 3])
                .stride([2, 2])
                .padding([1, 1])
                .build(),
        );
        net.push(make_layer(config.norm_layer, &mut config, 64, [1, 1], 0));
        net.push(make_layer(config.norm_layer, &mut config, 128, [2, 2], 1));
        net.push(make_layer(config.norm_layer, &mut config, 256, [2, 2], 2));
        net.push(make_layer(config.norm_layer, &mut config, 512, [2, 2], 3));
        net.push(
            AdaptiveAveragePooling2DBuilder::default()
                .output_size([1, 1])
                .build(),
        );
        let fc = seq!(LinearBuilder::default()
            .input_dim(T::expansion() * 512)
            .output_dim(config.num_classes)
            .build());
        ResNet {
            base_width: config.base_width,
            num_classes: config.num_classes,
            layers: config.layers,
            net: Mod::new(net),
            fc,
            replace_stride_with_dilation: config.replace_stride_with_dilation,
            groups: config.groups,
            norm_layer: config.norm_layer,
            _phantom: PhantomData::<T>,
            dilation: config.dilation,
            inplanes: config.inplanes,
        }
    }
}

fn make_layer<T: Block<U> + 'static, U: Fn(i64) -> Mod<Sequential> + Send + Debug + Copy>(
    normlayer: U,
    config: &mut ResNetConfig<T, U>,
    planes: i64,
    mut stride: [i64; 2],
    id: i64,
) -> Mod<Sequential> {
    let mut dilate = false;
    if id > 0 {
        dilate = config.replace_stride_with_dilation[(id - 1) as usize];
    }
    let block_num = config.layers[id as usize];
    let previous_dilation = config.dilation;
    if dilate {
        config.dilation[0] *= stride[0];
        config.dilation[1] *= stride[1];
        stride[0] = 1;
        stride[1] = 1;
    }
    let temp_inplanes = config.inplanes;
    let downsample = || {
        if stride != [1, 1] || temp_inplanes != planes * T::expansion() {
            Some(seq!(
                conv1x1(temp_inplanes, planes * T::expansion(), stride),
                normlayer(planes * T::expansion()),
            ))
        } else {
            None
        }
    };
    let mut layers = Sequential::default();
    layers.push(T::new_block(
        config.inplanes,
        planes,
        stride,
        config.groups,
        config.base_width,
        previous_dilation,
        downsample(),
        normlayer,
    ));
    config.inplanes = planes * T::expansion();
    for _ in 1..=block_num - 1 {
        layers.push(T::new_block(
            config.inplanes,
            planes,
            [1, 1],
            config.groups,
            config.base_width,
            config.dilation,
            None,
            normlayer,
        ));
    }
    Mod::new(layers)
}

/// ResNet18 model from "Deep Residual Learning for Image Recognition" <https://arxiv.org/pdf/1512.03385.pdf>
pub fn resnet18(num_classes: i64) -> Mod<ResNet<BasicBlock, fn(i64) -> Mod<Sequential>>> {
    ResNetBuilder::<BasicBlock, fn(i64) -> Mod<Sequential>>::default()
        .layers([2, 2, 2, 2])
        .num_classes(num_classes)
        .build()
}

/// ResNet34 model from "Deep Residual Learning for Image Recognition" <https://arxiv.org/pdf/1512.03385.pdf>
pub fn resnet34(num_classes: i64) -> Mod<ResNet<BasicBlock, fn(i64) -> Mod<Sequential>>> {
    ResNetBuilder::<BasicBlock, fn(i64) -> Mod<Sequential>>::default()
        .layers([3, 4, 6, 3])
        .num_classes(num_classes)
        .build()
}

/// ResNet50 model from "Deep Residual Learning for Image Recognition" <https://arxiv.org/pdf/1512.03385.pdf>
pub fn resnet50(num_classes: i64) -> Mod<ResNet<BottleNeck, fn(i64) -> Mod<Sequential>>> {
    ResNetBuilder::<BottleNeck, fn(i64) -> Mod<Sequential>>::default()
        .layers([3, 4, 6, 3])
        .num_classes(num_classes)
        .build()
}

/// ResNet101 model from "Deep Residual Learning for Image Recognition" <https://arxiv.org/pdf/1512.03385.pdf>
pub fn resnet101(num_classes: i64) -> Mod<ResNet<BottleNeck, fn(i64) -> Mod<Sequential>>> {
    ResNetBuilder::<BottleNeck, fn(i64) -> Mod<Sequential>>::default()
        .layers([3, 4, 23, 3])
        .num_classes(num_classes)
        .build()
}

/// ResNet152 model from "Deep Residual Learning for Image Recognition" <https://arxiv.org/pdf/1512.03385.pdf>
pub fn resnet152(num_classes: i64) -> Mod<ResNet<BottleNeck, fn(i64) -> Mod<Sequential>>> {
    ResNetBuilder::<BottleNeck, fn(i64) -> Mod<Sequential>>::default()
        .layers([3, 8, 36, 3])
        .num_classes(num_classes)
        .build()
}
