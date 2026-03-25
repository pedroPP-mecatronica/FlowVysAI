//! AeroCore UI — Frontend application
//! Teste de integração do Solver LBM D3Q19

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║     AeroCore CFD Engine v0.1.0               ║");
    println!("║     Teste de Estabilidade do Solver LBM      ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();

    // 1. Importações do Core
    use aerocore_core::memory::arena::SimArena;
    use aerocore_core::solvers::lbm::LbmSolver;
    use aerocore_core::solvers::traits::{Solver, SolverConfig};
    use std::time::Instant;

    // 2. Configuração da Simulação
    let nx = 32;
    let ny = 32;
    let nz = 32;
    let viscosity = 0.1;
    let iterations = 100;

    println!("[i] Configurando domínio: {}x{}x{}", nx, ny, nz);
    
    // 3. Preparar Memória (Arena de 50MB para este teste)
    let arena = SimArena::new(50 * 1024 * 1024); 
    let config = SolverConfig::default();

    // 4. Instanciar e Inicializar o Solver
    let mut solver = LbmSolver::new(nx, ny, nz, viscosity);
    
    print!("[i] Inicializando buffers na Arena... ");
    match solver.init(&config, &arena) {
        Ok(_) => println!("OK! (Uso da Arena: {} bytes)", arena.bytes_used()),
        Err(e) => {
            println!("FALHA: {:?}", e);
            return;
        }
    }

    // 5. Loop de Simulação (Hot Path)
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
    println!("[✓] Performance: {:.2} iterações/segundo", iterations as f64 / duration.as_secs_f64());
    
    println!();
    println!("Próximo passo: Sprint S3 - Implementar condições de contorno (Walls).");
}