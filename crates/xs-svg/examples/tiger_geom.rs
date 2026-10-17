// Baseline: how long one frame's geometry build costs for the tiger.
fn main() {
    use xs_layout::Color;
    use xs_svg::Svg;
    use std::time::Instant;

    static TIGER: Svg = Svg::new(include_str!(
        "/home/cbosse/Workspaces/rust/xs-ui/ressources/images/tiger.svg"
    ));
    // The cache lives in the app context now, exactly as a frame finds it.
    let ctx = xs_core::AppContext::default();
    let mut uncached = Vec::new();
    let mut cached = Vec::new();
    let mut runs = 30;
    while runs > 0 {
        // Every frame rebuilds the element tree, so the document's shapes are
        // cloned into a fresh node...
        let t = Instant::now();
        let vector = TIGER.vector(1253., 944.).tolerance(2.);
        let shapes = t.elapsed();
        // ...and layout asks the app's cache for the mesh.
        let t = Instant::now();
        let key = MeshKeyForBench {
            shapes: vector.shapes.clone(),
            view_box: vector.view_box,
            tolerance: vector.tolerance,
            size: xs_math::Vector2::new(1253., 944.),
        };
        let geometry = ctx
            .meshes
            .get_or_build(key, || vector.mesh(xs_math::Vector2::new(1253., 944.)));
        let mesh = t.elapsed();
        uncached.push(shapes);
        cached.push(mesh);
        std::hint::black_box(&geometry);
        runs -= 1;
    }
    uncached.sort();
    cached.sort();
    println!("SvgImage clone per frame: {:.3} ms", uncached[15].as_secs_f64() * 1e3);
    println!("cached mesh  per frame:   {:.3} ms", cached[15].as_secs_f64() * 1e3);
    println!("cache holds {} entries", ctx.meshes.len());

    // And what it would have cost with no cache at all.
    let mut cold = Vec::new();
    for _ in 0..30 {
        let vector = TIGER.vector(1253., 944.).tolerance(2.);
        let t = Instant::now();
        std::hint::black_box(vector.mesh(xs_math::Vector2::new(1253., 944.)));
        cold.push(t.elapsed());
    }
    cold.sort();
    println!("uncached mesh per frame:  {:.3} ms", cold[15].as_secs_f64() * 1e3);
}

/// The benchmark stands in for `Vector::layout`, which builds the key itself.
#[derive(PartialEq)]
struct MeshKeyForBench {
    shapes: Vec<xs_vector::Shape>,
    view_box: Option<xs_layout::Rectangle>,
    tolerance: Option<f32>,
    size: xs_math::Vector2,
}
}
