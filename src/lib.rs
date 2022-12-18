mod constants;
use crate::constants::*;

use anyhow::Error;
use std::{
    fmt::Display,
    ops::Drop,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::{self, SystemTime, UNIX_EPOCH},
    vec,
};

enum Activity {
    Start,
    Eat,
    Sleep,
    Think,
    TakingFork,
}

// TODO implement the drop trait for philosopher allowing you to just print died when dropped
struct Philosopher {
    pub activity: Activity,
    pub ts_last_eat: SystemTime,
    pub table_position: usize,
}

impl Display for Philosopher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self.activity {
            Activity::Eat => "is eating",
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
        println!("{} {} died", SystemTime::now() .duration_since(UNIX_EPOCH) .unwrap() .as_millis(), self.table_position + 1);
    }
}

impl Philosopher {
    pub fn new(pos: usize) -> Self {
        return Philosopher {
            activity: Activity::Start,
            ts_last_eat: SystemTime::now(),
            table_position: pos,
        };
    }

    pub fn change_activity(&mut self) {
        match self.activity {
            Activity::Start => self.activity = Activity::Eat,
            Activity::Eat => self.activity = Activity::Sleep,
            Activity::Sleep => self.activity = Activity::Think,
            Activity::Think => self.activity = Activity::Eat,
            _ => (),
        }
    }
}

// TODO make a philoroutine function for the thread
// TODO add state eating, eating has minimum time and can die while eating
pub fn start_dinning(amount: usize) -> Result<Vec<JoinHandle<()>>, Error> {
    let mut handles = vec![];
    let dead_philo = Arc::new(Mutex::new(false));
    let min = std::cmp::max(amount, 2);
    let forks: Arc<Vec<Mutex<i32>>> = Arc::new((0..min).map(|_| Mutex::new(1)).collect());

    for i in 0..amount {
        let my_dead_philo = Arc::clone(&dead_philo);
        let my_forks = Arc::clone(&forks);
        let handle = thread::spawn(move || {
            println!("Spawing philosopher");
            let forks = my_forks;
            let mut philo = Philosopher::new(i);
            let position = philo.table_position;
            // allows to pick position, (position + amount - 1) % amount if position is even and (position + amount - 1) % amount, position if position is odd
            let f = vec![position, (position + amount - 1) % amount, position];
            loop {
                philo.change_activity();
                match philo.activity {
                    Activity::Sleep => {
                        println!("{}", philo);
                        thread::sleep(time::Duration::from_millis(TIME_SLEEP as u64))
                    }
                    Activity::Think => {
                        println!("{}", philo);
                        // TODO thinking has no duration time, should just be a state after eating before taking both forks
                    }
                    Activity::Eat => {
                        philo.activity = Activity::TakingFork;
                        let _right_fork = forks[f[position % 2]].lock().unwrap();
                        println!("{}", philo);
                        let _left_fork = forks[f[position % 2 + 1]].lock().unwrap();
                        println!("{}", philo);
                        philo.activity = Activity::Eat;
                        println!("{}", philo);
                        philo.ts_last_eat = SystemTime::now();
                        thread::sleep(time::Duration::from_millis(TIME_EAT as u64));
                    }
                    _ => panic!("incorrect state"),
                }
                if SystemTime::now()
                    .duration_since(philo.ts_last_eat)
                    .unwrap()
                    .as_millis()
                    >= TIME_DIE
                {
                    let mut dead = my_dead_philo.lock().unwrap();
                    *dead = true;
                    return;
                }
                if *my_dead_philo.lock().unwrap() {
                    return;
                }
            }
        });
        handles.push(handle);
    }
    return Ok(handles);
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
        let handles = start_dinning(50).unwrap();
        for h in handles {
            h.join().unwrap();
        }
    }
}
