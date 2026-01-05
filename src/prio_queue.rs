use std::{cmp::Ordering, collections::VecDeque, ffi::c_void};

#[allow(dead_code)]
struct PriorityQueueEntry<T> {
    ctr: usize,
    data: T,
}

#[allow(dead_code)]
pub struct PriorityQueue<T, D> {
    compare: Option<Box<dyn Fn(&T, &T, &D) -> i32>>,
    insertion_ctr: usize,
    cb_data: D,
    array: VecDeque<PriorityQueueEntry<T>>,
}

#[allow(dead_code)]
impl<T, D> PriorityQueue<T, D> {
    fn put(&mut self, thing: T) {
        let entry = PriorityQueueEntry {
            ctr: self.insertion_ctr,
            data: thing,
        };
        self.array.push_back(entry);
        self.insertion_ctr += 1;

        let Some(ref compare) = self.compare else {
            return;
        };

        let cb_data = &self.cb_data;

        self.array.make_contiguous().sort_by(|one, two| {
            match compare(&one.data, &two.data, cb_data) {
                i if i < 0 => Ordering::Less,
                i if i > 0 => Ordering::Greater,
                _ => Ordering::Equal,
            }
        });

        // let mut ix = self.array.len() - 1;
        // while ix != 0 {
        //     let parent = (ix - 1) / 2;
        //     if compare(
        //         &self.array[parent].data,
        //         &self.array[ix].data,
        //         &self.cb_data,
        //     ) > 0
        //     {
        //         break;
        //     }

        //     self.array.swap(parent, ix);

        //     ix = parent;
        // }
    }

    fn get(&mut self) -> Option<T> {
        // Might be able to get better performance out of this by storing the
        // 'first out' entry at the end of a vec, not the start. Will require
        // tests to be in place before that refactoring.
        let result = self.array.pop_front().map(|entry| entry.data);

        result
    }
    fn peek(&self) -> Option<&T> {
        todo!();
    }
    fn replace(&mut self, _thing: T) {
        todo!();
    }
    fn reverse(&mut self) {
        self.array.make_contiguous().reverse();
    }
}

#[repr(C)]
pub struct PriorityQueueHandle {
    pub(crate) inner: *mut PriorityQueue<*mut c_void, *mut c_void>,
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
