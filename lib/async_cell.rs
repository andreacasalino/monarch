use crate::cell::CellReader;

use futures::Future;

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

#[cfg(test)]
mod tests {
    use std::time::Duration;

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
            async {
                reader.next_value().await
            }
        );

        assert_eq!(value, value_back);
    }
}
