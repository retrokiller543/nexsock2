use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nexsock_protocol_core::constants::HEADER_SIZE;
#[cfg(feature = "simd")]
use nexsock_protocol_core::header::optimized::OptimizedHeaderParser;
#[cfg(all(feature = "simd", target_arch = "aarch64"))]
use nexsock_protocol_core::header::simd::Aarch64NeonHeaderParser;
use nexsock_protocol_core::header::standard::StandardHeaderParser;
use nexsock_protocol_core::header::Header;
use nexsock_protocol_core::message_flags::MessageFlags;
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
const SAMPLE_SIZE: usize = 10000;
const NOISE_THRESHOLD: f64 = 0.04;

// Create more realistic test data with varying properties
fn create_test_headers() -> Vec<Header> {
    vec![
        // Small payload header
        Header::new(
            5,
            1,
            MessageFlags::NONE,
            128,
            42
        ),
        // Medium payload header with flags
        Header::new(
            10,
            2,
            MessageFlags::COMPRESSED | MessageFlags::HAS_PAYLOAD,
            0x1000,
            0x1234_5678
        ),
        // Large payload header with all flags
        Header::new(
            63, // Max value for 6 bits
            3,
            MessageFlags::COMPRESSED | MessageFlags::ENCRYPTED |
                MessageFlags::REQUIRES_ACK | MessageFlags::HAS_PAYLOAD,
            0x1000_0000,
            0xFFFF_FFFF_FFFF_FFFF
        )
    ]
}

pub fn header_from_byte_parsing_benchmark(c: &mut Criterion) {
    let headers = create_test_headers();
    let mut group = c.benchmark_group("Header Deserialization");

    // Configure the benchmark
    group.sample_size(SAMPLE_SIZE)
        .noise_threshold(NOISE_THRESHOLD)
        .measurement_time(std::time::Duration::from_secs(5));

    for (i, header) in headers.iter().enumerate() {
        let header_bytes = header.to_bytes::<StandardHeaderParser>();
        let header_size = header_bytes.len() as u64;

        // Set throughput to measure bytes processed per second
        group.throughput(Throughput::Bytes(header_size));
        
        group.bench_with_input(
            BenchmarkId::new("Optimized", format!("case_{}", i)),
            &header_bytes,
            |b, bytes| {
                b.iter(|| {
                    black_box(Header::parse::<OptimizedHeaderParser>(black_box(bytes)))
                })
            }
        );

        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        group.bench_with_input(
            BenchmarkId::new("Aarch64 Neon", format!("case_{}", i)),
            &header_bytes,
            |b, bytes| {
                b.iter(|| {
                    black_box(Header::parse::<Aarch64NeonHeaderParser>(black_box(bytes)))
                })
            }
        );

        // Standard parser
        group.bench_with_input(
            BenchmarkId::new("Standard", format!("case_{}", i)),
            &header_bytes,
            |b, bytes| {
                b.iter(|| {
                    black_box(Header::parse::<StandardHeaderParser>(black_box(bytes)))
                })
            }
        );
    }

    group.finish();
}

pub fn header_to_byte_conversion_benchmark(c: &mut Criterion) {
    let headers = create_test_headers();
    let mut group = c.benchmark_group("Header Serialization");

    // Configure the benchmark
    group.sample_size(SAMPLE_SIZE)
        .noise_threshold(NOISE_THRESHOLD)
        .measurement_time(std::time::Duration::from_secs(5))
        .throughput(Throughput::Bytes(HEADER_SIZE as u64));

    for (i, header) in headers.iter().enumerate() {
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        group.bench_with_input(
            BenchmarkId::new("Aarch64 Neon", format!("case_{}", i)),
            header,
            |b, header| {
                b.iter(|| {
                    black_box(black_box(header).to_bytes::<Aarch64NeonHeaderParser>())
                })
            }
        );

        group.bench_with_input(
            BenchmarkId::new("Standard", format!("case_{}", i)),
            header,
            |b, header| {
                b.iter(|| {
                    black_box(black_box(header).to_bytes::<StandardHeaderParser>())
                })
            }
        );
    }

    group.finish();
}

// Add a new benchmark for bytes-to-bytes roundtrip
pub fn header_roundtrip_benchmark(c: &mut Criterion) {
    let headers = create_test_headers();
    let mut group = c.benchmark_group("Header Roundtrip");

    group.sample_size(SAMPLE_SIZE)
        .noise_threshold(NOISE_THRESHOLD)
        .measurement_time(std::time::Duration::from_secs(5))
        .throughput(Throughput::Bytes((HEADER_SIZE * 2) as u64));

    for (i, header) in headers.iter().enumerate() {
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        group.bench_with_input(
            BenchmarkId::new("Aarch64 Neon", format!("case_{}", i)),
            header,
            |b, header| {
                b.iter(|| {
                    let bytes = black_box(header).to_bytes::<Aarch64NeonHeaderParser>();
                    black_box(Header::parse::<Aarch64NeonHeaderParser>(black_box(&bytes)))
                })
            }
        );
        
        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        group.bench_with_input(
            BenchmarkId::new("Optimized + Aarch64 Neon", format!("case_{}", i)),
            header,
            |b, header| {
                b.iter(|| {
                    let bytes = black_box(header).to_bytes::<Aarch64NeonHeaderParser>();
                    black_box(Header::parse::<OptimizedHeaderParser>(black_box(&bytes)))
                })
            }
        );

        #[cfg(all(feature = "simd", target_arch = "aarch64"))]
        group.bench_with_input(
            BenchmarkId::new("Standard + Aarch64 Neon", format!("case_{}", i)),
            header,
            |b, header| {
                b.iter(|| {
                    let bytes = black_box(header).to_bytes::<Aarch64NeonHeaderParser>();
                    black_box(Header::parse::<StandardHeaderParser>(black_box(&bytes)))
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("Optimized + Standard", format!("case_{}", i)),
            header,
            |b, header| {
                b.iter(|| {
                    let bytes = black_box(header).to_bytes::<StandardHeaderParser>();
                    black_box(Header::parse::<OptimizedHeaderParser>(black_box(&bytes)))
                })
            }
        );

        // Standard roundtrip
        group.bench_with_input(
            BenchmarkId::new("Standard", format!("case_{}", i)),
            header,
            |b, header| {
                b.iter(|| {
                    let bytes = black_box(header).to_bytes::<StandardHeaderParser>();
                    black_box(Header::parse::<StandardHeaderParser>(black_box(&bytes)))
                })
            }
        );
    }

    group.finish();
}

// Run multiple benchmark types with different test cases
criterion_group!(
    benches,
    header_to_byte_conversion_benchmark,
    header_from_byte_parsing_benchmark,
    header_roundtrip_benchmark
);
criterion_main!(benches);