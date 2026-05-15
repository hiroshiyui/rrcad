use super::{FeatureOp, Shape, ffi};
use crate::occt::shape_core::summarize;

impl Shape {
    pub fn fuse_all(shapes: &[&Shape]) -> Result<Shape, String> {
        if shapes.len() < 2 {
            return Err("fuse_all: requires at least 2 shapes".to_string());
        }
        let mut iter = shapes.iter();
        let first = *iter
            .next()
            .expect("fuse_all: invariant: at least 2 shapes after len check");
        let second = *iter
            .next()
            .expect("fuse_all: invariant: at least 2 shapes after len check");
        let mut acc = first.fuse(second)?;
        for s in iter {
            acc = acc.fuse(s)?;
        }
        Ok(acc)
    }

    pub fn cut_all(&self, tools: &[&Shape]) -> Result<Shape, String> {
        if tools.is_empty() {
            return Err("cut_all: requires at least 1 tool".to_string());
        }
        let mut acc = self.translate(0.0, 0.0, 0.0)?;
        for tool in tools {
            acc = acc.cut(tool)?;
        }
        Ok(acc)
    }

    pub fn fragment_all(shapes: &[&Shape]) -> Result<Shape, String> {
        if shapes.is_empty() {
            return Err("fragment: requires at least 1 shape".to_string());
        }
        let mut builder = ffi::fragment_new().map_err(|e| e.to_string())?;
        for s in shapes {
            ffi::fragment_add(builder.pin_mut(), &s.inner).map_err(|e| e.to_string())?;
        }
        ffi::fragment_build(builder.pin_mut())
            .map(|p| {
                let summary = shapes
                    .iter()
                    .map(|s| summarize(s))
                    .collect::<Vec<_>>()
                    .join(", ");
                Shape::fresh_with_feature_parents(
                    p,
                    FeatureOp::FragmentAll {
                        count: shapes.len(),
                    },
                    format!("fragment_all(shapes=[{summary}])"),
                    shapes.iter().map(|s| s.feature.clone()).collect(),
                )
            })
            .map_err(|e| e.to_string())
    }

    pub fn convex_hull(&self) -> Result<Shape, String> {
        ffi::shape_convex_hull(&self.inner)
            .map(|p| {
                self.with_feature(
                    p,
                    FeatureOp::ConvexHull,
                    "convex_hull()".to_string(),
                    vec![self.feature.clone()],
                )
            })
            .map_err(|e| e.to_string())
    }
}
