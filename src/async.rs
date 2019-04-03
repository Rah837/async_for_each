extern crate crossbeam;

use std::{
    sync::{
        Condvar,
        RwLock,
        Mutex,
        Arc
    },
    thread,
    time,
    fmt,
    io
};



struct Element<T>((RwLock<T>, Mutex<bool>, Condvar));

impl<T> Element<T> {
    fn new(val: T) -> Self {
        Element((RwLock::new(val), Mutex::new(false), Condvar::new()))
    }
}

impl<T> fmt::Debug for Element<T>
    where T: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Element((ref val, _, _)) = self;
        write!(f, "{:?}", *val.read().unwrap())
    }
}


pub trait Async<F>
    where   Self: Iterator,
            Self::Item: fmt::Debug + Send + Sync,
            F: Send + Sync {
    fn call(self, F, bool)
        where F: Fn(&mut Self::Item);
}

impl<'a, F, T> Async<F> for &'a mut T
    where   Self: Iterator,
            Self::Item: fmt::Debug + Send + Sync,
            F: Send + Sync {
    fn call(self, f: F, show: bool)
        where F: Fn(&mut Self::Item) {
        let f = Arc::new(f);
        let complete = Arc::new(RwLock::new(false));
        let vec = Arc::new(self.map(|e| Element::new(e)).collect::<Vec<_>>());
        
        crossbeam::scope(|scope| {
            for &Element((ref val, ref pause, ref cvar)) in vec.iter() {
                let f = f.clone();
                let complete = complete.clone();
                scope.spawn(move || {
                    loop {
                        let pause = pause.lock().unwrap();
                        if *pause { let _ = cvar.wait(pause).unwrap(); }
                        if *complete.read().unwrap() { break; }
                        
                        f(&mut *val.write().unwrap());
                    }
                });
            }

            if show {
                let vec = vec.clone();
                let complete = complete.clone();
                scope.spawn(move || {
                    loop {
                        if *complete.read().unwrap() { break; }

                        thread::sleep(time::Duration::from_secs(1));
                        println!("{:?}", *vec);
                    }
                });
            }
        
            let vec = vec.clone();
            scope.spawn(move || {
                let mut input = String::new();

                loop {
                    input.clear();
                    io::stdin().read_line(&mut input)
                        .expect("could not read line");

                    match &input.trim().to_lowercase().as_str().split_whitespace().collect::<Vec<_>>()[..] {
                        &["complete"]   => {
                                *complete.write().unwrap() = true;

                                for &Element((_, ref pause, ref cvar)) in vec.iter() {
                                    *pause.lock().unwrap() = false;
                                    cvar.notify_one();
                                }

                                break;
                        },
                        &[idx, comm]    => match idx.parse::<usize>() {
                            Ok(idx) if idx < vec.len() => {
                                let &Element((_, ref pause, ref cvar)) = &vec[idx]; 
                                
                                match comm {
                                    "pause" => *pause.lock().unwrap() = true,
                                    "run" => {
                                        *pause.lock().unwrap() = false;
                                        cvar.notify_one();
                                    },
                                    _ => println!("no such subcommand")
                                };
                            },
                            Err(_) => println!("invalid index"),
                            _ => println!("the index is too large")
                        },
                        _ => println!("no such command")
                    }
                }
            });
        });
    }
}