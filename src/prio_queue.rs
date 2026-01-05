use std::{cmp::Ordering, collections::BinaryHeap, ffi::c_void, sync::Arc};

#[allow(dead_code)]
struct Entry<T> {
    ctr: usize,
    data: T,
    compare: Arc<dyn Fn(&T, &T) -> Ordering>,
}

#[allow(dead_code)]
pub struct PriorityQueue<'a, T, D> {
    compare: Arc<dyn Fn(&Entry<T>, &Entry<T>) -> Ordering + 'a>,
    insertion_ctr: usize,
    cb_data: &'a D,
    array: BinaryHeap<Entry<T>>,
}

#[allow(dead_code)]
impl<'a, T, D> PriorityQueue<'a, T, D> {
    fn set_comparator(&mut self, compare: Option<&'a dyn Fn(&T, &T, &D) -> i32>, cb_data: &'a D) {
        self.cb_data = cb_data;

        self.compare = Arc::new(move |one: &Entry<T>, two: &Entry<T>| {
            let default = one.ctr.cmp(&two.ctr);
            let Some(ref compare) = compare else {
                return default;
            };
            match compare(&one.data, &two.data, &cb_data) {
                i if i < 0 => Ordering::Less,
                i if i > 0 => Ordering::Greater,
                _ => default,
            }
        });
    }
    fn put(&mut self, _thing: T) {
        //
    }

    fn get(&mut self) -> Option<T> {
        todo!();
    }
    fn peek(&self) -> Option<&T> {
        todo!();
    }
    fn replace(&mut self, _thing: T) {
        todo!();
    }
    fn reverse(&mut self) {
        //
    }
}

#[repr(C)]
pub struct PriorityQueueHandle {
    pub(crate) inner: *mut PriorityQueue<'static, *mut c_void, *mut c_void>,
}

#[no_mangle]
pub unsafe extern "C" fn prio_queue_put(queue: *mut PriorityQueueHandle, thing: *mut c_void) {
    let handle = match queue.as_mut() {
        Some(h) => h,
        None => return,
    };
    let queue = match handle.inner.as_mut() {
        Some(q) => q,
        None => return,
    };
    queue.put(thing)
}
