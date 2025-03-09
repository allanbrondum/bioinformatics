use itertools::Itertools;

fn main() {
    let data = include_str!("perm_data.txt");

    let length:usize = data.parse().unwrap();

    let mut res = vec![String::new()];
    for _ in 0..length {
        res = res
            .into_iter()
            .cartesian_product(1..=length)
            .filter(|(item, letter)|!item.contains(&letter.to_string()))
            .map(|(item, digit)| format!("{}{} ", item, digit))
            .collect();
    }

    println!("{}", res.len());
    for item in res {
        println!("{}", item);
    }
}
