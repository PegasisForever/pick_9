#[macro_use]
extern crate log;

use fastrand::Rng as FastRng;
use rand::Rng;
use rand::prelude::ThreadRng;
use serde::Serialize;
use std::env;
use std::thread;
use std::time::SystemTime;
use reqwest::blocking::Client;
use std::env::VarError;
use std::ops::RangeInclusive;

const ROUND: usize = 100000000;

struct Collector {
    three_digit_number: [usize; 670],
    mod_by_9: [usize; 9],
    divided_by_9_count: [usize; 5],
    coin_head_calc: [usize; 17],
    coin_tail_calc: [usize; 6],
}

impl Collector {
    pub fn into_vec(self) -> CollectorVec {
        CollectorVec {
            three_digit_number: Vec::from(self.three_digit_number),
            mod_by_9: Vec::from(self.mod_by_9),
            divided_by_9_count: Vec::from(self.divided_by_9_count),
            coin_head_calc: Vec::from(self.coin_head_calc),
            coin_tail_calc: Vec::from(self.coin_tail_calc),
        }
    }
}

#[derive(Serialize)]
struct CollectorVec {
    three_digit_number: Vec<usize>,
    mod_by_9: Vec<usize>,
    divided_by_9_count: Vec<usize>,
    coin_head_calc: Vec<usize>,
    coin_tail_calc: Vec<usize>,
}

const DEFAULT_SERVER_ADDRESS: &str = "https://api.pegasis.site/pick_9/compute/";

fn main() {
    env::set_var("RUST_LOG", "compute=info");
    env_logger::init();

    let server_address = env::var("SERVER_ADDRESS")
        .or::<VarError>(Ok(String::from(DEFAULT_SERVER_ADDRESS)))
        .unwrap();

    let args: Vec<String> = env::args().collect();
    let thread_count = if args.len() == 2 {
        args.get(1).unwrap().parse::<usize>().unwrap()
    } else {
        1usize
    };

    info!("Starting {} thread(s).....", thread_count);

    let mut thread_handles = vec![];
    for i in 0..thread_count {
        let server_address = server_address.clone();
        thread_handles.push(thread::spawn(move || {
            let client = Client::new();
            let mut rng = FastRng::new();
            loop {
                let start_time = SystemTime::now();
                let mut collector = Collector {
                    three_digit_number: [0; 670],
                    mod_by_9: [0; 9],
                    divided_by_9_count: [0; 5],
                    coin_head_calc: [0; 17],
                    coin_tail_calc: [0; 6],
                };

                for _ in 0..ROUND {
                    round(&mut rng, &mut collector);
                }

                info!("Thread #{}: Did {} trials, speed {:.0} trials/sec/thread, uploading", i + 1, ROUND, ROUND as f64 / start_time.elapsed().unwrap().as_secs_f64());

                let collector_vec = collector.into_vec();
                let json = serde_json::to_string(&collector_vec).unwrap();

                let response = client.post(&server_address)
                    .header("content-type", "application/json")
                    .body(json)
                    .send();
                if let Err(e) = response {
                    error!("Thread #{}: Error uploading, error: {:?}", i + 1, e);
                } else {
                    let status = response.unwrap().status().as_u16();
                    if status == 200 {
                        info!("Thread #{}: Uploaded.", i + 1)
                    } else {
                        error!("Thread #{}: Error uploading, status: {:?}", i + 1, status);
                    }
                }
            }
        }));
    }

    info!("Started {} thread(s).", thread_count);

    thread_handles.into_iter().for_each(|handle| {
        let _ = handle.join();
    });
}

fn round(rng: &mut FastRng, collector: &mut Collector) -> usize {
    let mut x = divided_by_9_count(rng, collector);
    if coin(rng) {
        x = x * x;
        collector.coin_head_calc[x] += 1;
        x
    } else {
        x = x + 1;
        collector.coin_tail_calc[x] += 1;
        x
    }
}


fn divided_by_9_count(rng: &mut FastRng, collector: &mut Collector) -> usize {
    let mut count = 0;

    let a = three_digit_number(rng, collector) % 9;
    let b = three_digit_number(rng, collector) % 9;
    let c = three_digit_number(rng, collector) % 9;
    let d = three_digit_number(rng, collector) % 9;
    collector.mod_by_9[a] += 1;
    collector.mod_by_9[b] += 1;
    collector.mod_by_9[c] += 1;
    collector.mod_by_9[d] += 1;

    if a == 0 { count += 1; }
    if b == 0 { count += 1; }
    if c == 0 { count += 1; }
    if d == 0 { count += 1; }
    collector.divided_by_9_count[count] += 1;

    count
}

fn three_digit_number(rng: &mut FastRng, collector: &mut Collector) -> usize {
    let mut number = 3;
    number += dice(rng);
    number += dice(rng) * 10;
    number += dice(rng) * 100;
    collector.three_digit_number[number] += 1;
    number
}

// 1..=6
const DICE_RANGE: RangeInclusive<u64> = 1..=6;

fn dice(rng: &mut FastRng) -> usize {
    rng.u64(DICE_RANGE) as usize
}

// true: head  false: tail
fn coin(rng: &mut FastRng) -> bool {
    rng.bool()
}
