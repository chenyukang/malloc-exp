extern crate jemallocator;
extern crate jemalloc_sys;
extern crate jemalloc_ctl;

use std::thread;
use std::mem;


use std::time::Duration;
//use jemalloc_ctl::{stats, epoch};
use jemallocator::Jemalloc;

use std::alloc::{GlobalAlloc, Layout};


use std::rc::Rc;
use std::cell::RefCell;
use crate::List::{Cons, Nil};

#[derive(Debug)]
enum List {
    Cons(i32, RefCell<Rc<List>>),
    Nil,
}

impl List {
    fn tail(&self) -> Option<&RefCell<Rc<List>>> {
        match self {
            Cons(_, item) => Some(item),
            Nil => None,
        }
    }
}

#[global_allocator]
static ALLOC: Jemalloc = jemallocator::Jemalloc;

fn smoke_one() {
    let layout = Layout::from_size_align(1000, 8).unwrap();
    unsafe {
        let ptr = Jemalloc.alloc(layout.clone());
        assert!(!ptr.is_null());
        //Jemalloc.dealloc(ptr, layout);
    }
}

fn smoke_two() {
    unsafe {
        let ptr = jemalloc_sys::malloc(4);
        *(ptr as *mut u32) = 0xDECADE;
        assert_eq!(*(ptr as *mut u32), 0xDECADE);
        jemalloc_sys::free(ptr);
        *(ptr as *mut u32) = 0xDECADE;
    }
}

fn smoke_three() {
    let s = String::from("hello");
    let ptr = s.as_ptr();
    let handler = ptr as u64;
    let len = s.len();
    let capacity = s.capacity();
    mem::forget(s);

    let s_new = unsafe { String::from_raw_parts(ptr as *mut _, len, capacity) };
    let s_new2 = unsafe { String::from_raw_parts(ptr as *mut _, len, capacity) };
}

fn main() {
    println!("Hello, world!");
    let a = Rc::new(Cons(5, RefCell::new(Rc::new(Nil))));

    println!("a initial rc count = {}", Rc::strong_count(&a));
    println!("a next item = {:?}", a.tail());

    let b = Rc::new(Cons(10, RefCell::new(Rc::clone(&a))));

    println!("a rc count after b creation = {}", Rc::strong_count(&a));
    println!("b initial rc count = {}", Rc::strong_count(&b));
    println!("b next item = {:?}", b.tail());

    if let Some(link) = a.tail() {
        *link.borrow_mut() = Rc::clone(&b);
    }

    println!("b rc count after changing a = {}", Rc::strong_count(&b));
    println!("a rc count after changing a = {}", Rc::strong_count(&a));



    for _i in 1..5 {
        // many statistics are cached and only updated when the epoch is advanced.
        jemalloc_ctl::epoch().unwrap();
        smoke_one();
        smoke_two();
        smoke_three();
        let allocated = jemalloc_ctl::stats::allocated().unwrap();
        let resident = jemalloc_ctl::stats::resident().unwrap();
        println!("{} bytes allocated/{} bytes resident", allocated, resident);
        thread::sleep(Duration::from_secs(1));
    }
}
