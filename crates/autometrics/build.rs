use cfg_aliases::cfg_aliases;

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    cfg_aliases! {
      // Backends
      metrics: { feature = "metrics-0_23" },

      // Custom objectives
      custom_objective_percentile: { feature = "custom-objective-percentile" },
      custom_objective_latency: { feature = "custom-objective-latency" },
    }
}
