use std::{cmp::Ordering, collections::BinaryHeap, ffi::c_void, sync::Arc};

type EntryComparator<'a, T> = Arc<dyn Fn(&Entry<T>, &Entry<T>) -> Ordering + 'a>;
type EntryComparatorRaw<'a, T, D> = Option<&'a dyn Fn(&T, &T, Option<&D>) -> i32>;

#[allow(dead_code)]
struct Entry<'a, T> {
    ctr: usize,
    data: T,
    compare: EntryComparator<'a, T>,
}

impl<'a, T> Ord for Entry<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.compare)(self, other)
    }
}

impl<'a, T> PartialOrd for Entry<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T> Eq for Entry<'a, T> {}

impl<'a, T> PartialEq for Entry<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

#[allow(dead_code)]
pub struct PriorityQueue<'a, T, D> {
    compare: EntryComparator<'a, T>,
    insertion_ctr: usize,
    cb_data: Option<&'a D>,
    array: BinaryHeap<Entry<'a, T>>,
}

fn default_compare<T>(one: &Entry<T>, two: &Entry<T>) -> Ordering {
    one.ctr.cmp(&two.ctr)
}

#[allow(dead_code)]
impl<'a, T: 'a, D: 'a> PriorityQueue<'a, T, D> {
    fn new() -> Self {
        PriorityQueue {
            compare: Arc::new(default_compare),
            insertion_ctr: 0,
            cb_data: None,
            array: BinaryHeap::new(),
        }
    }
    fn set_comparator(&mut self, compare: EntryComparatorRaw<'a, T, D>, cb_data: Option<&'a D>) {
        self.cb_data = cb_data;

        self.compare = Arc::new(move |one: &Entry<T>, two: &Entry<T>| {
            let default = one.ctr.cmp(&two.ctr);
            let Some(ref compare) = compare else {
                return default;
            };
            match compare(&one.data, &two.data, cb_data) {
                i if i < 0 => Ordering::Less,
                i if i > 0 => Ordering::Greater,
                _ => default,
            }
        });
    }

    fn put(&mut self, thing: T) {
        self.array.push(Entry {
            ctr: self.insertion_ctr,
            data: thing,
            compare: self.compare.clone(),
        });
        self.insertion_ctr += 1;
    }

    fn get(&mut self) -> Option<T> {
        self.array.pop().map(|entry| entry.data)
    }
    fn peek(&self) -> Option<&T> {
        self.array.peek().map(|entry| &entry.data)
    }
    fn replace(&mut self, thing: T) {
        let Some(mut top) = self.array.peek_mut() else {
            self.put(thing);
            return;
        };

        *top = Entry {
            ctr: self.insertion_ctr,
            data: thing,
            compare: self.compare.clone(),
        };
        self.insertion_ctr += 1;
    }
    fn reverse(&mut self) {
        // Construct our new comparator. Note that all the existing `Entry`
        // structs will still have an Arc to the old comparator.
        let orig = std::mem::replace(
            &mut self.compare,
            Arc::new(|_: &Entry<T>, _: &Entry<T>| Ordering::Equal),
        );
        self.compare = Arc::new(move |one: &Entry<T>, two: &Entry<T>| orig(one, two).reverse());

        // Rebuild all entries with the new comparator...
        let mut entries = Vec::with_capacity(self.array.len());
        for entry in self.array.drain().rev() {
            entries.push(Entry {
                ctr: self.insertion_ctr,
                data: entry.data,
                compare: self.compare.clone(),
            });
            self.insertion_ctr += 1;
        }

        // and throw them back in the heap.
        self.array = BinaryHeap::with_capacity(entries.len());
        self.array.extend(entries);
    }
}

#[repr(C)]
pub struct PriorityQueueCHandle {
    pub(crate) inner: *mut PriorityQueue<'static, *mut c_void, *mut c_void>,
}

#[no_mangle]
/// Put an item into the priority queue.
///
/// # Safety
///
/// The caller must ensure that `queue` is a valid pointer to a
/// `PriorityQueueCHandle` and that `thing` is a valid pointer to the item to be
/// inserted. The caller is responsible for managing the memory of `thing` and
/// for ensuring that its payload type is the queue's established payload type.
pub unsafe extern "C" fn prio_queue_put(queue: *mut PriorityQueueCHandle, thing: *mut c_void) {
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
