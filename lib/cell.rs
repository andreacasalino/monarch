use crate::spin_lock::SpinLock;

use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::cell::UnsafeCell;

pub(crate) struct CellContent<T: Default + Clone> {
    pub version: AtomicU64,
    pub lock: SpinLock,
    pub value: UnsafeCell<T>
}

pub struct Cell<T: Default + Clone> {
    content: Arc<CellContent<T>>
}

impl<T:Default + Clone> Cell<T> {
    pub fn new() -> Self {
        Self{content: Arc::new(CellContent{
            version: AtomicU64::new(1),
            lock: SpinLock::new(),
            value: UnsafeCell::new(Default::default()),
        })}
    }

    pub fn set(&mut self, value: T) {
        let _ = self.content.lock.make_guard();
        unsafe {
            let ptr = self.content.value.get().as_mut().unwrap();
            *ptr = value;
        }
        self.content.version.fetch_add(1, std::sync::atomic::Ordering::Release);
    }
}

unsafe impl<T: Default + Clone> Send for Cell<T> {}
unsafe impl<T: Default + Clone> Sync for Cell<T> {}

#[derive(Clone)]
pub struct CellReader<T: Default + Clone> {
    content: Arc<CellContent<T>>,
    local_version: u64,
    local_value: T
}

impl<T: Default + Clone> CellReader<T> {
    pub fn new(content: Arc<CellContent<T>>) -> Self {
        Self { 
            content,
            local_version: 0,
            local_value: Default::default()
        }
    }

    fn update_local(&mut self) {
        let _ = self.content.lock.make_guard();
        unsafe {
            let ptr = self.content.value.get().as_mut().unwrap();
            self.local_value.clone_from(ptr);
        }
        self.local_version = self.content.version.load(std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get(&'_ mut self)-> &'_ T {
        let current_version = self.content.version.load(std::sync::atomic::Ordering::Acquire);
        if self.local_version < current_version  {
            self.update_local();
        }
        &self.local_value
    }
}

impl<T: Default + Clone> Cell<T> {
    pub fn make_reader(&self) -> CellReader<T> {
        CellReader::new(self.content.clone())
    }
}

unsafe impl<T: Default + Clone> Send for CellReader<T> {}
unsafe impl<T: Default + Clone> Sync for CellReader<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_cell_test() {
        let mut writer: Cell<String> = Cell::new();
        let mut reader = writer.make_reader();

        let val = "some value".to_owned();
        writer.set(val.clone());
        let val_back = reader.get();
        assert_eq!(val, *val_back);

        let val = "some other value".to_owned();
        writer.set(val.clone());
        let val_back = reader.get();
        assert_eq!(val, *val_back);
    }

    use std::collections::VecDeque;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU8};
    use std::time::Duration;

    struct Values {
        values: VecDeque<String>
    }

    impl Values {
        pub fn new() -> Self {
            let mut values: VecDeque<String> = VecDeque::new();
            values.push_back("".to_owned());
            Self{values}
        }        

        pub fn push(&mut self, val: String) {
            if *self.values.back().unwrap() != val {
                self.values.push_back(val);
            }
        }

        pub fn extract(self) -> Vec<String> {
            self.values.into_iter().skip(1).collect()
        }
    }

    fn read_till_done(mut reader: CellReader<String>, started_barrier: Arc<AtomicU8>, completed_barrier: Arc<AtomicBool>) -> std::thread::JoinHandle<Vec<String>> {
        std::thread::spawn(move || {
            let mut values = Values::new();
            started_barrier.fetch_sub(1, std::sync::atomic::Ordering::Acquire);
            loop {
                values.push(reader.get().clone());
                if completed_barrier.load(std::sync::atomic::Ordering::Acquire) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            values.extract()
        })
    }

    #[test]
    fn concurrent_get_set_cell_test() {
        let mut writer: Cell<String> = Cell::new();
        let started_barrier: Arc<AtomicU8> = Arc::new(AtomicU8::new(2));
        let completed_barrier: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let j1= read_till_done(writer.make_reader(), started_barrier.clone(), completed_barrier.clone());
        let j2= read_till_done(writer.make_reader(), started_barrier.clone(), completed_barrier.clone());

        let values_expected: Vec<String> = (0..10).map(|index| format!("value-{}", index)).collect(); 
        let values_expected_clone = values_expected.clone();

        while 0 < started_barrier.load(std::sync::atomic::Ordering::Acquire) {}
        for value in values_expected_clone {                
            std::thread::sleep(Duration::from_millis(50));
            writer.set(value);
        }
        completed_barrier.store(true, std::sync::atomic::Ordering::Release);

        assert_eq!(j1.join().unwrap(), values_expected);
        assert_eq!(j2.join().unwrap(), values_expected);
    }
}
