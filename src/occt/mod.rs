#[cxx::bridge(namespace = "rrcad")]
mod ffi {
    unsafe extern "C++" {
        include!("bridge.h");

        type OcctShape;

        // --- Primitives ---
        fn make_box(dx: f64, dy: f64, dz: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_cylinder(radius: f64, height: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_sphere(radius: f64) -> Result<UniquePtr<OcctShape>>;

        // --- Boolean operations ---
        fn shape_fuse(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;
        fn shape_cut(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;
        fn shape_common(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;

        // --- Fillets and chamfers (applied to all edges) ---
        fn shape_fillet(shape: &OcctShape, radius: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_chamfer(shape: &OcctShape, dist: f64) -> Result<UniquePtr<OcctShape>>;

        // --- Transforms (immutable; return new shapes) ---
        fn shape_translate(
            shape: &OcctShape,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_rotate(
            shape: &OcctShape,
            axis_x: f64,
            axis_y: f64,
            axis_z: f64,
            angle_deg: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_scale(shape: &OcctShape, factor: f64) -> Result<UniquePtr<OcctShape>>;

        fn shape_mirror(shape: &OcctShape, plane: &str) -> Result<UniquePtr<OcctShape>>;

        fn make_rect(w: f64, h: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_circle_face(r: f64) -> Result<UniquePtr<OcctShape>>;

        fn shape_extrude(shape: &OcctShape, height: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_revolve(shape: &OcctShape, angle_deg: f64) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 3: splines and sweep ---
        fn make_spline_2d(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;
        fn make_spline_3d(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;
        fn shape_sweep(profile: &OcctShape, path: &OcctShape) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 3: sub-shape selectors ---
        fn shape_faces_count(shape: &OcctShape, selector: &str) -> Result<i32>;
        fn shape_faces_get(
            shape: &OcctShape,
            selector: &str,
            idx: i32,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_edges_count(shape: &OcctShape, selector: &str) -> Result<i32>;
        fn shape_edges_get(
            shape: &OcctShape,
            selector: &str,
            idx: i32,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Export ---
        fn export_step(shape: &OcctShape, path: &str) -> Result<()>;
        fn export_stl(shape: &OcctShape, path: &str) -> Result<()>;
        fn export_gltf(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
        fn export_glb(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
    }
}

use cxx::UniquePtr;

/// Owned handle to a live OCCT shape.
pub struct Shape {
    inner: UniquePtr<ffi::OcctShape>,
}

impl Shape {
    // --- Constructors ---

    pub fn make_box(dx: f64, dy: f64, dz: f64) -> Result<Self, String> {
        ffi::make_box(dx, dy, dz)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_cylinder(radius: f64, height: f64) -> Result<Self, String> {
        ffi::make_cylinder(radius, height)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_sphere(radius: f64) -> Result<Self, String> {
        ffi::make_sphere(radius)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    // --- Boolean operations ---

    pub fn fuse(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_fuse(&self.inner, &other.inner)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn cut(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_cut(&self.inner, &other.inner)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn common(&self, other: &Shape) -> Result<Shape, String> {
        ffi::shape_common(&self.inner, &other.inner)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    // --- Fillets and chamfers ---

    pub fn fillet(&self, radius: f64) -> Result<Shape, String> {
        ffi::shape_fillet(&self.inner, radius)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn chamfer(&self, dist: f64) -> Result<Shape, String> {
        ffi::shape_chamfer(&self.inner, dist)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    // --- Transforms ---

    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Result<Shape, String> {
        ffi::shape_translate(&self.inner, dx, dy, dz)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn rotate(&self, ax: f64, ay: f64, az: f64, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_rotate(&self.inner, ax, ay, az, angle_deg)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn scale(&self, factor: f64) -> Result<Shape, String> {
        ffi::shape_scale(&self.inner, factor)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn mirror(&self, plane: &str) -> Result<Shape, String> {
        ffi::shape_mirror(&self.inner, plane)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_rect(w: f64, h: f64) -> Result<Self, String> {
        ffi::make_rect(w, h)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_circle_face(r: f64) -> Result<Self, String> {
        ffi::make_circle_face(r)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn extrude(&self, height: f64) -> Result<Shape, String> {
        ffi::shape_extrude(&self.inner, height)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn revolve(&self, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_revolve(&self.inner, angle_deg)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    // --- Phase 3: splines and sweep ---

    pub fn make_spline_2d(pts: &[f64]) -> Result<Self, String> {
        ffi::make_spline_2d(pts)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_spline_3d(pts: &[f64]) -> Result<Self, String> {
        ffi::make_spline_3d(pts)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn sweep(&self, path: &Shape) -> Result<Shape, String> {
        ffi::shape_sweep(&self.inner, &path.inner)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    // --- Phase 3: sub-shape selectors ---

    pub fn faces(&self, selector: &str) -> Result<Vec<Shape>, String> {
        let n = ffi::shape_faces_count(&self.inner, selector).map_err(|e| e.to_string())?;
        (0..n)
            .map(|i| {
                ffi::shape_faces_get(&self.inner, selector, i)
                    .map(|p| Shape { inner: p })
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    pub fn edges(&self, selector: &str) -> Result<Vec<Shape>, String> {
        let n = ffi::shape_edges_count(&self.inner, selector).map_err(|e| e.to_string())?;
        (0..n)
            .map(|i| {
                ffi::shape_edges_get(&self.inner, selector, i)
                    .map(|p| Shape { inner: p })
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    // --- Export ---

    pub fn export_step(&self, path: &str) -> Result<(), String> {
        ffi::export_step(&self.inner, path).map_err(|e| e.to_string())
    }

    pub fn export_stl(&self, path: &str) -> Result<(), String> {
        ffi::export_stl(&self.inner, path).map_err(|e| e.to_string())
    }

    /// Export to glTF. `linear_deflection` controls tessellation quality (e.g. `0.1` for 0.1 mm).
    pub fn export_gltf(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_gltf(&self.inner, path, linear_deflection).map_err(|e| e.to_string())
    }

    /// Export to binary glTF (GLB). Single-file format suitable for HTTP serving.
    pub fn export_glb(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_glb(&self.inner, path, linear_deflection).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Shape;

    #[test]
    fn smoke_filleted_box_to_step() {
        let shape = Shape::make_box(10.0, 20.0, 30.0).expect("make_box failed");
        let filleted = shape.fillet(2.0).expect("fillet failed");

        let out = std::env::temp_dir().join("rrcad_smoke_filleted_box.step");
        filleted
            .export_step(out.to_str().unwrap())
            .expect("export_step failed");

        assert!(out.exists(), "STEP file was not created");
        assert!(
            std::fs::metadata(&out).unwrap().len() > 0,
            "STEP file is empty"
        );
        let content = std::fs::read_to_string(&out).unwrap();
        assert!(
            content.contains("ISO-10303-21"),
            "output does not look like a STEP file"
        );
    }

    #[test]
    fn smoke_boolean_cut() {
        let base = Shape::make_box(20.0, 20.0, 20.0).unwrap();
        let cyl = Shape::make_cylinder(5.0, 25.0).unwrap();
        let result = base.cut(&cyl).expect("boolean cut failed");

        let out = std::env::temp_dir().join("rrcad_smoke_cut.step");
        result.export_step(out.to_str().unwrap()).unwrap();
        assert!(out.exists());
    }
}
