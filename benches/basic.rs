use std::alloc::{GlobalAlloc, System};

use criterion::{criterion_group, criterion_main, Criterion};
use rc_vec::{ArcVec, RcVec};

//#[global_allocator]
//static NON_REALLOC: _NonRealloc = _NonRealloc;

struct _NonRealloc;
unsafe impl GlobalAlloc for _NonRealloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        System.dealloc(ptr, layout);
    }
}

fn basic(c: &mut Criterion) {
    let n: usize = 4000000;
    c.bench_function("Vec", |b| b.iter(|| {
        let mut vec = Vec::new();

        for i in 0..n {
            vec.push(i);
        }
    }));
    c.bench_function("RcVec", |b| b.iter(|| {
        let mut vec = RcVec::new();

        for i in 0..n {
            vec.push(i);
        }
    }));
    c.bench_function("ArcVec", |b| b.iter(|| {
        let mut vec = ArcVec::new();

        for i in 0..n {
            vec.push(i);
        }
    }));
}

criterion_group!(benches, basic);
criterion_main!(benches);
