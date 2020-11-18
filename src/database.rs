use std::sync::Mutex;
use tokio::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct Collector {
    three_digit_number: Vec<u128>,
    mod_by_9: Vec<u128>,
    divided_by_9_count: Vec<u128>,
    coin_head_calc: Vec<u128>,
    coin_tail_calc: Vec<u128>,
}

#[derive(Serialize, Deserialize)]
struct CollectorStore {
    three_digit_number: Vec<String>,
    mod_by_9: Vec<String>,
    divided_by_9_count: Vec<String>,
    coin_head_calc: Vec<String>,
    coin_tail_calc: Vec<String>,
}

impl CollectorStore {
    pub fn from_collector(collector: &Collector) -> Self {
        Self {
            three_digit_number: collector.three_digit_number.iter().map(|num| format!("{}", num)).collect(),
            mod_by_9: collector.mod_by_9.iter().map(|num| format!("{}", num)).collect(),
            divided_by_9_count: collector.divided_by_9_count.iter().map(|num| format!("{}", num)).collect(),
            coin_head_calc: collector.coin_head_calc.iter().map(|num| format!("{}", num)).collect(),
            coin_tail_calc: collector.coin_tail_calc.iter().map(|num| format!("{}", num)).collect(),
        }
    }

    pub fn into_collector(self) -> Collector {
        Collector {
            three_digit_number: self.three_digit_number.into_iter().map(|str| str.parse().unwrap()).collect(),
            mod_by_9: self.mod_by_9.into_iter().map(|str| str.parse().unwrap()).collect(),
            divided_by_9_count: self.divided_by_9_count.into_iter().map(|str| str.parse().unwrap()).collect(),
            coin_head_calc: self.coin_head_calc.into_iter().map(|str| str.parse().unwrap()).collect(),
            coin_tail_calc: self.coin_tail_calc.into_iter().map(|str| str.parse().unwrap()).collect(),
        }
    }
}

pub struct DataBase {
    file_path: String,
    collector: Mutex<Collector>,
    total: Mutex<u128>,
}

impl DataBase {
    pub async fn new(file_path: String) -> DataBase {
        let path = Path::new(&file_path);
        let collector: Collector = if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await.unwrap();
            }
            Collector {
                three_digit_number: vec![0u128; 670],
                mod_by_9: vec![0u128; 9],
                divided_by_9_count: vec![0u128; 5],
                coin_head_calc: vec![0u128; 17],
                coin_tail_calc: vec![0u128; 6],
            }
        } else {
            serde_json::from_str::<CollectorStore>(&fs::read_to_string(path).await.unwrap()).unwrap().into_collector()
        };

        let mut total = 0u128;
        for x in &collector.divided_by_9_count {
            total += x;
        }

        info!("Read {} trials from {}.", total, &file_path);
        DataBase {
            file_path,
            collector: Mutex::new(collector),
            total: Mutex::new(total),
        }
    }

    pub async fn save(&self) {
        fs::write(&self.file_path, self.serialize()).await.unwrap();
        info!("Database saved to {}.", &self.file_path);
    }

    fn serialize(&self) -> String {
        let collector = self.collector.lock().unwrap();
        let collector_store = CollectorStore::from_collector(&collector);

        serde_json::to_string(&collector_store).unwrap()
    }

    pub async fn add(&self, collector: Collector) -> u128 {
        let total = {
            let mut self_collector = self.collector.lock().unwrap();
            let mut total = self.total.lock().unwrap();
            for i in 0..670 {
                self_collector.three_digit_number[i] += collector.three_digit_number[i];
            }
            for i in 0..9 {
                self_collector.mod_by_9[i] += collector.mod_by_9[i];
            }
            for i in 0..5 {
                let count = collector.divided_by_9_count[i];
                self_collector.divided_by_9_count[i] += count;
                *total += count;
            }
            for i in 0..17 {
                self_collector.coin_head_calc[i] += collector.coin_head_calc[i];
            }
            for i in 0..6 {
                self_collector.coin_tail_calc[i] += collector.coin_tail_calc[i];
            }
            *total
        };
        self.save().await;
        total
    }
}
