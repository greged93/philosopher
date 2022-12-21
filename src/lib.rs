mod constants;
use crate::constants::*;

use anyhow::Error;
use std::{
    fmt::Display,
    ops::Drop,
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, JoinHandle},
    time::{self, SystemTime, UNIX_EPOCH},
    vec,
};

enum Activity {
    Start,
    Eat,
    Eating,
    Sleep,
    Think,
    TakingFork,
}

struct Philosopher {
    pub activity: Activity,
    pub ts_last_eat: SystemTime,
    pub ts_eating: SystemTime,
    pub table_position: usize,
}

impl Display for Philosopher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self.activity {
            Activity::Eating => "is eating",
            Activity::Sleep => "is sleeping",
            Activity::Think => "is thinking",
            Activity::TakingFork => "has taken a fork",
            _ => "error",
        };
        write!(
            f,
            "{} {} {}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            self.table_position + 1,
            state
        )
    }
}

impl Drop for Philosopher {
    fn drop(&mut self) {
        println!(
            "{} {} died",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            self.table_position + 1
        );
    }
}

impl Philosopher {
    pub fn new(pos: usize) -> Self {
        Philosopher {
            activity: Activity::Start,
            ts_last_eat: SystemTime::now(),
            ts_eating: SystemTime::now(),
            table_position: pos,
        }
    }

    pub fn is_dead(&self) -> bool {
        SystemTime::now()
            .duration_since(self.ts_last_eat)
            .unwrap()
            .as_millis()
            >= TIME_DIE
    }

    pub fn is_done_eating(&self) -> bool {
        SystemTime::now()
            .duration_since(self.ts_eating)
            .unwrap()
            .as_millis()
            > TIME_EAT
    }
}

pub fn start_dinning(amount: usize) -> Result<Vec<JoinHandle<()>>, Error> {
    let mut handles = vec![];
    let dead_philo = Arc::new(Mutex::new(false));
    let min = std::cmp::max(amount, 2);
    let forks: Arc<Vec<Mutex<i32>>> = Arc::new((0..min).map(|_| Mutex::new(1)).collect());

    for i in 0..amount {
        let my_dead_philo = Arc::clone(&dead_philo);
        let my_forks = Arc::clone(&forks);
        let handle = thread::spawn(move || philo_routine(i, amount, my_forks, my_dead_philo));
        handles.push(handle);
        thread::sleep(time::Duration::from_nanos(100));
    }
    Ok(handles)
}

fn philo_routine(
    i: usize,
    amount: usize,
    forks: Arc<Vec<Mutex<i32>>>,
    dead_philo: Arc<Mutex<bool>>,
) {
    let mut philo = Philosopher::new(i);
    let position = philo.table_position;
    // allows to pick position, (position + amount - 1) % amount if position is even and (position + amount - 1) % amount, position if position is odd
    let f = vec![position, (position + amount - 1) % amount, position];
    // allows to keep the mutex for the forks until the values are popped when done eating
    let mut dishes: Vec<MutexGuard<i32>> = vec![];
    loop {
        match philo.activity {
            Activity::Start => philo.activity = Activity::Eat,
            Activity::Sleep => {
                println!("{}", philo);
                thread::sleep(time::Duration::from_millis(TIME_SLEEP as u64));
                philo.activity = Activity::Think;
            }
            Activity::Think => {
                println!("{}", philo);
                philo.activity = Activity::Eat;
            }
            Activity::Eat => {
                philo.activity = Activity::TakingFork;
                dishes.push(forks[f[position % 2]].lock().unwrap());
                println!("{}", philo);
                dishes.push(forks[f[position % 2 + 1]].lock().unwrap());
                println!("{}", philo);
                philo.activity = Activity::Eating;
                println!("{}", philo);
                philo.ts_eating = SystemTime::now();
            }
            Activity::Eating => {
                if philo.is_done_eating() {
                    philo.ts_last_eat = SystemTime::now();
                    philo.activity = Activity::Sleep;
                    dishes.pop();
                    dishes.pop();
                }
            }
            _ => panic!("incorrect state"),
        }
        if philo.is_dead() {
            let mut dead = dead_philo.lock().unwrap();
            *dead = true;
            return;
        }
        if *dead_philo.lock().unwrap() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_philosopher() {
        let handles = start_dinning(1).unwrap();
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_two_philosopher() {
        let handles = start_dinning(2).unwrap();
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_n_philosopher() {
        let handles = start_dinning(16).unwrap();
        for h in handles {
            h.join().unwrap();
        }
    }
}
