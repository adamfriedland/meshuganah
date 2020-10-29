use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use mongodb::Cursor;

use crate::repository::GenericRepository;
use crate::repository::RepositoryTrait;

use super::model::Model;

pub struct TypeCursor<T> {
    cursor: Cursor,
    phantom: PhantomData<T>,
}

impl<T: Model> TypeCursor<T> {
    pub fn new(cursor: Cursor) -> Self {
        TypeCursor {
            cursor,
            phantom: PhantomData,
        }
    }
}

impl<T> Unpin for TypeCursor<T> {}

impl<T: Model> Stream for TypeCursor<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item = match Pin::new(&mut self.cursor).poll_next(cx) {
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(Ok(doc))) => doc,
            Poll::Ready(Some(Err(_))) => panic!(""),
            Poll::Pending => return Poll::Pending,
        };

        match GenericRepository::<T>::get_instance_from_document(item) {
            Ok(doc) => Poll::Ready(Some(doc)),
            Err(_) => panic!("failed to convert"),
        }
    }
}
