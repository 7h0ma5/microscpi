use criterion::{black_box, criterion_group, criterion_main, Criterion};
use microscpi::Interface;

// Example benchmark for the `system_value` function
pub struct ExampleInterface {
    value: u64,
}

impl microscpi::ErrorHandler for ExampleInterface {
    fn handle_error(&mut self, error: microscpi::Error) {
        println!("Error: {error}");
    }
}

#[microscpi::interface]
impl ExampleInterface {
    #[scpi(cmd = "SYSTem:VALue?")]
    async fn system_value(&mut self) -> Result<u64, microscpi::Error> {
        Ok(self.value)
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut interface = ExampleInterface { value: 42 };
    let mut output = String::new();
    c.bench_function("system_value", |b| {
        b.iter(|| {
            let _ = black_box(interface.run(b"SYSTEM:VAL?\n", &mut output));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
