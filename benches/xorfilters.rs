use criterion::{Criterion, criterion_group, criterion_main};
use std::{fs::File, hint::black_box, io::BufReader};
use xorf::{BinaryFuse8, BinaryFuse16, Filter};

fn fuse16(b: &BinaryFuse16, num: u64) -> bool {
    b.contains(&num)
}

fn fuse8(b: &BinaryFuse8, num: u64) -> bool {
    b.contains(&num)
}

fn criterion_benchmark(c: &mut Criterion) {
    let reader = BufReader::new(File::open("./data/xorfilter8").unwrap());
    let filter: BinaryFuse8 =
        bincode::decode_from_reader(reader, bincode::config::standard()).unwrap();
    c.bench_function("fuse8", |b| b.iter(|| fuse8(&filter, black_box(52))));
    drop(filter);

    let reader = BufReader::new(File::open("./data/xorfilter16").unwrap());
    let filter: BinaryFuse16 =
        bincode::decode_from_reader(reader, bincode::config::standard()).unwrap();
    c.bench_function("fuse16", |b| b.iter(|| fuse16(&filter, black_box(52))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
