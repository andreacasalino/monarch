This package offers a single writer multiple readers cell implementation.
With this package, you can define a region of the memory, a **Cell**, to which only one thread can write, while at the same time, multiple consumers, **CellReader**, can safely read from the same region.
The name, **monarch**, is inspired by the fact that there can be only one **Cell**, i.e. one monarch dictating the current value in the memory reagion, while at the same time there can be multiple vassala, i.e. the **CellReader**s that can access it.

Each time the **monarch** write something into the **Cell**, a version number is internally bumped. On the other side, any **CellReader** will keep a local copy of the same value that can be consumed in the thread where the reader lives and will monitor the version number to detect when is time to update the local copy.

Clearly, it does not make any sense to use a **Cell** to guard a value that can be embedded into an **Atomic**. **Cell** is meant for those type that don't fit **Atomic**, but are the same type cheap enough to be copied.

#EXAMPLES

Create the cell and one (or multiple) readers:
```rust
use monarch::cell::Cell;

let mut monarch: Cell<String> = Cell::new();

let mut vassal = monarch.make_reader();
let j = std::thread::spawn(move || {
    std::thread::sleep(Duration::from_millis(500));
    let current_value: &String  = vassal.get().unwrap();
    // TODO use the current_value
});

monarch.set("Some important message".to_owned());
```

An asynchronous extension is available too, allowing any consumer to await for the next change of the **Cell**.
```rust
use monarch::cell::CellReader;

async fn foo(mut reader: CellReader<String>) {
    // some code
    let val = reader.next_value().await;
    // use val ...
}
```

... or actually keep listening for updates and push them into a stream:
```rust
use std::pin::Pin;

let reader: CellReader<String> = ...;

async {
    let stream: Pin<Box<dyn Stream<Item = String> + Send>> = reader.into_update_stream();

    stream.for_each(async |value| {
        // use value ...
    }).await;
}
```

Attention!!! There is no guarantee that ALL updates will be catched by the above stream. Indeed, it could happen that the write will update 2 or more times the **Cell** while the last value pushed in the stream is being processed. This shall not be a problem as the implementation in this crate is meant to allow the user to get the latest value of the cell and not ALL intermediate ones!
