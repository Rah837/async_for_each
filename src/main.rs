mod async;
use async::Async;

use std::collections::BTreeMap;



fn main() {
    let mut map = BTreeMap::new();
    map.insert('a', 1);
    map.insert('b', 2);
    map.insert('c', 3);
    map.insert('d', 4);

    map.iter_mut().enumerate().call(|&mut (i, (_, ref mut val))| **val += i + 1, true);

    println!("map = {:?}", map);
}