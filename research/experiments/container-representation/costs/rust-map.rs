//! Standard-library comparison for the exact scalar lookup workload in map-layout.c.
//! This is an algorithm comparator, not an isolation of representation alone.
#![forbid(unsafe_code)]
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::hint::black_box;
use std::time::Instant;

fn mix(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}
fn key_at(i: usize) -> u64 {
    i as u64 * 2 + 1
}
fn value_at(key: u64) -> u64 {
    mix(key.wrapping_add(91))
}
#[derive(Default)]
struct KeyHasher(u64);
impl Hasher for KeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        assert_eq!(bytes.len(), 8, "this experiment hashes only u64 keys");
        self.write_u64(u64::from_ne_bytes(bytes.try_into().unwrap()));
    }
    fn write_u64(&mut self, key: u64) {
        self.0 = mix(key);
    }
}
type Map = HashMap<u64, [u64; 3], BuildHasherDefault<KeyHasher>>;
#[inline(never)]
fn lookup(map: &Map, key: u64) -> Option<u64> {
    map.get(&key).map(|v| v[0])
}
#[inline(never)]
fn batch(map: &Map, queries: &[u64]) -> u64 {
    queries.iter().fold(0_u64, |sum, key| {
        sum.wrapping_add(lookup(map, *key).unwrap_or(0x9e37_79b9_7f4a_7c15))
    })
}
fn verify() {
    for capacity in [1, 2, 4, 8, 16, 32, 64] {
        let mut map = Map::default();
        let mut present = vec![false; capacity * 2];
        for (i, exists) in present.iter_mut().take(capacity).enumerate() {
            let key = key_at(i);
            assert!(
                map.insert(key, [value_at(key), key ^ 33, key ^ 77])
                    .is_none()
            );
            *exists = true;
        }
        assert!(map.insert(1, [value_at(1), 1 ^ 33, 1 ^ 77]).is_some());
        for i in (0..capacity).step_by(2) {
            assert!(map.remove(&key_at(i)).is_some());
            present[i] = false;
        }
        for (i, exists) in present
            .iter_mut()
            .enumerate()
            .skip(capacity)
            .take(capacity.div_ceil(2))
        {
            let key = key_at(i);
            assert!(
                map.insert(key, [value_at(key), key ^ 33, key ^ 77])
                    .is_none()
            );
            *exists = true;
        }
        for (i, exists) in present.iter().enumerate() {
            assert_eq!(lookup(&map, key_at(i)), exists.then(|| value_at(key_at(i))));
        }
    }
    println!("verified,rust_standard_map,replace,remove,reuse,missing,7_capacities");
}
fn measure() {
    println!(
        "kind,capacity,load_eighths,variant,sample,queries,nanoseconds,checksum,reported_capacity"
    );
    for capacity in [256_usize, 65536] {
        for eighths in [2, 7] {
            let live = capacity * eighths / 8;
            let mut map = Map::with_capacity_and_hasher(capacity * 7 / 8, Default::default());
            for i in 0..live {
                let key = key_at(i);
                map.insert(key, [value_at(key), key ^ 33, key ^ 77]);
            }
            let mut state = 7_640_891_576_956_012_809_u64;
            let mut expected = 0_u64;
            let queries: Vec<u64> = (0..32768)
                .map(|i| {
                    state = mix(state.wrapping_add(i));
                    let chosen = (state % (live as u64 * 2)) as usize;
                    let key = key_at(chosen);
                    expected = expected.wrapping_add(if chosen < live {
                        value_at(key)
                    } else {
                        0x9e37_79b9_7f4a_7c15
                    });
                    key
                })
                .collect();
            assert_eq!(batch(&map, &queries), expected);
            for sample in 0..7 {
                black_box(batch(black_box(&map), black_box(&queries)));
                let start = Instant::now();
                let mut sum = 0_u64;
                for _ in 0..12 {
                    sum = sum.wrapping_add(batch(black_box(&map), black_box(&queries)));
                }
                let elapsed = start.elapsed().as_nanos();
                black_box(sum);
                assert_eq!(sum, expected.wrapping_mul(12));
                println!(
                    "sample,{capacity},{eighths},rust_standard,{sample},{},{elapsed},{sum},{}",
                    queries.len() * 12,
                    map.capacity()
                );
            }
        }
    }
}
fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("check") => verify(),
        Some("measure") => measure(),
        _ => panic!("expected check or measure"),
    }
}
