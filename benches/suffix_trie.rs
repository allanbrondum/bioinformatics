//! Benchmarks of insert which is the operation where we add custom logic

mod bench_util;

use bumpalo::Bump;
use criterion::{Bencher, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::mem;

use crate::bench_util::Char;
use bioinformatics::string;
use bioinformatics::string::{border_array, bwt, lcs, suffix_trie_compact, suffix_trie_mcc_arena, suffix_trie_mcc_petgraph, suffix_trie_mcc_rc, suffix_trie_ukn};
use bioinformatics::string_model::{AStr, arb_astring};
use bioinformatics::util::print_histogram;
use proptest::strategy::{Strategy, ValueTree};

const STRING_LENGTHS: &[usize] = &[200, 5000, 1_000_000];

fn bench_build_trie_compact(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| suffix_trie_compact::build_trie(s));
}

fn bench_build_trie_mcc_rc(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| suffix_trie_mcc_rc::build_trie(s));
}

fn bench_build_trie_mcc_rc_bumpalo(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| {
        let alloc = Bump::new();
        let trie = suffix_trie_mcc_rc::build_trie_with_allocator(s, &alloc);
        mem::forget(trie);
        alloc
    });
}

fn bench_build_trie_mcc_arena(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| {
        let alloc = Bump::new();
        let _trie = suffix_trie_mcc_arena::build_trie_with_allocator(s, &alloc);
        alloc
    });
}

fn bench_build_trie_mcc_petgraph(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| suffix_trie_mcc_petgraph::build_trie(s));
}

fn bench_build_trie_ukn(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter_with_large_drop(|| {
        let alloc = Bump::new();
        let _trie = suffix_trie_ukn::build_trie_with_allocator(s, &alloc);
        alloc
    });
}

// fn bench_build_and_drop_trie_compact(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
//     bencher.iter(|| suffix_trie_compact::build_trie(s));
// }
//
// fn bench_build_and_drop_trie_suffix_links(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
//     bencher.iter(|| suffix_trie_mcc_rc::build_trie(s));
// }
//
// fn bench_build_and_drop_trie_suffix_links_bumpalo(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
//     bencher.iter(|| {
//         let alloc = Bump::new();
//         suffix_trie_mcc_rc::build_trie_with_allocator(s, &alloc);
//     });
// }

fn bench_lcs_simple(bencher: &mut Bencher<'_>, s: &AStr<Char>, t: &AStr<Char>) {
    bencher.iter(|| string::lcs_simple(s, t));
}

fn bench_lcs_single_trie(bencher: &mut Bencher<'_>, s: &AStr<Char>, t: &AStr<Char>) {
    bencher.iter(|| lcs::lcs_single_trie(s, t));
}

fn bench_lcs_joined_trie(bencher: &mut Bencher<'_>, s: &AStr<Char>, t: &AStr<Char>) {
    bencher.iter(|| lcs::lcs_joined_trie(s, t));
}

fn bench_substr_simple(bencher: &mut Bencher<'_>, s: &AStr<Char>, t: &AStr<Char>) {
    bencher.iter(|| string::indexes(s, t))
}

fn bench_substr_suffix_trie(
    bencher: &mut Bencher<'_>,
    trie: &suffix_trie_mcc_arena::SuffixTrie<Char>,
    t: &AStr<Char>,
) {
    bencher.iter(|| trie.indexes_substr(t));
}

fn bench_substr_bwt(bencher: &mut Bencher<'_>, bwt: &bwt::BWT<Char>, t: &AStr<Char>) {
    bencher.iter(|| bwt.indexes_substr(t));
}

fn bench_border_array_simple(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter(|| {
        border_array::border_array_simple(s);
    });
}

fn bench_border_array(bencher: &mut Bencher<'_>, s: &AStr<Char>) {
    bencher.iter(|| {
        border_array::border_array(s);
    });
}


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
                BenchmarkId::new("build_trie_mcc_rc", string_length),
                &s,
                |bencher, s| bench_build_trie_mcc_rc(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_mcc_rc_bumpalo", string_length),
                &s,
                |bencher, s| bench_build_trie_mcc_rc_bumpalo(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_mcc_arena", string_length),
                &s,
                |bencher, s| bench_build_trie_mcc_arena(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_mcc_petgraph", string_length),
                &s,
                |bencher, s| bench_build_trie_mcc_petgraph(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
        build_trie_benches
            .bench_with_input(
                BenchmarkId::new("build_trie_ukn", string_length),
                &s,
                |bencher, s| bench_build_trie_ukn(bencher, s),
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
}

fn lcs_benches(criterion: &mut Criterion) {
    let mut lcs_benches = criterion.benchmark_group("lcs");
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
            lcs_benches
                .bench_with_input(
                    BenchmarkId::new("lcs_simple", string_length),
                    &(s.as_str(), t.as_str()),
                    |bencher, (s, t)| bench_lcs_simple(bencher, s, t),
                )
                .throughput(Throughput::Elements(string_length as u64));
        }
        lcs_benches
            .bench_with_input(
                BenchmarkId::new("lcs_single_trie", string_length),
                &(s.as_str(), t.as_str()),
                |bencher, (s, t)| bench_lcs_single_trie(bencher, s, t),
            )
            .throughput(Throughput::Elements(string_length as u64));
        lcs_benches
            .bench_with_input(
                BenchmarkId::new("lcs_joined_trie", string_length),
                &(s.as_str(), t.as_str()),
                |bencher, (s, t)| bench_lcs_joined_trie(bencher, s, t),
            )
            .throughput(Throughput::Elements(string_length as u64));
    }
    lcs_benches.finish();
}

fn substr_benches(criterion: &mut Criterion) {
    let mut substr_benches = criterion.benchmark_group("substr");
    for &string_length in STRING_LENGTHS {
        let mut runner = proptest::test_runner::TestRunner::default();
        let s = arb_astring::<Char>(string_length)
            .new_tree(&mut runner)
            .unwrap()
            .current();
        let t = arb_astring::<Char>(10)
            .new_tree(&mut runner)
            .unwrap()
            .current();

        if string_length < 10_000 {
            substr_benches
                .bench_with_input(
                    BenchmarkId::new("substr_simple", string_length),
                    &(s.as_str(), t.as_str()),
                    |bencher, (s, t)| bench_substr_simple(bencher, s, t),
                )
                .throughput(Throughput::Elements(string_length as u64));
        }
        let bump = Bump::new();
        let trie = suffix_trie_mcc_arena::build_trie_with_allocator(&s, &bump);
        substr_benches
            .bench_with_input(
                BenchmarkId::new("substr_suffix_trie", string_length),
                &(s.as_str(), t.as_str()),
                |bencher, (s, t)| bench_substr_suffix_trie(bencher, &trie, t),
            )
            .throughput(Throughput::Elements(string_length as u64));
        let bwt = bwt::build_bwt(&s);
        substr_benches
            .bench_with_input(
                BenchmarkId::new("substr_bwt", string_length),
                &(s.as_str(), t.as_str()),
                |bencher, (s, t)| bench_substr_bwt(bencher, &bwt, t),
            )
            .throughput(Throughput::Elements(string_length as u64));
        print_histogram("bwt suffix offset", &bwt.suffix_offset_hist.borrow());
    }
    substr_benches.finish();
}

fn border_array_benches(criterion: &mut Criterion) {
    let mut substr_benches = criterion.benchmark_group("substr");
    for &string_length in STRING_LENGTHS {
        let mut runner = proptest::test_runner::TestRunner::default();
        let s = arb_astring::<Char>(string_length)
            .new_tree(&mut runner)
            .unwrap()
            .current();

        substr_benches
            .bench_with_input(
                BenchmarkId::new("border_array_simple", string_length),
                s.as_str(),
                |bencher, s| bench_border_array_simple(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
        substr_benches
            .bench_with_input(
                BenchmarkId::new("border_array", string_length),
                s.as_str(),
                |bencher, s| bench_border_array(bencher, s),
            )
            .throughput(Throughput::Elements(string_length as u64));
    }
    substr_benches.finish();
}

criterion_group!(trie_benches_group, build_trie_benches);
criterion_group!(lcs_benches_group, lcs_benches);
criterion_group!(substr_benches_group, substr_benches);
criterion_group!(border_array_benches_group, border_array_benches);
criterion_main!(
    trie_benches_group,
    lcs_benches_group,
    substr_benches_group,
    border_array_benches_group
);
