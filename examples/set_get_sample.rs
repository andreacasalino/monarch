use std::time::Duration;

fn main() {    
    use monarch::cell::Cell;

    let mut monarch: Cell<String> = Cell::new();
    
    let mut vassal = monarch.make_reader();
    let j = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        let current_value: &String  = vassal.get().unwrap();
        // use current_value ...
    });

    monarch.set("Some important message".to_owned());

    j.join().unwrap();
}
