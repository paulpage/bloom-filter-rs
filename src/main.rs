use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::env;

extern crate bit_vec;
extern crate time;

use bit_vec::BitVec;
use time::PreciseTime;

#[derive(Debug)]
struct BloomFilter<T> {
    bit_vec: BitVec,
    false_positive_prob: f64,
    bit_vec_size: usize,
    hash_count: usize,
    phantom: PhantomData<T>,
}

impl<T: Hash> BloomFilter<T> {
    fn new(item_count: usize, false_positive_prob: f64) -> BloomFilter<T> {
        let bit_vec_size = BloomFilter::<T>::get_size(item_count, false_positive_prob);
        BloomFilter {
            false_positive_prob: false_positive_prob,
            bit_vec_size: BloomFilter::<T>::get_size(item_count, false_positive_prob),
            hash_count: BloomFilter::<T>::get_hash_count(bit_vec_size, item_count),
            bit_vec: BitVec::from_elem(bit_vec_size, false),
            phantom: PhantomData,
        }
    }

    fn add(&mut self, item: &T) {
        for i in 0..self.hash_count {
            let index = BloomFilter::<T>::hash(i, item) % self.bit_vec_size;
            self.bit_vec.set(index, true);
        }
    }

    fn contains(&self, item: &T) -> bool {
        for i in 0..self.hash_count {
            let index = BloomFilter::<T>::hash(i, item) % self.bit_vec_size;
            if self.bit_vec[index] == false {
                return false;
            }
        }
        true
    }

    fn hash(i: usize, t: &T) -> usize {
        let mut s = DefaultHasher::new();
        s.write_usize(i);
        t.hash(&mut s);
        s.finish() as usize
    }

    fn get_size(n: usize, p: f64) -> usize {
        (n as f64 * p.ln() / ((2 as f64).ln() * (2 as f64).ln()) * (-1 as f64)) as usize
    }

    fn get_hash_count(m: usize, n: usize) -> usize {
        std::cmp::max((m as f64 / n as f64 * (2 as f64).ln()) as usize, 1)
    }
}

fn filter_from_file(path: &str, capacity: usize, false_positive_prob: f64) -> BloomFilter<String> {
    let mut filter = BloomFilter::<String>::new(capacity, false_positive_prob);

    let file = BufReader::new(File::open(path).expect(format!("Could not open file {}", path).as_str()));
    for line in file.lines() {
        filter.add(&line.unwrap().trim().to_string());
    }
    filter
}

fn check_from_file(path: &str, filter: &BloomFilter<String>) {

    let mut true_positives = 0;
    let mut false_negatives = 0;
    let mut false_positives = 0;
    let mut true_negatives = 0;

    // Check the rate at which the filter correctly identifies items that are in the file.
    // We will also track the largest line in the file so that we can use that value
    // to generate strings that are definitely not in the file later.
    let mut longest_string: String = String::new();
    let file = BufReader::new(File::open(path).expect(format!("Could not open file {}", path).as_str()));
    for line in file.lines() {
        let line = line.unwrap().trim().to_string();
        if filter.contains(&line) {
            true_positives += 1;
        } else {
            false_negatives += 1;
        }
        if line.len() > longest_string.len() {
            longest_string = line;
        }
        // max_line_len = std::cmp::max(max_line_len, line.unwrap().len());
    }

    // Generate strings that are longer than the longest line in the file, and are
    // thus guaranteed not to be in the file, and check how well the filter correctly
    // identifies that they are not in the filter.
    for i in 0..filter.bit_vec.len() {
        let mut st = longest_string.clone();
        st.push_str(&i.to_string());
        if filter.contains(&st) {
            false_positives += 1;
        } else {
            true_negatives += 1;
        }
    }

    println!("True Positives: {}", true_positives);
    println!("False Negatives: {}", false_negatives);
    println!("False Positives: {}", false_positives);
    println!("True Negatives: {}", true_negatives);
    println!("");
    println!("False Positives percentage: {}", false_positives as f64 / (false_positives + true_negatives) as f64);
}

fn main() {
    // let mut b = BloomFilter::<String>::new(1000, 0.1);
    let args: Vec<String> = env::args().collect();
    match args.len() {
        4 => {
            let filter = filter_from_file(
                &args[1],
                args[2].parse::<usize>().expect("Filter capacity must be a positive integer."),
                args[3].parse::<f64>().expect("False positive probability must be between 0 and 1."));
            check_from_file(&args[1], &filter);
        },
        2 => {

            let sizes = [
                1000,
                10000,
                100000,
                1000000,
                10000000,
                100000000,
                1000000000,
            ];
            let false_positive_prob = 0.1;

            for size in &sizes {
                let mut filter: BloomFilter<String> = BloomFilter::new(*size, false_positive_prob);

                // Populate the filter
                for i in 0..1000 {
                    filter.add(&i.to_string());
                }

                // Check 1000 elements, half of which will be in the filter
                let start = PreciseTime::now();
                for i in 0..1000 {
                    filter.contains(&(i + 500).to_string());
                }
                let end = PreciseTime::now();
                println!("{} {:?}", size, start.to(end));

                // println!("{} {}", size, filter.bit_vec.len());
            }
        },
        _ => {
            println!("Usage: {} <input-file>", &args[0]);
        },
    }
    // println!("{:?}", b);
    // b.add(&"Holle!".to_string());
    // println!("{:?}", b.contains(&"Holle!".to_string()));
}

