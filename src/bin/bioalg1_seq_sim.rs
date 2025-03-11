use rand::Rng;
use rosalind::util::fasta_polymers;
use std::fs::File;
use std::io::Write;
use std::iter;

fn main() {
    let read_len = 200;
    let target_depth = 2.5;
    let theta = 0.1;
    let input_path = "src/bin/bioalg1_seq_sim_data.txt";
    let output_path = "src/bin/bioalg1_assemble_data.txt";

    let polymer = fasta_polymers(input_path).next().unwrap().polymer;
    let mut out_file = File::create(output_path).unwrap();
    let genome_length = polymer.len();
    let num_reads = (target_depth * genome_length as f64 / read_len as f64).ceil() as usize;
    let mut rng = rand::rng();
    let mut base_read_depths = vec![0; polymer.len()];
    let mut read_start_ends = vec![StartEnd::default(); polymer.len()];
    for i in 0..num_reads {
        let start = rng.random_range(0..genome_length - read_len);
        read_start_ends[start].start += 1;
        read_start_ends[start + read_len].end += 1;
        for base in &mut base_read_depths[start..start + read_len] {
            *base += 1;
        }
        writeln!(out_file, ">{}:{}:{}", i, start + 1, read_len).unwrap();
        out_file
            .write_all(polymer[start..start + read_len].as_bytes())
            .unwrap();
        writeln!(out_file).unwrap();
    }

    let bases_covered = base_read_depths.iter().filter(|&&count| count != 0).count();
    let bases_covered2 = cover_depth(read_start_ends.iter().copied())
        .filter(|depth| depth.depth != 0)
        .count();
    let avg_depth = (num_reads * read_len) as f64 / genome_length as f64;
    let var_depth = base_read_depths
        .iter()
        .copied()
        .map(|base_depth| (base_depth as f64 - avg_depth).powi(2))
        .sum::<f64>()
        / (genome_length - 1) as f64;
    let var_depth2 = cover_depth(read_start_ends.iter().copied())
        .map(|base_depth| (base_depth.depth as f64 - avg_depth).powi(2))
        .sum::<f64>()
        / (genome_length - 1) as f64;
    let overlap_length = (read_len as f64 * theta).ceil() as usize;
    let num_islands = cover_depth(
        read_start_ends
            .iter()
            .copied()
            .zip(
                read_start_ends
                    .iter()
                    .copied()
                    .skip(overlap_length)
                    .chain(iter::repeat(StartEnd::default())),
            )
            .map(|(start_end, start_end_shifted)| StartEnd {
                start: start_end.start,
                end: start_end_shifted.end,
            }),
    )
    .fold((0, false), |(mut num_islands, mut on_island), cover| {
        if cover.depth != 0 && !on_island {
            on_island = true;
            num_islands += 1;
        } else if cover.depth == 0 && on_island {
            on_island = false;
        };

        // println!("num: {}, cov: {}, isl: {}", num_islands, cover.depth, on_island);

        (num_islands, on_island)
    })
    .0;

    println!(
        "genome_length: {}, num_reads: {}, bases_covered: {}, bases_covered2: {}, avg_depth: {}, var_depth: {}, var_depth2: {}, num_islands: {}",
        genome_length,
        num_reads,
        bases_covered,
        bases_covered2,
        avg_depth,
        var_depth,
        var_depth2,
        num_islands
    );
}

fn cover_depth(iter: impl IntoIterator<Item = StartEnd>) -> impl Iterator<Item = CoverDepth> {
    iter.into_iter()
        .scan(CoverDepth::default(), |cur_depth, start_end| {
            cur_depth.depth += start_end.start;
            cur_depth.depth -= start_end.end;
            Some(*cur_depth)
        })
}

#[derive(Debug, Clone, Default, Copy)]
struct StartEnd {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone, Default, Copy)]
struct CoverDepth {
    depth: usize,
}
