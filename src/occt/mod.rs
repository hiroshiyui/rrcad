#[cxx::bridge(namespace = "rrcad")]
mod ffi {
    unsafe extern "C++" {
        include!("bridge.h");

        type OcctShape;

        // --- Color ---
        fn shape_set_color(
            shape: &OcctShape,
            r: f64,
            g: f64,
            b: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Assembly mating ---
        fn shape_mate(
            shape: &OcctShape,
            from_face: &OcctShape,
            to_face: &OcctShape,
            offset: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Primitives ---
        fn make_box(dx: f64, dy: f64, dz: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_cylinder(radius: f64, height: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_sphere(radius: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_cone(r1: f64, r2: f64, height: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_torus(r1: f64, r2: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_wedge(dx: f64, dy: f64, dz: f64, ltx: f64) -> Result<UniquePtr<OcctShape>>;

        // --- Boolean operations ---
        fn shape_fuse(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;
        fn shape_cut(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;
        fn shape_common(a: &OcctShape, b: &OcctShape) -> Result<UniquePtr<OcctShape>>;

        // --- Fillets and chamfers ---
        fn shape_fillet(shape: &OcctShape, radius: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_chamfer(shape: &OcctShape, dist: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_fillet_sel(
            shape: &OcctShape,
            radius: f64,
            selector: &str,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_chamfer_sel(
            shape: &OcctShape,
            dist: f64,
            selector: &str,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_fillet_var(shape: &OcctShape, r1: f64, r2: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_fillet_var_sel(
            shape: &OcctShape,
            r1: f64,
            r2: f64,
            selector: &str,
        ) -> Result<UniquePtr<OcctShape>>;

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
        fn shape_scale_xyz(
            shape: &OcctShape,
            sx: f64,
            sy: f64,
            sz: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        fn shape_mirror(shape: &OcctShape, plane: &str) -> Result<UniquePtr<OcctShape>>;

        fn make_rect(w: f64, h: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_circle_face(r: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_polygon(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;
        fn make_ellipse_face(rx: f64, ry: f64) -> Result<UniquePtr<OcctShape>>;
        fn make_arc(r: f64, start_deg: f64, end_deg: f64) -> Result<UniquePtr<OcctShape>>;

        fn shape_extrude(shape: &OcctShape, height: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_revolve(shape: &OcctShape, angle_deg: f64) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 4: ThruSections (loft) builder ---
        type ThruSectionsBuilder;
        fn thru_sections_new(solid: bool, ruled: bool) -> Result<UniquePtr<ThruSectionsBuilder>>;
        fn thru_sections_add(
            builder: Pin<&mut ThruSectionsBuilder>,
            profile: &OcctShape,
        ) -> Result<()>;
        fn thru_sections_build(
            builder: Pin<&mut ThruSectionsBuilder>,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 3: PipeShellBuilder (variable-section sweep) ---
        type PipeShellBuilder;
        fn pipe_shell_new(path: &OcctShape) -> Result<UniquePtr<PipeShellBuilder>>;
        fn pipe_shell_add(builder: Pin<&mut PipeShellBuilder>, profile: &OcctShape) -> Result<()>;
        fn pipe_shell_build(builder: Pin<&mut PipeShellBuilder>) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 4: 3-D operations ---
        fn shape_shell(shape: &OcctShape, thickness: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_offset(shape: &OcctShape, distance: f64) -> Result<UniquePtr<OcctShape>>;
        fn shape_simplify(shape: &OcctShape, min_feature_size: f64)
            -> Result<UniquePtr<OcctShape>>;
        fn shape_extrude_ex(
            shape: &OcctShape,
            height: f64,
            twist_deg: f64,
            scale: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Phase 3: splines and sweep ---
        fn make_spline_2d(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;
        fn make_spline_3d(pts: &[f64]) -> Result<UniquePtr<OcctShape>>;
        // Tangent-constrained variants (Phase 4 / Tier 4 quality improvement).
        fn make_spline_2d_tan(
            pts: &[f64],
            t0x: f64,
            t0z: f64,
            t1x: f64,
            t1z: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn make_spline_3d_tan(
            pts: &[f64],
            t0x: f64,
            t0y: f64,
            t0z: f64,
            t1x: f64,
            t1y: f64,
            t1z: f64,
        ) -> Result<UniquePtr<OcctShape>>;
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
        fn shape_vertices_count(shape: &OcctShape, selector: &str) -> Result<i32>;
        fn shape_vertices_get(
            shape: &OcctShape,
            selector: &str,
            idx: i32,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Patterns ---
        fn shape_linear_pattern(
            shape: &OcctShape,
            n: i32,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> Result<UniquePtr<OcctShape>>;
        fn shape_polar_pattern(
            shape: &OcctShape,
            n: i32,
            angle_deg: f64,
        ) -> Result<UniquePtr<OcctShape>>;

        // --- Import ---
        fn import_step(path: &str) -> Result<UniquePtr<OcctShape>>;
        fn import_stl(path: &str) -> Result<UniquePtr<OcctShape>>;

        // --- Query / introspection ---
        fn shape_bounding_box(shape: &OcctShape, out: &mut [f64]) -> Result<()>;
        fn shape_volume(shape: &OcctShape) -> Result<f64>;
        fn shape_surface_area(shape: &OcctShape) -> Result<f64>;

        // --- Export ---
        fn export_step(shape: &OcctShape, path: &str) -> Result<()>;
        fn export_stl(shape: &OcctShape, path: &str) -> Result<()>;
        fn export_gltf(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
        fn export_glb(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
        fn export_obj(shape: &OcctShape, path: &str, linear_deflection: f64) -> Result<()>;
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

    pub fn make_cone(r1: f64, r2: f64, height: f64) -> Result<Self, String> {
        ffi::make_cone(r1, r2, height)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_torus(r1: f64, r2: f64) -> Result<Self, String> {
        ffi::make_torus(r1, r2)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_wedge(dx: f64, dy: f64, dz: f64, ltx: f64) -> Result<Self, String> {
        ffi::make_wedge(dx, dy, dz, ltx)
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

    /// Fillet only edges matching `selector` (`:all` / `:vertical` / `:horizontal`).
    pub fn fillet_sel(&self, radius: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_fillet_sel(&self.inner, radius, selector)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Chamfer only edges matching `selector` (`:all` / `:vertical` / `:horizontal`).
    pub fn chamfer_sel(&self, dist: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_chamfer_sel(&self.inner, dist, selector)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Variable-radius fillet on all edges: radius transitions from `r1` at one
    /// end-vertex of each edge to `r2` at the other.
    pub fn fillet_var(&self, r1: f64, r2: f64) -> Result<Shape, String> {
        ffi::shape_fillet_var(&self.inner, r1, r2)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Variable-radius fillet on edges matching `selector`.
    pub fn fillet_var_sel(&self, r1: f64, r2: f64, selector: &str) -> Result<Shape, String> {
        ffi::shape_fillet_var_sel(&self.inner, r1, r2, selector)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    // --- Color ---

    /// Return a copy of `self` rigidly transformed so that `from_face` (a planar
    /// face of this shape) lies flush against `to_face` (a fixed reference face).
    ///
    /// The transform aligns face centroids and makes the outward normals antiparallel.
    /// `offset > 0` leaves a gap; `offset < 0` creates interference.
    pub fn mate(&self, from_face: &Shape, to_face: &Shape, offset: f64) -> Result<Shape, String> {
        ffi::shape_mate(&self.inner, &from_face.inner, &to_face.inner, offset)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Return a copy of this shape with an sRGB surface color attached.
    /// `r`, `g`, `b` are each in [0.0, 1.0].  The color is written into
    /// the XDE document during GLB / glTF / OBJ export.
    pub fn set_color(&self, r: f64, g: f64, b: f64) -> Result<Shape, String> {
        ffi::shape_set_color(&self.inner, r, g, b)
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

    /// Non-uniform scale with independent factors for each axis.
    pub fn scale_xyz(&self, sx: f64, sy: f64, sz: f64) -> Result<Shape, String> {
        ffi::shape_scale_xyz(&self.inner, sx, sy, sz)
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

    pub fn make_polygon(pts: &[f64]) -> Result<Self, String> {
        ffi::make_polygon(pts)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_ellipse_face(rx: f64, ry: f64) -> Result<Self, String> {
        ffi::make_ellipse_face(rx, ry)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn make_arc(r: f64, start_deg: f64, end_deg: f64) -> Result<Self, String> {
        ffi::make_arc(r, start_deg, end_deg)
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

    // --- Phase 4: loft (ThruSections builder pattern) ---

    /// Loft through a sequence of planar profiles (Faces or Wires).
    /// `ruled=true` produces a ruled surface (straight lines between sections).
    pub fn loft(profiles: &[&Shape], ruled: bool) -> Result<Shape, String> {
        let mut builder = ffi::thru_sections_new(true, ruled).map_err(|e| e.to_string())?;
        for p in profiles {
            ffi::thru_sections_add(builder.pin_mut(), &p.inner).map_err(|e| e.to_string())?;
        }
        ffi::thru_sections_build(builder.pin_mut())
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    // --- Phase 4: 3-D operations ---

    /// Hollow out the solid, removing the topmost face and offsetting inward
    /// by `thickness`.  Wraps BRepOffsetAPI_MakeThickSolid::MakeThickSolidByJoin.
    pub fn shell(&self, thickness: f64) -> Result<Shape, String> {
        ffi::shape_shell(&self.inner, thickness)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Inflate (positive) or deflate (negative) the solid uniformly.
    /// Wraps BRepOffsetAPI_MakeOffsetShape::PerformByJoin.
    pub fn offset(&self, distance: f64) -> Result<Shape, String> {
        ffi::shape_offset(&self.inner, distance)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Remove small holes and fillets.  Faces with area < min_feature_size²
    /// are passed to BRepAlgoAPI_Defeaturing.  Returns the original shape
    /// unchanged if no faces qualify.
    pub fn simplify(&self, min_feature_size: f64) -> Result<Shape, String> {
        ffi::shape_simplify(&self.inner, min_feature_size)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Extrude with optional end-twist (degrees) and end-scale.
    /// Falls back to MakePrism for the zero-twist/unity-scale case.
    pub fn extrude_ex(&self, height: f64, twist_deg: f64, scale: f64) -> Result<Shape, String> {
        ffi::shape_extrude_ex(&self.inner, height, twist_deg, scale)
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

    /// Like `make_spline_2d` but with explicit start/end tangent vectors in
    /// the XZ plane — suppresses natural-boundary oscillation on short splines.
    pub fn make_spline_2d_tan(
        pts: &[f64],
        t0x: f64,
        t0z: f64,
        t1x: f64,
        t1z: f64,
    ) -> Result<Self, String> {
        ffi::make_spline_2d_tan(pts, t0x, t0z, t1x, t1z)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Like `make_spline_3d` but with explicit start/end tangent vectors —
    /// suppresses natural-boundary oscillation on short splines.
    pub fn make_spline_3d_tan(
        pts: &[f64],
        t0x: f64,
        t0y: f64,
        t0z: f64,
        t1x: f64,
        t1y: f64,
        t1z: f64,
    ) -> Result<Self, String> {
        ffi::make_spline_3d_tan(pts, t0x, t0y, t0z, t1x, t1y, t1z)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn sweep(&self, path: &Shape) -> Result<Shape, String> {
        ffi::shape_sweep(&self.inner, &path.inner)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// Variable-section pipe sweep using BRepOffsetAPI_MakePipeShell.
    /// `path` is a Wire (from `spline_3d`); `profiles` are Faces, Wires, or
    /// Vertices distributed along the spine (first at start, last at end).
    /// Requires at least 2 profiles.
    pub fn sweep_sections(profiles: &[&Shape], path: &Shape) -> Result<Shape, String> {
        if profiles.len() < 2 {
            return Err("sweep_sections requires at least 2 profiles".to_string());
        }
        let mut builder = ffi::pipe_shell_new(&path.inner).map_err(|e| e.to_string())?;
        for p in profiles {
            ffi::pipe_shell_add(builder.pin_mut(), &p.inner).map_err(|e| e.to_string())?;
        }
        ffi::pipe_shell_build(builder.pin_mut())
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

    /// Returns all unique vertices matching the selector (currently only `"all"`).
    pub fn vertices(&self, selector: &str) -> Result<Vec<Shape>, String> {
        let n = ffi::shape_vertices_count(&self.inner, selector).map_err(|e| e.to_string())?;
        (0..n)
            .map(|i| {
                ffi::shape_vertices_get(&self.inner, selector, i)
                    .map(|p| Shape { inner: p })
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    // --- Query / introspection ---

    /// Returns `[xmin, ymin, zmin, xmax, ymax, zmax]`.
    pub fn bounding_box(&self) -> Result<[f64; 6], String> {
        let mut out = [0f64; 6];
        ffi::shape_bounding_box(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    pub fn volume(&self) -> Result<f64, String> {
        ffi::shape_volume(&self.inner).map_err(|e| e.to_string())
    }

    pub fn surface_area(&self) -> Result<f64, String> {
        ffi::shape_surface_area(&self.inner).map_err(|e| e.to_string())
    }

    // --- Patterns ---

    /// `n` translated copies of the shape at positions `i * [dx, dy, dz]` (i = 0..n-1).
    /// Returns a `TopoDS_Compound` — fuse explicitly if a merged solid is needed.
    pub fn linear_pattern(&self, n: i32, dx: f64, dy: f64, dz: f64) -> Result<Shape, String> {
        ffi::shape_linear_pattern(&self.inner, n, dx, dy, dz)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    /// `n` copies rotated around Z by `i * (angle_deg / n)` (i = 0..n-1).
    /// Returns a `TopoDS_Compound`.
    pub fn polar_pattern(&self, n: i32, angle_deg: f64) -> Result<Shape, String> {
        ffi::shape_polar_pattern(&self.inner, n, angle_deg)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    // --- Import ---

    pub fn import_step(path: &str) -> Result<Self, String> {
        ffi::import_step(path)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
    }

    pub fn import_stl(path: &str) -> Result<Self, String> {
        ffi::import_stl(path)
            .map(|p| Shape { inner: p })
            .map_err(|e| e.to_string())
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

    /// Export to Wavefront OBJ. Tessellates with `linear_deflection` and writes
    /// the `.obj` file plus a companion `.mtl` material file in the same directory.
    pub fn export_obj(&self, path: &str, linear_deflection: f64) -> Result<(), String> {
        ffi::export_obj(&self.inner, path, linear_deflection).map_err(|e| e.to_string())
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

    #[test]
    fn fillet_sel_vertical_only() {
        // A box has 4 vertical + 8 horizontal edges.
        // Filleting only :vertical edges should succeed and produce more edges
        // than the unfilleted box (each rounded edge becomes an arc).
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let original_edge_count = b.edges("all").unwrap().len();
        let filleted = b
            .fillet_sel(1.0, "vertical")
            .expect("fillet_sel vertical failed");
        let new_edge_count = filleted.edges("all").unwrap().len();
        assert!(
            new_edge_count > original_edge_count,
            "expected more edges after selective fillet, got {new_edge_count} vs {original_edge_count}"
        );
    }

    #[test]
    fn chamfer_sel_horizontal_only() {
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let original_edge_count = b.edges("all").unwrap().len();
        let chamfered = b
            .chamfer_sel(1.0, "horizontal")
            .expect("chamfer_sel horizontal failed");
        let new_edge_count = chamfered.edges("all").unwrap().len();
        assert!(
            new_edge_count > original_edge_count,
            "expected more edges after selective chamfer, got {new_edge_count} vs {original_edge_count}"
        );
    }

    #[test]
    fn fillet_var_all_edges_produces_valid_shape() {
        // Variable-radius fillet: r=0.5 at one vertex, r=2.0 at the other.
        // A box(10,10,10) has 12 edges; after filleting all, the edge count rises.
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let original_edge_count = b.edges("all").unwrap().len();
        let filleted = b.fillet_var(0.5, 2.0).expect("fillet_var failed");
        let new_edge_count = filleted.edges("all").unwrap().len();
        assert!(
            new_edge_count > original_edge_count,
            "expected more edges after variable-radius fillet, got {new_edge_count} vs {original_edge_count}"
        );
    }

    #[test]
    fn fillet_var_sel_vertical_only() {
        // Variable-radius fillet on just the 4 vertical edges.
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let original_edge_count = b.edges("all").unwrap().len();
        let filleted = b
            .fillet_var_sel(0.5, 2.0, "vertical")
            .expect("fillet_var_sel vertical failed");
        let new_edge_count = filleted.edges("all").unwrap().len();
        assert!(
            new_edge_count > original_edge_count,
            "expected more edges after variable-radius selective fillet, got {new_edge_count} vs {original_edge_count}"
        );
    }

    #[test]
    fn scale_xyz_stretches_bounding_box() {
        // box(1,1,1) scaled by (2,3,4) should produce extents (2,3,4).
        let unit = Shape::make_box(1.0, 1.0, 1.0).unwrap();
        let scaled = unit.scale_xyz(2.0, 3.0, 4.0).expect("scale_xyz failed");
        let bb = scaled.bounding_box().expect("bounding_box failed");
        // bb = [xmin, ymin, zmin, xmax, ymax, zmax]
        let (dx, dy, dz) = (bb[3] - bb[0], bb[4] - bb[1], bb[5] - bb[2]);
        assert!((dx - 2.0).abs() < 1e-6, "expected dx=2, got {dx}");
        assert!((dy - 3.0).abs() < 1e-6, "expected dy=3, got {dy}");
        assert!((dz - 4.0).abs() < 1e-6, "expected dz=4, got {dz}");
    }

    #[test]
    fn linear_pattern_copies_along_axis() {
        // 3 copies of a 2×2×2 box spaced 5 units apart along X should have
        // a bounding box that spans [0, 0+5+5+2] = [0, 12] in X.
        let b = Shape::make_box(2.0, 2.0, 2.0).unwrap();
        let pattern = b
            .linear_pattern(3, 5.0, 0.0, 0.0)
            .expect("linear_pattern failed");
        let bb = pattern.bounding_box().expect("bounding_box failed");
        let dx = bb[3] - bb[0]; // xmax - xmin
        assert!(
            (dx - 12.0).abs() < 1e-4,
            "expected x-extent of 12, got {dx}"
        );
    }

    #[test]
    fn polar_pattern_fills_circle() {
        // 4 copies at 360° — each rotated 90° further — should span roughly
        // the same extents in X and Y.
        let b = Shape::make_box(1.0, 1.0, 5.0)
            .unwrap()
            .translate(3.0, 0.0, 0.0)
            .unwrap();
        let pattern = b.polar_pattern(4, 360.0).expect("polar_pattern failed");
        let bb = pattern.bounding_box().expect("bounding_box failed");
        let dx = bb[3] - bb[0];
        let dy = bb[4] - bb[1];
        // With 4 copies at 90° intervals, the compound should be roughly symmetric.
        assert!(
            (dx - dy).abs() < 0.5,
            "expected symmetric extents, got dx={dx}, dy={dy}"
        );
    }

    #[test]
    fn linear_pattern_n1_returns_original_shape() {
        // n=1 should produce a single-copy compound with the same bounding box.
        let b = Shape::make_box(3.0, 4.0, 5.0).unwrap();
        let bb_orig = b.bounding_box().unwrap();
        let pattern = b
            .linear_pattern(1, 10.0, 0.0, 0.0)
            .expect("linear_pattern n=1 failed");
        let bb_pat = pattern.bounding_box().unwrap();
        let orig_dx = bb_orig[3] - bb_orig[0];
        let pat_dx = bb_pat[3] - bb_pat[0];
        assert!(
            (orig_dx - pat_dx).abs() < 1e-4,
            "n=1 pattern should match original x-extent"
        );
    }

    #[test]
    fn vertices_count_box() {
        // A box has exactly 8 corners.
        let b = Shape::make_box(5.0, 5.0, 5.0).unwrap();
        let verts = b.vertices("all").expect("vertices failed");
        assert_eq!(
            verts.len(),
            8,
            "expected 8 vertices on a box, got {}",
            verts.len()
        );
    }

    #[test]
    fn vertices_bad_selector_returns_error() {
        let b = Shape::make_box(5.0, 5.0, 5.0).unwrap();
        let result = b.vertices("top");
        assert!(
            result.is_err(),
            "expected error for unsupported selector 'top'"
        );
    }

    #[test]
    fn faces_direction_selector_top() {
        // A box has exactly 1 top face (normal pointing in +Z).
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let top_faces = b.faces(">Z").expect("faces(>Z) failed");
        assert_eq!(
            top_faces.len(),
            1,
            "expected 1 top face, got {}",
            top_faces.len()
        );
    }

    #[test]
    fn faces_direction_selector_bottom() {
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let bottom_faces = b.faces("<Z").expect("faces(<Z) failed");
        assert_eq!(
            bottom_faces.len(),
            1,
            "expected 1 bottom face, got {}",
            bottom_faces.len()
        );
    }

    #[test]
    fn faces_direction_selector_x_sides() {
        // A box has 2 faces with normals along the X axis.
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let pos_x = b.faces(">X").expect("faces(>X) failed");
        let neg_x = b.faces("<X").expect("faces(<X) failed");
        assert_eq!(pos_x.len(), 1, "expected 1 +X face");
        assert_eq!(neg_x.len(), 1, "expected 1 -X face");
    }

    #[test]
    fn export_obj_creates_file() {
        let b = Shape::make_box(5.0, 5.0, 5.0).unwrap();
        let out = std::env::temp_dir().join("rrcad_test_export.obj");
        b.export_obj(out.to_str().unwrap(), 0.1)
            .expect("export_obj failed");
        assert!(out.exists(), "OBJ file was not created");
        assert!(
            std::fs::metadata(&out).unwrap().len() > 0,
            "OBJ file is empty"
        );
    }

    // --- Color ---

    #[test]
    fn set_color_returns_new_shape() {
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        // set_color must succeed and produce a usable shape.
        let colored = b.set_color(1.0, 0.0, 0.0).expect("set_color failed");
        // The colored shape should export cleanly to GLB.
        let out = std::env::temp_dir().join("rrcad_test_colored.glb");
        colored
            .export_glb(out.to_str().unwrap(), 0.1)
            .expect("export_glb on colored shape failed");
        assert!(out.exists(), "GLB file was not created");
        assert!(
            std::fs::metadata(&out).unwrap().len() > 0,
            "GLB file is empty"
        );
    }

    // --- Assembly mating ---

    #[test]
    fn mate_stacks_box_on_box_z() {
        // A 5×5×5 post mated (bottom → top) onto a 10×10×10 base.
        // The base occupies Z = 0..10; the post should end up at Z = 10..15.
        let base = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let post = Shape::make_box(5.0, 5.0, 5.0).unwrap();

        let from_faces = post.faces("bottom").unwrap();
        let to_faces = base.faces("top").unwrap();

        let mated = post.mate(&from_faces[0], &to_faces[0], 0.0).unwrap();
        let bb = mated.bounding_box().unwrap();
        // bb = [xmin, ymin, zmin, xmax, ymax, zmax]
        assert!(
            (bb[2] - 10.0).abs() < 0.01,
            "Zmin should be ≈10, got {}",
            bb[2]
        );
        assert!(
            (bb[5] - 15.0).abs() < 0.01,
            "Zmax should be ≈15, got {}",
            bb[5]
        );
    }

    #[test]
    fn mate_with_offset_leaves_gap() {
        let base = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let post = Shape::make_box(5.0, 5.0, 5.0).unwrap();
        let from_faces = post.faces("bottom").unwrap();
        let to_faces = base.faces("top").unwrap();
        let mated = post.mate(&from_faces[0], &to_faces[0], 3.0).unwrap();
        let bb = mated.bounding_box().unwrap();
        // With offset=3, post bottom should be at Z=13 (10 + 3 gap).
        assert!(
            (bb[2] - 13.0).abs() < 0.01,
            "Zmin should be ≈13, got {}",
            bb[2]
        );
    }

    #[test]
    fn mate_non_planar_face_returns_error() {
        let cyl = Shape::make_cylinder(5.0, 10.0).unwrap();
        let base = Shape::make_box(20.0, 20.0, 5.0).unwrap();
        // Side face of a cylinder is non-planar — mate should error.
        let side_faces = cyl.faces("side").unwrap();
        let to_faces = base.faces("top").unwrap();
        let result = cyl.mate(&side_faces[0], &to_faces[0], 0.0);
        match result {
            Ok(_) => panic!("expected error for non-planar from-face"),
            Err(err) => assert!(err.contains("planar"), "unexpected error: {err}"),
        }
    }

    #[test]
    fn set_color_does_not_modify_original() {
        let b = Shape::make_box(10.0, 10.0, 10.0).unwrap();
        let _colored = b.set_color(0.0, 1.0, 0.0).expect("set_color failed");
        // Original shape must still export without error.
        let out = std::env::temp_dir().join("rrcad_test_uncolored.glb");
        b.export_glb(out.to_str().unwrap(), 0.1)
            .expect("original shape export failed after set_color");
    }
}
