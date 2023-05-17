#[cfg(test)]
mod test_end_to_end {
    use std::time::Instant;

    use graph_walker::{hyper_params::HyperParams, universe::Universe, universe::Universe2D};

    #[test]
    fn performance_test_tick() {
        let mut universe = Universe2D::new(100, 100000);
        universe.set_hyper_params(HyperParams::new(0.5, 0.5, 0.1));

        let start = Instant::now();

        for _ in 0..300 {
            universe.tick();
        }
        // 2.651681208s
        println!("{:?} \n{}", start.elapsed(), universe);
    }
}
