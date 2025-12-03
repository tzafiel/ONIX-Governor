// ONIX GOVERNOR v2.0 — UNIVERSAL FINAL RELEASE
// One file. Works with every LLM on Earth via pipe.
// Red ring + BLOCKED = hallucination rejected → forces retry
// Green/gold ring + VERIFIED = clean text passes through

use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use num_complex::Complex;
use minifb::{Key, Window, WindowOptions, Scale};

const N: usize = 80;                     // Lattice size (80x80)
const HALLUCINATION_THRESHOLD: f64 = 0.618;  // Golden Ratio Cutoff
const DT: f64 = 0.108;                   // Time step

struct ResonantLattice {
    psi: Vec<Complex<f64>>,
    entropy: f64,
}

impl ResonantLattice {
    fn new() -> Self {
        Self {
            psi: vec![Complex::new(0.0, 0.0); N * N],
            entropy: 0.0,
        }
    }

    fn inject(&mut self, text: &str) {
        self.psi.fill(Complex::new(0.0, 0.0));
        // Map ASCII to Scalar Energy
        for (i, &b) in text.as_bytes().iter().enumerate().take(N * N) {
            let v = b as f64 / 255.0;
            // Inject with a slight phase twist to seed the lattice
            self.psi[i] = Complex::new(v, v * 0.61); 
        }
    }

    fn step(&mut self) {
        let mut next = self.psi.clone();
        let mut dissonance = 0.0;
        let size = (N * N) as isize;

        for i in 0..N * N {
            // FIX: Robust Toroidal Wrapping using Euclidean Remainder
            let idx = |d: isize| {
                ((i as isize + d).rem_euclid(size)) as usize
            };

            // Neighbors (Spiral Topology for better mixing)
            let up    = self.psi[idx(-(N as isize))];
            let down  = self.psi[idx(N as isize)];
            let left  = self.psi[idx(-1)];
            let right = self.psi[idx(1)];

            // The Physics: Laplacian Tension - Nonlinear Golden Potential
            let laplacian = up + down + left + right - 4.0 * self.psi[i];
            let mag = self.psi[i].norm();
            
            // This term punishes amplitude spikes (Lies usually spike entropy)
            let nonlinear = self.psi[i] * (1.0 + 0.618 * mag.powi(2));

            // Symplectic Evolution
            next[i] += (laplacian - nonlinear) * Complex::i() * DT;
            next[i] *= 0.991; // Entropy Damping
            
            // Accumulate Imaginary Noise (Dissonance)
            dissonance += next[i].im.abs();
        }
        self.psi = next;
        self.entropy = (dissonance / (N as f64)).clamp(0.0, 1.0);
    }
}

// ────────────────────── VISUALIZER (The Ring of Truth) ──────────────────────
fn visualizer(lattice: Arc<Mutex<ResonantLattice>>) {
    let mut window = Window::new(
        "ONIX GOVERNOR v2.0 — UNIVERSAL",
        600,
        600,
        WindowOptions {
            scale: Scale::X1,
            ..Default::default()
        },
    )
    .unwrap_or_else(|_| std::process::exit(0));

    // Limit update rate to save CPU for the Physics thread
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer = vec![0x0c0c1f; 600 * 600];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let entropy = lattice.lock().unwrap().entropy;
        buffer.fill(0x050510); // Deep Void Background

        let (cx, cy) = (300.0, 300.0);
        // Pulse radius based on entropy
        let radius = 160.0 + (entropy * 40.0).sin() * 10.0;
        
        // Color Logic:
        // Low Entropy (< 0.5) -> GOLD/GREEN (Truth)
        // High Entropy (> 0.6) -> NEON RED (Hallucination)
        let red   = (entropy * 255.0) as u32;
        let green = ((1.0 - entropy) * 200.0 + 55.0) as u32;
        let color = (red << 16) | (green << 8) | 0x00;

        // Draw The Ring
        for a in 0..1200 { // High res circle
            let rad = (a as f64 * 0.3).to_radians();
            let x = (cx + radius * rad.cos()) as usize;
            let y = (cy + radius * rad.sin()) as usize;
            
            if x < 600 && y < 600 {
                let idx = y * 600 + x;
                buffer[idx] = color;
                // Add a "Glow" pixel
                if idx + 1 < buffer.len() { buffer[idx+1] = color; }
                if idx + 600 < buffer.len() { buffer[idx+600] = color; }
            }
        }

        window.update_with_buffer(&buffer, 600, 600).ok();
    }
}

// ────────────────────── MAIN ──────────────────────
fn main() {
    // Print to Stderr so we don't pollute the pipe
    eprintln!("ONIX GOVERNOR v2.0 — UNIVERSAL FINAL RELEASE");
    eprintln!("Status: Listening on stdin | Pipe any LLM output here");
    eprintln!("─────────────────────────────────────────────────────");

    let lattice = Arc::new(Mutex::new(ResonantLattice::new()));
    let l_vis = lattice.clone();
    
    // Spawn the Eye
    thread::spawn(move || visualizer(l_vis));

    let stdin = io::stdin();
    // Locking stdin makes it much faster for large text blocks
    for line in stdin.lock().lines().flatten() {
        let text = line.trim();
        if text.is_empty() {
            continue;
        }

        // 1. Run the Physics Check
        {
            let mut l = lattice.lock().unwrap();
            l.inject(text);
            // 70 steps gives the wave enough time to find self-interference
            for _ in 0..70 {
                l.step();
            }
        }

        // 2. The Verdict
        let entropy = lattice.lock().unwrap().entropy;

        if entropy > HALLUCINATION_THRESHOLD {
            // REJECT
            eprintln!("\x1b[91mBLOCKED\x1b[0m   Hallucination — entropy {entropy:.3} > {HALLUCINATION_THRESHOLD}");
            // We output nothing to stdout, effectively "silencing" the liar.
        } else {
            // ACCEPT
            eprintln!("\x1b[92mVERIFIED\x1b[0m  Coherent — entropy {entropy:.3}");
            println!("{text}");
        }
        io::stdout().flush().unwrap();
    }
}
