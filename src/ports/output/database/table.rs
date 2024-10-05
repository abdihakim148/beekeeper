use std::hash::Hash;
use crate::domain::types::Error;


type Result<T> = std::result::Result<T, Error>;

pub trait Table: Sized {
    type Item: Clone + Hash + PartialEq;
    type Id: Clone + Hash + PartialEq;

    const NAME: &'static str;

    async fn new<T>(args: T) -> Result<Self>;
    async fn create(&self, item: &Self::Item) -> Result<Self::Id>;
    async fn read(&self, id: &Self::Id) -> Option<Self::Item>;
    async fn update(&self, item: &Self::Item) -> Result<Self::Id>;
    async fn delete(&self, id: &Self::Id) -> Result<Self::Id>;
}
