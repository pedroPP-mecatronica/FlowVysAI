//! AeroCore UI — Frontend application and CLI entry point.
//!
//! # Usage
//!
//! Normal stability test (default):
//! ```text
//! cargo run -p aerocore_ui
//! ```
//!
//! 1000-step LBM benchmark (prints MLUPS):
//! ```text
//! cargo run -p aerocore_ui --release -- --bench
//! ```

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--bench") {
        run_benchmark();
    } else {
        run_stability_test();
    }
}

// ── benchmark ─────────────────────────────────────────────────────────────────

fn run_benchmark() {
    use aerocore_core::memory::arena::SimArena;
    use aerocore_core::solvers::lbm::LbmSolver;
    use aerocore_core::solvers::traits::{Solver, SolverConfig};
    use std::time::Instant;

    let nx: usize = 32;
    let ny: usize = 32;
    let nz: usize = 32;
    let viscosity: f64 = 0.1;
    let steps: u64 = 1_000;

    println!("╔══════════════════════════════════════════════╗");
    println!("║     AeroCore LBM Benchmark (--bench)         ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!(
        "[i] Grid       : {}×{}×{} ({} cells)",
        nx,
        ny,
        nz,
        nx * ny * nz
    );
    println!("[i] Steps      : {}", steps);
    println!("[i] Viscosity  : {}", viscosity);
    println!();

    let arena_bytes = 32 * nx * ny * nz * std::mem::size_of::<f64>() * 20;
    let arena = SimArena::new(arena_bytes);
    let config = SolverConfig::default();

    let mut solver: LbmSolver<f64> = LbmSolver::new(nx, ny, nz, viscosity);
    solver.init(&config, &arena).expect("LBM init failed");

    print!("[i] Warming up 1 step... ");
    solver.step().expect("warmup step failed");
    println!("done.");

    let cells = (nx * ny * nz) as f64;
    let start = Instant::now();

    for _ in 0..steps {
        solver.step().expect("LBM step failed");
    }

    let elapsed = start.elapsed();
    let mlups = cells * steps as f64 / elapsed.as_secs_f64() / 1_000_000.0;

    println!();
    println!("[✓] Elapsed    : {:.3?}", elapsed);
    println!("[✓] Performance: {:.2} MLUPS", mlups);
}

// ── stability test (original demo) ───────────────────────────────────────────

fn run_stability_test() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║     AeroCore CFD Engine v0.1.0               ║");
    println!("║     Teste de Estabilidade do Solver LBM      ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();

    use aerocore_core::memory::arena::SimArena;
    use aerocore_core::solvers::lbm::LbmSolver;
    use aerocore_core::solvers::traits::{Solver, SolverConfig};
    use std::time::Instant;

    let nx = 32;
    let ny = 32;
    let nz = 32;
    let viscosity = 0.1;
    let iterations = 100;

    println!("[i] Configurando domínio: {}x{}x{}", nx, ny, nz);

    let arena = SimArena::new(50 * 1024 * 1024);
    let config = SolverConfig::default();

    let mut solver = LbmSolver::new(nx, ny, nz, viscosity);

    print!("[i] Inicializando buffers na Arena... ");
    match solver.init(&config, &arena) {
        Ok(_) => println!("OK! (Uso da Arena: {} bytes)", arena.bytes_used()),
        Err(e) => {
            println!("FALHA: {:?}", e);
            return;
        }
    }

    println!("[i] Executando {} iterações...", iterations);
    let start = Instant::now();

    for i in 1..=iterations {
        match solver.step() {
            Ok(result) => {
                if i % 10 == 0 {
                    println!("    Passo {:>3} | t = {:.3}s", result.timestep, result.time);
                }
            }
            Err(e) => {
                println!("    [!] Erro na iteração {}: {:?}", i, e);
                break;
            }
        }
    }

    let duration = start.elapsed();
    println!();
    println!("[✓] Simulação finalizada com sucesso!");
    println!("[✓] Tempo total: {:?}", duration);
    println!(
        "[✓] Performance: {:.2} iterações/segundo",
        iterations as f64 / duration.as_secs_f64()
    );

    println!();
    println!("Próximo passo: experimente diferentes configurações de domínio e viscosidade.");
}
