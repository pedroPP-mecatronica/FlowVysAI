//! # cavity2d — 2D Lid-Driven Cavity (D2Q9 LBM)
//!
//! Runs a 2D lid-driven cavity simulation using the D2Q9 Lattice Boltzmann
//! Method and writes velocity-field snapshots and centreline profiles to CSV.
//!
//! ## Usage
//! ```
//! cargo run --example cavity2d -- [OPTIONS]
//! ```
//!
//! ## Options (all optional, shown with defaults)
//! ```
//! --nx 64          Grid width  (includes boundary nodes)
//! --ny 64          Grid height (includes boundary nodes)
//! --re 100         Reynolds number  (mutually exclusive with --nu)
//! --nu 0.01        Kinematic viscosity (overrides --re if both given)
//! --steps 10000    Total time steps
//! --freq 1000      Output frequency (CSV snapshot every N steps; 0 = only final)
//! --out outputs    Output directory
//! --ulid 0.05      Lid velocity in lattice units (low Mach: ≪ cs ≈ 0.577)
//! ```
//!
//! ## Example
//! ```
//! cargo run --example cavity2d -- --nx 64 --ny 64 --re 100 --steps 10000 --freq 2000
//! ```
//!
//! ## Output files
//! - `<out>/velocity_<step>.csv` — full field: x, y, rho, ux, uy
//! - `<out>/centerline_<step>.csv` — centreline profiles: index, u_vert, v_horiz

use aerocore_core::solvers::lbm::lbm2d::LbmSolver2D;
use std::path::PathBuf;

fn main() {
    let cfg = Config::from_args();

    println!("=== FlowVysAI — 2D Lid-Driven Cavity (D2Q9 LBM) ===");
    println!(
        "Grid: {}×{}  Re: {:.1}  nu: {:.6}  U_lid: {:.4}  steps: {}",
        cfg.nx, cfg.ny, cfg.re, cfg.nu, cfg.u_lid, cfg.steps
    );
    println!("Output: {}", cfg.out_dir.display());

    // Create output directory.
    std::fs::create_dir_all(&cfg.out_dir).expect("Cannot create output directory");

    let mut solver = LbmSolver2D::new(cfg.nx, cfg.ny, cfg.nu);

    let initial_mass = solver.total_density();

    for step in 1..=cfg.steps {
        solver.step(cfg.u_lid);

        let should_output = cfg.freq > 0 && step % cfg.freq == 0 || step == cfg.steps;
        if should_output {
            let ke = solver.total_kinetic_energy();
            let mass = solver.total_density();
            let mass_err = (mass - initial_mass) / initial_mass;
            println!(
                "  step {:6}  KE={:.4e}  mass_err={:.2e}",
                step, ke, mass_err
            );

            // Write velocity field snapshot.
            let vel_path = cfg.out_dir.join(format!("velocity_{step:06}.csv"));
            solver.write_velocity_csv(&vel_path).unwrap_or_else(|e| {
                eprintln!("Warning: could not write {}: {e}", vel_path.display())
            });

            // Write centreline profiles.
            let cl_path = cfg.out_dir.join(format!("centerline_{step:06}.csv"));
            solver.write_centerline_csv(&cl_path).unwrap_or_else(|e| {
                eprintln!("Warning: could not write {}: {e}", cl_path.display())
            });
        }
    }

    println!("Done. Outputs written to: {}", cfg.out_dir.display());
}

// ── CLI configuration ────────────────────────────────────────────────────────

struct Config {
    nx: usize,
    ny: usize,
    re: f64,
    nu: f64,
    u_lid: f64,
    steps: u64,
    freq: u64,
    out_dir: PathBuf,
}

impl Config {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();

        let mut nx: usize = 64;
        let mut ny: usize = 64;
        let mut re: Option<f64> = None;
        let mut nu: Option<f64> = None;
        let mut u_lid: f64 = 0.05;
        let mut steps: u64 = 10_000;
        let mut freq: u64 = 1_000;
        let mut out_dir = PathBuf::from("outputs");

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--nx" => nx = parse_next(&args, &mut i, "--nx"),
                "--ny" => ny = parse_next(&args, &mut i, "--ny"),
                "--re" => re = Some(parse_next(&args, &mut i, "--re")),
                "--nu" => nu = Some(parse_next(&args, &mut i, "--nu")),
                "--ulid" => u_lid = parse_next(&args, &mut i, "--ulid"),
                "--steps" => steps = parse_next(&args, &mut i, "--steps"),
                "--freq" => freq = parse_next(&args, &mut i, "--freq"),
                "--out" => {
                    i += 1;
                    out_dir = PathBuf::from(
                        args.get(i)
                            .unwrap_or_else(|| panic!("Expected value after --out")),
                    );
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => eprintln!("Unknown argument: {other}"),
            }
            i += 1;
        }

        // Resolve viscosity: explicit --nu overrides --re.
        let nu_final = if let Some(n) = nu {
            n
        } else {
            let re_val = re.unwrap_or(100.0);
            // ν = U_lid × (nx − 2) / Re
            u_lid * (nx - 2) as f64 / re_val
        };

        let re_final = u_lid * (nx - 2) as f64 / nu_final;

        Config {
            nx,
            ny,
            re: re_final,
            nu: nu_final,
            u_lid,
            steps,
            freq,
            out_dir,
        }
    }
}

fn parse_next<T>(args: &[String], i: &mut usize, flag: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    *i += 1;
    args.get(*i)
        .unwrap_or_else(|| panic!("Expected value after {flag}"))
        .parse::<T>()
        .unwrap_or_else(|e| panic!("Invalid value for {flag}: {e}"))
}

fn print_usage() {
    println!(
        "cavity2d — 2D Lid-Driven Cavity (D2Q9 LBM)\n\n\
        USAGE:\n  cargo run --example cavity2d -- [OPTIONS]\n\n\
        OPTIONS:\n\
        \x20 --nx <N>      Grid width  [default: 64]\n\
        \x20 --ny <N>      Grid height [default: 64]\n\
        \x20 --re <R>      Reynolds number [default: 100]\n\
        \x20 --nu <N>      Kinematic viscosity (overrides --re)\n\
        \x20 --ulid <U>    Lid velocity [default: 0.05]\n\
        \x20 --steps <S>   Total time steps [default: 10000]\n\
        \x20 --freq <F>    Output every F steps [default: 1000]\n\
        \x20 --out <DIR>   Output directory [default: outputs]"
    );
}
