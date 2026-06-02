use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Duration;

use array_list::ArrayList;
use criterion::measurement::WallTime;
use criterion::{
    BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};

const WORKLOAD_SIZES: [usize; 3] = [257, 1_031, 4099];

macro_rules! bench_array_list_capacities {
    ($group:expr, $len:expr, $bench:ident) => {{
        $bench::<8>($group, $len);
        $bench::<16>($group, $len);
        $bench::<32>($group, $len);
        $bench::<64>($group, $len);
    }};
}

fn list_id<const N: usize>(len: usize) -> BenchmarkId {
    BenchmarkId::new(format!("ArrayList<{N}>"), len)
}

fn pseudo_random_indexes(len: usize, count: usize) -> Vec<usize> {
    let mut state = 0x9e37_79b9_7f4a_7c15usize;
    let mut indexes = Vec::with_capacity(count);

    for _ in 0..count {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005usize)
            .wrapping_add(1);
        indexes.push(state % len);
    }

    indexes
}

fn operation_count(len: usize) -> usize {
    (len / 8).clamp(16, 256)
}

fn dense_list<const N: usize>(len: usize) -> ArrayList<usize, N> {
    (0..len).collect()
}

fn churned_list<const N: usize>(len: usize) -> ArrayList<usize, N> {
    let ops = operation_count(len);
    let mut list = dense_list::<N>(len);

    for (offset, index) in pseudo_random_indexes(len, ops).into_iter().enumerate() {
        list.insert(index % (len + offset), offset);
    }
    for (offset, index) in pseudo_random_indexes(len, ops / 2).into_iter().enumerate() {
        list.remove(index % (len + ops - offset));
    }

    list
}

fn churned_vec(len: usize) -> Vec<usize> {
    let ops = operation_count(len);
    let mut vec = Vec::from_iter(0..len);

    for (offset, index) in pseudo_random_indexes(len, ops).into_iter().enumerate() {
        vec.insert(index % (len + offset), offset);
    }
    for (offset, index) in pseudo_random_indexes(len, ops / 2).into_iter().enumerate() {
        vec.remove(index % (len + ops - offset));
    }

    vec
}

fn churned_deque(len: usize) -> VecDeque<usize> {
    let ops = operation_count(len);
    let mut deque = VecDeque::from_iter(0..len);

    for (offset, index) in pseudo_random_indexes(len, ops).into_iter().enumerate() {
        deque.insert(index % (len + offset), offset);
    }
    for (offset, index) in pseudo_random_indexes(len, ops / 2).into_iter().enumerate() {
        deque.remove(index % (len + ops - offset)).unwrap();
    }

    deque
}

fn bench_list_push_back<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    group.bench_function(list_id::<N>(len), |b| {
        b.iter(|| {
            let mut list = ArrayList::<usize, N>::new();
            for value in 0..len {
                list.push_back(black_box(value));
            }
            black_box(list)
        });
    });
}

fn bench_build_push_back(c: &mut Criterion) {
    let mut group = c.benchmark_group("build/push_back");

    for len in WORKLOAD_SIZES {
        group.throughput(Throughput::Elements(len as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_push_back);

        group.bench_function(BenchmarkId::new("Vec", len), |b| {
            b.iter(|| {
                let mut vec = Vec::new();
                for value in 0..len {
                    vec.push(black_box(value));
                }
                black_box(vec)
            });
        });

        group.bench_function(BenchmarkId::new("VecDeque", len), |b| {
            b.iter(|| {
                let mut deque = VecDeque::new();
                for value in 0..len {
                    deque.push_back(black_box(value));
                }
                black_box(deque)
            });
        });
    }

    group.finish();
}

fn bench_list_push_front<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    group.bench_function(list_id::<N>(len), |b| {
        b.iter(|| {
            let mut list = ArrayList::<usize, N>::new();
            for value in 0..len {
                list.push_front(black_box(value));
            }
            black_box(list)
        });
    });
}

fn bench_build_push_front(c: &mut Criterion) {
    let mut group = c.benchmark_group("build/push_front");

    for len in WORKLOAD_SIZES {
        group.throughput(Throughput::Elements(len as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_push_front);

        group.bench_function(BenchmarkId::new("Vec", len), |b| {
            b.iter(|| {
                let mut vec = Vec::new();
                for value in 0..len {
                    vec.insert(0, black_box(value));
                }
                black_box(vec)
            });
        });

        group.bench_function(BenchmarkId::new("VecDeque", len), |b| {
            b.iter(|| {
                let mut deque = VecDeque::new();
                for value in 0..len {
                    deque.push_front(black_box(value));
                }
                black_box(deque)
            });
        });
    }

    group.finish();
}

fn bench_list_pop_back<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    group.bench_function(list_id::<N>(len), |b| {
        b.iter_batched(
            || dense_list::<N>(len),
            |mut list| {
                let mut sum = 0usize;
                while let Some(value) = list.pop_back() {
                    sum = sum.wrapping_add(black_box(value));
                }
                black_box(sum)
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_pop_back(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove/pop_back");

    for len in WORKLOAD_SIZES {
        group.throughput(Throughput::Elements(len as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_pop_back);

        group.bench_function(BenchmarkId::new("Vec", len), |b| {
            b.iter_batched(
                || Vec::from_iter(0..len),
                |mut vec| {
                    let mut sum = 0usize;
                    while let Some(value) = vec.pop() {
                        sum = sum.wrapping_add(black_box(value));
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("VecDeque", len), |b| {
            b.iter_batched(
                || VecDeque::from_iter(0..len),
                |mut deque| {
                    let mut sum = 0usize;
                    while let Some(value) = deque.pop_back() {
                        sum = sum.wrapping_add(black_box(value));
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_list_pop_front<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    group.bench_function(list_id::<N>(len), |b| {
        b.iter_batched(
            || dense_list::<N>(len),
            |mut list| {
                let mut sum = 0usize;
                while let Some(value) = list.pop_front() {
                    sum = sum.wrapping_add(black_box(value));
                }
                black_box(sum)
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_pop_front(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove/pop_front");

    for len in WORKLOAD_SIZES {
        group.throughput(Throughput::Elements(len as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_pop_front);

        group.bench_function(BenchmarkId::new("Vec", len), |b| {
            b.iter_batched(
                || Vec::from_iter(0..len),
                |mut vec| {
                    let mut sum = 0usize;
                    while !vec.is_empty() {
                        sum = sum.wrapping_add(black_box(vec.remove(0)));
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("VecDeque", len), |b| {
            b.iter_batched(
                || VecDeque::from_iter(0..len),
                |mut deque| {
                    let mut sum = 0usize;
                    while let Some(value) = deque.pop_front() {
                        sum = sum.wrapping_add(black_box(value));
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_list_iter_dense<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    let list = dense_list::<N>(len);

    group.bench_function(list_id::<N>(len), |b| {
        b.iter(|| {
            black_box(
                list.iter()
                    .fold(0usize, |acc, value| acc.wrapping_add(*value)),
            )
        });
    });
}

fn bench_iter_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("iter/dense_sum");

    for len in WORKLOAD_SIZES {
        group.throughput(Throughput::Elements(len as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_iter_dense);

        let vec = Vec::from_iter(0..len);
        let deque = VecDeque::from_iter(0..len);

        group.bench_function(BenchmarkId::new("Vec", len), |b| {
            b.iter(|| {
                black_box(
                    vec.iter()
                        .fold(0usize, |acc, value| acc.wrapping_add(*value)),
                )
            });
        });

        group.bench_function(BenchmarkId::new("VecDeque", len), |b| {
            b.iter(|| {
                black_box(
                    deque
                        .iter()
                        .fold(0usize, |acc, value| acc.wrapping_add(*value)),
                )
            });
        });
    }

    group.finish();
}

fn bench_list_get_churned<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    let list = churned_list::<N>(len);
    let indexes = pseudo_random_indexes(list.len(), list.len());

    group.bench_function(list_id::<N>(list.len()), |b| {
        b.iter(|| {
            let mut sum = 0usize;
            for &index in &indexes {
                sum = sum.wrapping_add(*black_box(list.get(index).unwrap()));
            }
            black_box(sum)
        });
    });
}

fn bench_random_access_churned(c: &mut Criterion) {
    let mut group = c.benchmark_group("get/churned_random");

    for len in WORKLOAD_SIZES {
        group.throughput(Throughput::Elements(len as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_get_churned);

        let vec = churned_vec(len);
        let deque = churned_deque(len);
        let indexes = pseudo_random_indexes(vec.len(), vec.len());

        group.bench_function(BenchmarkId::new("Vec", vec.len()), |b| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indexes {
                    sum = sum.wrapping_add(*black_box(vec.get(index).unwrap()));
                }
                black_box(sum)
            });
        });

        group.bench_function(BenchmarkId::new("VecDeque", deque.len()), |b| {
            b.iter(|| {
                let mut sum = 0usize;
                for &index in &indexes {
                    sum = sum.wrapping_add(*black_box(deque.get(index).unwrap()));
                }
                black_box(sum)
            });
        });
    }

    group.finish();
}

fn bench_list_insert_middle<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    let ops = operation_count(len);
    let indexes = pseudo_random_indexes(len, ops);

    group.bench_function(list_id::<N>(len), |b| {
        b.iter_batched(
            || dense_list::<N>(len),
            |mut list| {
                for (offset, &index) in indexes.iter().enumerate() {
                    list.insert(index % (len + offset), black_box(offset));
                }
                black_box(list)
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_insert_middle(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert/random_middle_batch");

    for len in WORKLOAD_SIZES {
        let ops = operation_count(len);
        group.throughput(Throughput::Elements(ops as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_insert_middle);
        let indexes = pseudo_random_indexes(len, ops);

        group.bench_function(BenchmarkId::new("Vec", len), |b| {
            b.iter_batched(
                || Vec::from_iter(0..len),
                |mut vec| {
                    for (offset, &index) in indexes.iter().enumerate() {
                        vec.insert(index % (len + offset), black_box(offset));
                    }
                    black_box(vec)
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("VecDeque", len), |b| {
            b.iter_batched(
                || VecDeque::from_iter(0..len),
                |mut deque| {
                    for (offset, &index) in indexes.iter().enumerate() {
                        deque.insert(index % (len + offset), black_box(offset));
                    }
                    black_box(deque)
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_list_remove_middle<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    let ops = operation_count(len);
    let indexes = pseudo_random_indexes(len, ops);

    group.bench_function(list_id::<N>(len), |b| {
        b.iter_batched(
            || dense_list::<N>(len),
            |mut list| {
                let mut sum = 0usize;
                for (offset, &index) in indexes.iter().enumerate() {
                    sum = sum.wrapping_add(list.remove(index % (len - offset)).unwrap());
                }
                black_box(sum)
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_remove_middle(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove/random_middle_batch");

    for len in WORKLOAD_SIZES {
        let ops = operation_count(len);
        group.throughput(Throughput::Elements(ops as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_remove_middle);
        let indexes = pseudo_random_indexes(len, ops);

        group.bench_function(BenchmarkId::new("Vec", len), |b| {
            b.iter_batched(
                || Vec::from_iter(0..len),
                |mut vec| {
                    let mut sum = 0usize;
                    for (offset, &index) in indexes.iter().enumerate() {
                        sum = sum.wrapping_add(vec.remove(index % (len - offset)));
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("VecDeque", len), |b| {
            b.iter_batched(
                || VecDeque::from_iter(0..len),
                |mut deque| {
                    let mut sum = 0usize;
                    for (offset, &index) in indexes.iter().enumerate() {
                        sum = sum.wrapping_add(deque.remove(index % (len - offset)).unwrap());
                    }
                    black_box(sum)
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_list_cursor_edit<const N: usize>(group: &mut BenchmarkGroup<'_, WallTime>, len: usize) {
    let operations = len / 4;

    group.bench_function(list_id::<N>(len), |b| {
        b.iter_batched(
            || dense_list::<N>(len),
            |mut list| {
                let mut cursor = list.cursor_front_mut();
                for _ in 0..(len / 2) {
                    cursor.move_next();
                }

                for value in 0..operations {
                    cursor.insert_after(black_box(value));
                    cursor.move_next();
                    black_box(cursor.remove_current());
                }

                black_box(list)
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_cursor_edit(c: &mut Criterion) {
    let mut group = c.benchmark_group("cursor/edit_near_middle");

    for len in WORKLOAD_SIZES {
        let operations = len / 4;

        group.throughput(Throughput::Elements(operations as u64));
        bench_array_list_capacities!(&mut group, len, bench_list_cursor_edit);

        // Match the ArrayList cursor edit pattern from the equivalent indexed
        // position: insert after the current element, remove the inserted value,
        // and continue from the next original element.
        group.bench_function(BenchmarkId::new("Vec", len), |b| {
            b.iter_batched(
                || Vec::from_iter(0..len),
                |mut vec| {
                    let mut index = len / 2;

                    for value in 0..operations {
                        let inserted = index + 1;
                        vec.insert(inserted, black_box(value));
                        index = inserted;
                        black_box(Some(vec.remove(index)));
                    }
                    black_box(vec)
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_function(BenchmarkId::new("VecDeque", len), |b| {
            b.iter_batched(
                || VecDeque::from_iter(0..len),
                |mut deque| {
                    let mut index = len / 2;

                    for value in 0..operations {
                        let inserted = index + 1;
                        deque.insert(inserted, black_box(value));
                        index = inserted;
                        black_box(deque.remove(index));
                    }
                    black_box(deque)
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
    targets =
        bench_build_push_back,
        bench_build_push_front,
        bench_pop_back,
        bench_pop_front,
        bench_iter_dense,
        bench_random_access_churned,
        bench_insert_middle,
        bench_remove_middle,
        bench_cursor_edit,
}
criterion_main!(benches);
