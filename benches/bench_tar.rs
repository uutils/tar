// Benchmarks for the uutils tar implementation.
//
// These benchmarks exercise the core archive operations (create, list, extract)
// at various scales to track performance over time.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tar::CompressionMode;
use tar::operations;
use tempfile::TempDir;

fn main() {
    divan::main();
}

/// Create `count` files of `size` bytes each inside `dir`.
fn create_sample_files(dir: &Path, count: usize, size: usize) {
    let content = vec![b'A'; size];
    for i in 0..count {
        let path = dir.join(format!("file_{i}.txt"));
        let mut f = File::create(&path).unwrap();
        f.write_all(&content).unwrap();
    }
}

/// Collect all regular file paths inside `dir` (non-recursive).
fn collect_files(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    paths
}

/// Build a tar archive at `archive_path` from all files in `source_dir`.
fn build_archive(archive_path: &Path, source_dir: &Path) {
    let files = collect_files(source_dir);
    let refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
    let output = File::create(archive_path).unwrap();
    let status_output = io::sink();
    operations::create::create_archive(
        output,
        status_output,
        &refs,
        true,
        false,
        CompressionMode::None,
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// Create benchmarks
// ---------------------------------------------------------------------------

#[divan::bench]
fn create_archive_10_files(bencher: divan::Bencher) {
    let source = TempDir::new().unwrap();
    create_sample_files(source.path(), 10, 4_096);
    let files = collect_files(source.path());

    let out = TempDir::new().unwrap();
    let archive_path = out.path().join("bench.tar");

    bencher.bench_local(|| {
        let refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
        let output = File::create(&archive_path).unwrap();
        let status_output = io::sink();
        operations::create::create_archive(
            output,
            status_output,
            &refs,
            true,
            false,
            CompressionMode::None,
        )
        .unwrap();
    });
}

#[divan::bench]
fn create_archive_100_files(bencher: divan::Bencher) {
    let source = TempDir::new().unwrap();
    create_sample_files(source.path(), 100, 4_096);
    let files = collect_files(source.path());

    let out = TempDir::new().unwrap();
    let archive_path = out.path().join("bench.tar");

    bencher.bench_local(|| {
        let refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
        let output = File::create(&archive_path).unwrap();
        let status_output = io::sink();
        operations::create::create_archive(
            output,
            status_output,
            &refs,
            true,
            false,
            CompressionMode::None,
        )
        .unwrap();
    });
}

#[divan::bench]
fn create_archive_directory(bencher: divan::Bencher) {
    let source = TempDir::new().unwrap();
    let sub = source.path().join("data");
    fs::create_dir_all(&sub).unwrap();
    create_sample_files(&sub, 30, 4_096);

    let out = TempDir::new().unwrap();
    let archive_path = out.path().join("bench.tar");

    bencher.bench_local(|| {
        let output = File::create(&archive_path).unwrap();
        let status_output = io::sink();
        operations::create::create_archive(
            output,
            status_output,
            &[sub.as_path()],
            true,
            false,
            CompressionMode::None,
        )
        .unwrap();
    });
}

// ---------------------------------------------------------------------------
// List benchmarks
// ---------------------------------------------------------------------------

#[divan::bench]
fn list_archive_50_files(bencher: divan::Bencher) {
    let source = TempDir::new().unwrap();
    create_sample_files(source.path(), 50, 4_096);
    let archive_dir = TempDir::new().unwrap();
    let archive_path = archive_dir.path().join("bench.tar");
    build_archive(&archive_path, source.path());

    bencher.bench_local(|| {
        let input = File::open(&archive_path).unwrap();
        operations::list::list_archive(input, &archive_path, false, CompressionMode::None).unwrap();
    });
}

#[divan::bench]
fn list_archive_verbose_50_files(bencher: divan::Bencher) {
    let source = TempDir::new().unwrap();
    create_sample_files(source.path(), 50, 4_096);
    let archive_dir = TempDir::new().unwrap();
    let archive_path = archive_dir.path().join("bench.tar");
    build_archive(&archive_path, source.path());

    bencher.bench_local(|| {
        let input = File::open(&archive_path).unwrap();
        operations::list::list_archive(input, &archive_path, true, CompressionMode::None).unwrap();
    });
}

// ---------------------------------------------------------------------------
// Extract benchmarks
// ---------------------------------------------------------------------------

#[divan::bench]
fn extract_archive_20_files(bencher: divan::Bencher) {
    let source = TempDir::new().unwrap();
    create_sample_files(source.path(), 20, 4_096);
    let archive_dir = TempDir::new().unwrap();
    let archive_path = archive_dir.path().join("bench.tar");
    build_archive(&archive_path, source.path());

    let original_dir = std::env::current_dir().unwrap();
    bencher
        .with_inputs(|| TempDir::new().unwrap())
        .bench_local_values(|extract_dir| {
            std::env::set_current_dir(extract_dir.path()).unwrap();
            let input = File::open(&archive_path).unwrap();
            operations::extract::extract_archive(
                input,
                &archive_path,
                false,
                CompressionMode::None,
            )
            .unwrap();
        });
    std::env::set_current_dir(original_dir).unwrap();
}

// ---------------------------------------------------------------------------
// CLI parsing benchmark
// ---------------------------------------------------------------------------

#[divan::bench]
fn build_cli_command(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        divan::black_box(tar::uu_app());
    });
}
