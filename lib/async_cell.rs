use crate::cell::CellReader;

use futures::{Future, Stream, StreamExt};
use std::pin::Pin;

pub struct CellReaderNextValue<'a, T: Clone> {
    reader: &'a mut CellReader<T>
}

impl<'a, T: Clone> Future for CellReaderNextValue<'a, T> {
    type Output = T;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.reader.was_remote_updated() {
            if let Some(next_val_ref) = self.reader.get() {
                std::task::Poll::Ready(next_val_ref.clone())
            }
            else {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
        }
        else {
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }
}

impl<T: Clone> CellReader<T> {
    pub fn next_value(&'_ mut self) -> CellReaderNextValue<'_, T> {
        CellReaderNextValue{reader: self}
    }
}

impl<T: 'static + Clone + Send> CellReader<T> {
    // TODO clarify some intermediate update could be missing ...
    pub fn into_update_stream(mut self) -> Pin<Box<dyn Stream<Item = T> + Send>> {
        async_stream::stream! {
            loop {
                yield self.next_value().await;
            }
        }.boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::StreamExt;

    use crate::cell::Cell;

    #[tokio::test]
    async fn await_next_val_test() {
        let mut writer: Cell<String> = Cell::new();
        let mut reader = writer.make_reader();

        let value = "some value".to_owned();
        let value_clone = value.clone();

        let (_, value_back) = tokio::join!(
            async {
                tokio::time::sleep(Duration::from_millis(500)).await;
                writer.set(value_clone);
            },
            reader.next_value()
        );

        assert_eq!(value, value_back);
    }

    #[tokio::test]
    async fn update_stream_test() {
        let mut writer: Cell<String> = Cell::new();
        let mut reader = writer.make_reader().into_update_stream();

        let values_expected: Vec<String> = (0..50).map(|index| format!("value-{}", index)).collect(); 
        let values_expected_clone = values_expected.clone();
        let len = values_expected.len();

        let (_, values_back) = tokio::join!(
            async {
                for value in values_expected_clone {
                    writer.set(value);
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            },
            async {
                let mut res: Vec<String> = Vec::new();
                while res.len() < len {
                    res.push(reader.next().await.unwrap());
                }
                res
            }
        );

        assert_eq!(values_expected, values_back);
    }
}
