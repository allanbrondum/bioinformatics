//! Benchmarks of insert which is the operation where we add custom logic

mod bench_util;

use criterion::{
    criterion_group, criterion_main, Bencher, BenchmarkId, Criterion, Throughput,
};
use proptest::arbitrary::any;
use proptest::collection::vec;


use crate::bench_util::Char;
use proptest::strategy::{Strategy, ValueTree};
use rosalind::string::{suffix_trie_simple, suffix_trie_suffix_links};
use rosalind::string_model::{AStr, AString};

const STRING_LENGTHS: &[usize] = &[32, 256, 1_024];

fn bench_build_trie_simple(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| {
        suffix_trie_simple::build_trie(s)
    });
}

fn bench_build_trie_suffix_links(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| {
        suffix_trie_suffix_links::build_trie(s)
    });
}

fn bench_build_and_drop_trie_simple(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter(|| {
        suffix_trie_simple::build_trie(s)
    });
}

fn bench_build_and_drop_trie_suffix_links(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter(|| {
        suffix_trie_suffix_links::build_trie(s)
    });
}

fn bench_indexes_substr(bencher: &mut Bencher<'_>, trie: suffix_trie_simple::SuffixTrie<Char>) {
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
        let s = AString::from(value.current());

        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_simple", string_length),
                &s,
                |bencher, s| bench_build_trie_simple(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));

        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_and_drop_trie_simple", string_length),
                &s,
                |bencher, s| bench_build_and_drop_trie_simple(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));

        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_suffix_links", string_length),
                &s,
                |bencher, s| bench_build_trie_suffix_links(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));

        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_and_drop_trie_suffix_links", string_length),
                &s,
                |bencher, s| bench_build_and_drop_trie_suffix_links(bencher, s),
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
