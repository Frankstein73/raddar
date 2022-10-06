use std::{cmp::min, sync::Arc};

#[derive(Debug)]
pub struct SimpleDataset<T> {
    pub inputs: Vec<Arc<T>>,
    pub labels: Vec<Arc<T>>,
    pub size: usize,
    pub batch_size: usize,
}

pub trait Dataset
where
    Self: Sized,
{
    type DataType;
    type BatchType;
    fn iter(&self) -> DatasetIterator<Self>;
    fn get_inputs(&self) -> &Vec<Arc<Self::DataType>>;
    fn get_labels(&self) -> &Vec<Arc<Self::DataType>>;
    fn get_size(&self) -> usize;
    fn get_batch_size(&self) -> usize;
}

impl<T> Dataset for SimpleDataset<T> {
    type DataType = T;
    type BatchType = Vec<Arc<T>>;

    fn iter(&self) -> DatasetIterator<Self> {
        DatasetIterator {
            dataset: self,
            index: 0,
        }
    }

    fn get_inputs(&self) -> &Vec<Arc<Self::DataType>> {
        &self.inputs
    }

    fn get_labels(&self) -> &Vec<Arc<Self::DataType>> {
        &self.labels
    }

    fn get_size(&self) -> usize {
        self.size
    }

    fn get_batch_size(&self) -> usize {
        self.batch_size
    }
}

pub struct DatasetIterator<'a, T: Dataset> {
    pub dataset: &'a T,
    pub index: usize,
}


impl<'a, T> Iterator for DatasetIterator<'a, SimpleDataset<T>> {
    type Item = (
        <SimpleDataset<T> as Dataset>::BatchType,
        <SimpleDataset<T> as Dataset>::BatchType,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.dataset.get_size() {
            return None;
        }
        let end = min(
            self.index + self.dataset.get_batch_size(),
            self.dataset.get_size(),
        );
        let batch = self.dataset.get_inputs()[self.index..end].to_vec();
        let batch_labels = self.dataset.get_labels()[self.index..end].to_vec();
        self.index = end;
        Some((batch, batch_labels))
    }
}
