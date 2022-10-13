#![cfg(test)]
//  Copyright 2021 Leon Stichternath
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// -----------
// Original source can be found at https://gitlab.informatik.uni-bremen.de/parametrisierte-algorithmen/rust/ceperus/-/blob/v2.0.0/ceperus-exact/benches/benchmark.rs.
// Modifications compared to the original source code are indicated with
// `MOD'.
use assert_cmd::Command;
use rayon::ThreadPoolBuilder;
use std::{
    env,
    io::{self, Write},
    path::Path,
    time::Duration,
};

fn main() {
    let num_threads = env::args().nth(1).unwrap().parse().unwrap();
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    let time_limit_secs = env::args().nth(2).unwrap().parse().unwrap();
    let time_limit = Duration::from_secs(time_limit_secs);

    let mut dir = std::fs::read_dir(env::args().nth(3).unwrap())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();

    dir.sort_unstable_by_key(|entry| entry.file_name());

    rayon::scope_fifo(|scope| {
        for entry in dir {
            scope.spawn_fifo(move |_| {
                // MOD: use real time instead of CPU time, may change in the future
                let time = std::time::Instant::now();
                let report = match solve_one(&entry.path(), time_limit) {
                    Ok(k) => format!("{},{:.3}", k, time.elapsed().as_secs_f32()),
                    Err(_) => format!(" ERROR {:.3}", time.elapsed().as_secs_f32(),),
                };

                let out = io::stdout();
                let mut lock = out.lock();
                let _ = writeln!(lock, "{},{}", entry.file_name().to_string_lossy(), report);
            });
        }
    });
}

fn solve_one(file_path: &Path, time_limit: Duration) -> Result<usize, String> {
    // Rayon has difficulties cleaning up child processes, so we need a small hack
    // We assume we equire about 100s to reach vc_solver in the execution path
    let args = ["-n".to_owned(), "-a=ilp".to_owned()];
    Ok(String::from_utf8(
        Command::cargo_bin("hex")
            .unwrap()
            .args(args)
            .pipe_stdin(file_path)
            .map_err(|err| err.to_string())?
            .timeout(time_limit)
            .ok()
            .map_err(|err| err.to_string())?
            .stdout,
    )
    .unwrap()
    .lines()
    .count())
}

fn stats_solve_one(file_path: &Path, time_limit: Duration) -> Result<String, String> {
    // Rayon has difficulties cleaning up child processes, so we need a small hack
    // We assume we equire about 100s to reach vc_solver in the execution path
    let args = [format!("-n"), format!("-a=ilp")];
    Ok(String::from_utf8(
        Command::cargo_bin("hex")
            .unwrap()
            .args(&args)
            .pipe_stdin(file_path)
            .map_err(|err| err.to_string())?
            .timeout(time_limit)
            .ok()
            .map_err(|err| err.to_string())?
            .stderr,
    )
    .unwrap())
}
