use super::{NamedRef, Shape, ffi};

impl Shape {
    // --- Phase 3: sub-shape selectors ---

    pub fn faces(&self, selector: &str) -> Result<Vec<Shape>, String> {
        if let Some(named) = self.resolve_named_selector(selector) {
            match named {
                NamedRef::FaceSelector(alias) => return self.faces(&alias),
                NamedRef::EdgeSelector(_) => {
                    return Err(format!("faces: named reference ':{selector}' is an edge"));
                }
                NamedRef::Datum(shape) => {
                    if shape.shape_type_name()? == "face" {
                        return Ok(vec![shape.as_ref().clone()]);
                    }
                    return Err(format!(
                        "faces: named reference ':{selector}' is not a face"
                    ));
                }
            }
        }
        let n = ffi::shape_faces_count(&self.inner, selector).map_err(|e| e.to_string())?;
        (0..n)
            .map(|i| {
                ffi::shape_faces_get(&self.inner, selector, i)
                    .map(|p| {
                        self.with_inner_and_history(
                            p,
                            format!("faces(selector={selector}, idx={i})"),
                        )
                    })
                    .map_err(|e| e.to_string())
            })
            .collect()
    }

    pub fn edges(&self, selector: &str) -> Result<Vec<Shape>, String> {
        if let Some(named) = self.resolve_named_selector(selector) {
            match named {
                NamedRef::EdgeSelector(alias) => return self.edges(&alias),
                NamedRef::FaceSelector(_) => {
                    return Err(format!("edges: named reference ':{selector}' is a face"));
                }
                NamedRef::Datum(shape) => {
                    if shape.shape_type_name()? == "edge" {
                        return Ok(vec![shape.as_ref().clone()]);
                    }
                    return Err(format!(
                        "edges: named reference ':{selector}' is not an edge"
                    ));
                }
            }
        }
        let n = ffi::shape_edges_count(&self.inner, selector).map_err(|e| e.to_string())?;
        (0..n)
            .map(|i| {
                ffi::shape_edges_get(&self.inner, selector, i)
                    .map(|p| {
                        self.with_inner_and_history(
                            p,
                            format!("edges(selector={selector}, idx={i})"),
                        )
                    })
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
                    .map(|p| {
                        self.with_inner_and_history(
                            p,
                            format!("vertices(selector={selector}, idx={i})"),
                        )
                    })
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

    /// Shape type as a lowercase string: `"solid"`, `"shell"`, `"face"`,
    /// `"wire"`, `"edge"`, `"vertex"`, `"compound"`, or `"other"`.
    pub fn shape_type_name(&self) -> Result<String, String> {
        ffi::shape_type_str(&self.inner).map_err(|e| e.to_string())
    }

    /// Centroid of the shape as `[x, y, z]`.
    pub fn centroid(&self) -> Result<[f64; 3], String> {
        let mut out = [0f64; 3];
        ffi::shape_centroid(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Outward unit normal of a face as `[nx, ny, nz]`.  Sampled at the
    /// middle of the face's parameter space; flipped when the face's
    /// orientation is REVERSED so it points out of the parent solid.
    pub fn face_normal(&self) -> Result<[f64; 3], String> {
        let mut out = [0f64; 3];
        ffi::shape_face_normal(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Cylindrical face axis as `[ox, oy, oz, ax, ay, az, radius]`.
    /// Errors if the shape is not a face or its surface is not a cylinder.
    pub fn cylinder_axis(&self) -> Result<[f64; 7], String> {
        let mut out = [0f64; 7];
        ffi::shape_cylinder_axis(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// `true` if every edge is shared by at least two faces (no free/boundary edges).
    pub fn is_closed(&self) -> Result<bool, String> {
        ffi::shape_is_closed(&self.inner).map_err(|e| e.to_string())
    }

    /// `true` if every edge is shared by exactly two faces (no T-junctions).
    pub fn is_manifold(&self) -> Result<bool, String> {
        ffi::shape_is_manifold(&self.inner).map_err(|e| e.to_string())
    }

    /// Run `BRepCheck_Analyzer`.  Returns `"ok"` if valid, or a
    /// newline-separated list of error names if not.
    pub fn validate(&self) -> Result<String, String> {
        ffi::shape_validate_str(&self.inner).map_err(|e| e.to_string())
    }

    /// Minimum distance between two shapes. Returns 0.0 when they intersect.
    pub fn distance_to(&self, other: &Shape) -> Result<f64, String> {
        ffi::shape_distance_to(&self.inner, &other.inner).map_err(|e| e.to_string())
    }

    /// Inertia tensor [Ixx, Iyy, Izz, Ixy, Ixz, Iyz] about the centre of mass.
    pub fn inertia(&self) -> Result<[f64; 6], String> {
        let mut out = [0f64; 6];
        ffi::shape_inertia(&self.inner, &mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Minimum wall thickness of a solid/shell.
    pub fn min_thickness(&self) -> Result<f64, String> {
        ffi::shape_min_thickness(&self.inner).map_err(|e| e.to_string())
    }
}
