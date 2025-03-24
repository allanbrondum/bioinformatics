//! Benchmarks of insert which is the operation where we add custom logic

mod bench_util;

use bumpalo::Bump;
use criterion::{Bencher, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::mem;

use crate::bench_util::Char;
use bioinformatics::string;
use bioinformatics::string::{lcs, suffix_trie_compact, suffix_trie_suffix_links, suffix_trie_suffix_links_arena_refs, suffix_trie_ukkonen};
use bioinformatics::string_model::{AStr, arb_astring};
use proptest::strategy::{Strategy, ValueTree};

const STRING_LENGTHS: &[usize] = &[200, 5000, 1_000_000];

fn bench_build_trie_compact(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| suffix_trie_compact::build_trie(s));
}

fn bench_build_trie_suffix_links(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| suffix_trie_suffix_links::build_trie(s));
}

fn bench_build_trie_suffix_links_bumpalo(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| {
        let alloc = Bump::new();
        let trie = suffix_trie_suffix_links::build_trie_with_allocator(s, &alloc);
        mem::forget(trie);
        alloc
    });
}

fn bench_build_trie_suffix_links_arena_refs(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| {
        let alloc = Bump::new();
        let _trie = suffix_trie_suffix_links_arena_refs::build_trie_with_allocator(s, &alloc);
        alloc
    });
}

fn bench_build_trie_ukkonen(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| {
        let alloc = Bump::new();
        let _trie = suffix_trie_ukkonen::build_trie_with_allocator(s, &alloc);
        alloc
    });
}

fn bench_build_and_drop_trie_compact(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter(|| suffix_trie_compact::build_trie(s));
}

fn bench_build_and_drop_trie_suffix_links(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter(|| suffix_trie_suffix_links::build_trie(s));
}

fn bench_build_and_drop_trie_suffix_links_bumpalo(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter(|| {
        let alloc = Bump::new();
        suffix_trie_suffix_links::build_trie_with_allocator(s, &alloc);
    });
}

fn bench_lcs_simple(bencher: &mut Bencher<'_>, s: &AStr<Char>, t: &AStr<Char>) {
    bencher.iter(|| string::lcs_simple(s, t));
}

fn bench_lcs_single_trie(bencher: &mut Bencher<'_>, s: &AStr<Char>, t: &AStr<Char>) {
    bencher.iter(|| lcs::lcs_single_trie(s, t));
}

fn bench_lcs_joined_trie(bencher: &mut Bencher<'_>, s: &AStr<Char>, t: &AStr<Char>) {
    bencher.iter(|| lcs::lcs_joined_trie(s, t));
}

// fn bench_indexes_substr(bencher: &mut Bencher<'_>, trie: suffix_trie_simple::SuffixTrie<Char>) {
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
// }

fn build_trie_benches(criterion: &mut Criterion) {
    let mut build_trie_benches = criterion.benchmark_group("build_trie");
    for &string_length in STRING_LENGTHS {
        let mut runner = proptest::test_runner::TestRunner::default();
        let s = arb_astring::<Char>(string_length)
            .new_tree(&mut runner)
            .unwrap()
            .current();

        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_compact", string_length),
                &s,
                |bencher, s| bench_build_trie_compact(bencher, s),
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
                BenchmarkId::new("build_trie_suffix_links_bumpalo", string_length),
                &s,
                |bencher, s| bench_build_trie_suffix_links_bumpalo(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_suffix_links_arena_refs", string_length),
                &s,
                |bencher, s| bench_build_trie_suffix_links_arena_refs(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_ukkonen", string_length),
                &s,
                |bencher, s| bench_build_trie_ukkonen(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
        // build_trie_benches
        //     .bench_with_input(
        //         BenchmarkId::new("build_and_drop_trie_compact", string_length),
        //         &s,
        //         |bencher, s| bench_build_and_drop_trie_compact(bencher, s),
        //     )
        //     .throughput(Throughput::Elements(string_length as u64));
        // build_trie_benches
        //     .bench_with_input(
        //         BenchmarkId::new("build_and_drop_trie_suffix_links", string_length),
        //         &s,
        //         |bencher, s| bench_build_and_drop_trie_suffix_links(bencher, s),
        //     )
        //     .throughput(Throughput::Elements(string_length as u64));
        // build_trie_benches
        //     .bench_with_input(
        //         BenchmarkId::new("build_and_drop_trie_suffix_links_bumpalo", string_length),
        //         &s,
        //         |bencher, s| bench_build_and_drop_trie_suffix_links_bumpalo(bencher, s),
        //     )
        //     .throughput(Throughput::Elements(string_length as u64));
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

fn lcs_benches(criterion: &mut Criterion) {
    let mut build_trie_benches = criterion.benchmark_group("lcs");
    for &string_length in STRING_LENGTHS {
        let mut runner = proptest::test_runner::TestRunner::default();
        let s = arb_astring::<Char>(string_length)
            .new_tree(&mut runner)
            .unwrap()
            .current();
        let t = arb_astring::<Char>(string_length)
            .new_tree(&mut runner)
            .unwrap()
            .current();

        if string_length < 10_000 {
            build_trie_benches
                .bench_with_input(
                    BenchmarkId::new("lcs_simple", string_length),
                    &(s.as_str(), t.as_str()),
                    |bencher, (s, t)| bench_lcs_simple(bencher, s, t),
                )
                .throughput(Throughput::Elements(string_length as u64));
        }
        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("lcs_single_trie", string_length),
                &(s.as_str(), t.as_str()),
                |bencher, (s, t)| bench_lcs_single_trie(bencher, s, t),
            )
            .throughput(Throughput::Elements(string_length as u64));
        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("lcs_joined_trie", string_length),
                &(s.as_str(), t.as_str()),
                |bencher, (s, t)| bench_lcs_joined_trie(bencher, s, t),
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

criterion_group!(trie_benches_group, build_trie_benches);
criterion_group!(lcs_benches_group, lcs_benches);
criterion_main!(trie_benches_group, lcs_benches_group);
