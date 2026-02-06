use futures::{Stream, StreamExt};
use monarch::cell::CellReader;

async fn foo(mut reader: CellReader<String>) {
    // some code
    let val = reader.next_value().await;
    // use val ...
}

async fn bla(reader: CellReader<String>) {
    use std::pin::Pin;

    let stream: Pin<Box<dyn Stream<Item = String> + Send>> = reader.into_update_stream();

    stream.for_each(async |value| {
        // use value ...
    }).await;
}

fn main() {
}
