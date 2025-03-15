//! Benchmarks of insert which is the operation where we add custom logic

mod bench_util;

use criterion::{
    criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput,
};
use proptest::arbitrary::any;
use proptest::collection::vec;
use rosalind::string::suffix_trie_simple::{build_trie, SuffixTrie};

use crate::bench_util::Char;
use proptest::strategy::{Strategy, ValueTree};

const STRING_LENGTHS: &[usize] = &[32, 256, 1_024];

fn bench_build_trie(bencher: &mut Bencher<'_>, s: &[Char]) {
    bencher.iter(|| {
        build_trie(s);
    });
}

fn bench_indexes_substr(bencher: &mut Bencher<'_>, trie: SuffixTrie<Char>) {
    // bencher.iter_batched(
    //     || map.clone(),
    //     |mut map| {
    //         // trigger expiration of all items in the map
    //         map.insert(map.len(), ());
    //         assert_eq!(map.len(), 1);
    //         map
    //     },
    //     BatchSize::SmallInput,
    // );
}

fn criterion_benches(criterion: &mut Criterion) {
    let mut build_trie_benches = criterion.benchmark_group("build_trie");
    for &string_length in STRING_LENGTHS {
        let mut runner = proptest::test_runner::TestRunner::default();
        let mut value = vec(any::<Char>(), string_length).new_tree(&mut runner).unwrap();
        let s = value.current();

        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie", string_length),
                &s,
                |bencher, s| bench_build_trie(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
    }
    build_trie_benches.finish();

    // let mut set_expire_all_benches = criterion.benchmark_group("indexes_substr");
    // for &map_size in STRING_LENGTHS {
    //     let mut map = ExpiringHashMap::with_duration(Duration::from_millis(500));
    //     for item in 0..map_size {
    //         map.insert(item, ());
    //     }
    //
    //     // wait such that all inserted items are expired
    //     thread::sleep(Duration::from_millis(600));
    //
    //     set_expire_all_benches
    //         .bench_with_input(
    //             BenchmarkId::new("map_expire_all", map_size),
    //             &map_size,
    //             |bencher, &_map_size| map_expire_all(bencher, map.clone()),
    //         )
    //         .throughput(Throughput::Elements(map_size as u64));
    // }
    // set_expire_all_benches.finish();
}

criterion_group!(benches, criterion_benches);
criterion_main!(benches);
