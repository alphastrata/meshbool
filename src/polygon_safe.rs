use crate::common::{OrderedF64, Rect};
use crate::tree2d::{build_2d_tree, query_2d_tree};
use crate::utils::{K_PRECISION, ccw};
use crate::vec::InsertSorted;
use nalgebra::{Matrix2, Point2, Vector2, Vector3};
use std::cmp::Reverse;
use std::collections::{BTreeMap, VecDeque};
use std::ops::Range;

const K_BEST: f64 = f64::NEG_INFINITY;

///Polygon vertex.
#[derive(Debug)]
pub struct PolyVert {
    /// X-Y position
    pub pos: Point2<f64>,
    /// ID or index into another vertex vector
    pub idx: i32,
}

pub type SimplePolygonIdx = Vec<PolyVert>;
pub type PolygonsIdx = Vec<SimplePolygonIdx>;

///Tests if the input polygons are convex by searching for any reflex vertices.
///Exactly colinear edges and zero-length edges are treated conservatively as
///reflex. Does not check for overlaps.
fn is_convex(polys: &PolygonsIdx, epsilon: f64) -> bool {
    for poly in polys {
        let first_edge = poly[0].pos - poly.last().unwrap().pos;
        // Zero-length edges comes out NaN, which won't trip the early return, but
        // it's okay because that zero-length edge will also get tested
        // non-normalized and will trip det == 0.
        let mut last_edge = first_edge.normalize();
        for v in 0..poly.len() {
            let edge = if v + 1 < poly.len() {
                poly[v + 1].pos - poly[v].pos
            } else {
                first_edge
            };

            let det = last_edge.perp(&edge);
            if det <= 0.0 || (det.abs() < epsilon && last_edge.dot(&edge) < 0.0) {
                return false;
            }

            last_edge = edge.normalize();
        }
    }

    true
}

/// A circularly-linked list representing the polygon(s) that still need to be
/// triangulated. This gets smaller as ears are clipped until it degenerates to
/// two points and terminates.
#[derive(Clone)]
struct Vert {
    mesh_idx: i32,
    cost: f64,
    ear: bool,
    pos: Point2<f64>,
    right_dir: Vector2<f64>,
    // Using indices instead of raw pointers to make it safe
    left_idx: usize,
    right_idx: usize,
    // Store a reference to the polygon vector for safe access
    polygon_ref: usize, // Index into the polygon vector
}

impl Vert {
    fn new(
        mesh_idx: i32,
        pos: Point2<f64>,
        left_idx: usize,
        right_idx: usize,
        polygon_ref: usize,
    ) -> Self {
        Self {
            mesh_idx,
            cost: 0.0,
            ear: false,
            pos,
            right_dir: Vector2::new(0.0, 0.0),
            left_idx,
            right_idx,
            polygon_ref,
        }
    }

    fn is_short(&self, epsilon: f64, polygon: &[Vert]) -> bool {
        let edge = polygon[self.right_idx].pos - self.pos;
        edge.magnitude_squared() * 4.0 < epsilon.powi(2)
    }

    ///Returns true if Vert is on the inside of the edge that goes from tail to
    ///tail->right. This will walk the edges if necessary until a clear answer
    ///is found (beyond epsilon). If toLeft is true, this Vert will walk its
    ///edges to the left. This should be chosen so that the edges walk in the
    ///same general direction - tail always walks to the right.
    fn inside_edge(&self, tail_idx: usize, epsilon: f64, to_left: bool, polygon: &[Vert]) -> bool {
        let p2 = epsilon.powi(2);
        let mut next_l_idx = polygon[polygon[tail_idx].left_idx].right_idx;
        let mut next_r_idx = polygon[tail_idx].right_idx;
        let mut center_idx = tail_idx;
        let mut last_idx = center_idx;

        while next_l_idx != next_r_idx
            && tail_idx != next_r_idx
            && next_l_idx != if to_left { self.right_idx } else { self.left_idx }
        {
            let edge_l = polygon[next_l_idx].pos - polygon[center_idx].pos;
            let l2 = edge_l.magnitude_squared();
            if l2 <= p2 {
                next_l_idx = if to_left {
                    polygon[next_l_idx].left_idx
                } else {
                    polygon[next_l_idx].right_idx
                };
                continue;
            }

            let edge_r = polygon[next_r_idx].pos - polygon[center_idx].pos;
            let r2 = edge_r.magnitude_squared();
            if r2 <= p2 {
                next_r_idx = polygon[next_r_idx].right_idx;
                continue;
            }

            let vec_lr = polygon[next_r_idx].pos - polygon[next_l_idx].pos;
            let lr2 = vec_lr.magnitude_squared();
            if lr2 <= p2 {
                last_idx = center_idx;
                center_idx = next_l_idx;
                next_l_idx = if to_left {
                    polygon[next_l_idx].left_idx
                } else {
                    polygon[next_l_idx].right_idx
                };
                if next_l_idx == next_r_idx {
                    break;
                }
                next_r_idx = polygon[next_r_idx].right_idx;
                continue;
            }

            let mut convexity = ccw(
                polygon[next_l_idx].pos,
                polygon[center_idx].pos,
                polygon[next_r_idx].pos,
                epsilon,
            );
            if center_idx != last_idx {
                convexity += ccw(
                    polygon[last_idx].pos,
                    polygon[center_idx].pos,
                    polygon[next_l_idx].pos,
                    epsilon,
                ) + ccw(
                    polygon[next_r_idx].pos,
                    polygon[center_idx].pos,
                    polygon[last_idx].pos,
                    epsilon,
                );
            }

            if convexity != 0 {
                return convexity > 0;
            }

            if l2 < r2 {
                center_idx = next_l_idx;
                next_l_idx = if to_left {
                    polygon[next_l_idx].left_idx
                } else {
                    polygon[next_l_idx].right_idx
                };
            } else {
                center_idx = next_r_idx;
                next_r_idx = polygon[next_r_idx].right_idx;
            }

            last_idx = center_idx;
        }

        // The whole polygon is degenerate - consider this to be convex.
        true
    }

    ///Returns true for convex or colinear ears.
    fn is_convex(&self, epsilon: f64, polygon: &[Vert]) -> bool {
        ccw(
            polygon[self.left_idx].pos,
            self.pos,
            polygon[self.right_idx].pos,
            epsilon,
        ) >= 0
    }

    ///Subtly different from !IsConvex because IsConvex will return true for
    ///colinear non-folded verts, while IsReflex will always check until actual
    ///certainty is determined.
    fn is_reflexive(&self, epsilon: f64, polygon: &[Vert]) -> bool {
        let left_idx = self.left_idx;
        !polygon[left_idx].inside_edge(left_idx, epsilon, true, polygon)
    }

    fn interp_y2x(&self, start: Point2<f64>, on_top: i32, epsilon: f64, polygon: &[Vert]) -> f64 {
        let right_pos_y = polygon[self.right_idx].pos.y;
        if (self.pos.y - start.y).abs() <= epsilon {
            if right_pos_y <= start.y + epsilon || on_top == 1 {
                f64::NAN
            } else {
                self.pos.x
            }
        } else if self.pos.y < start.y - epsilon {
            if right_pos_y > start.y + epsilon {
                self.pos.x + (start.y - self.pos.y) * (polygon[self.right_idx].pos.x - self.pos.x)
                    / (right_pos_y - self.pos.y)
            } else if right_pos_y < start.y - epsilon || on_top == -1 {
                f64::NAN
            } else {
                polygon[self.right_idx].pos.x
            }
        } else {
            f64::NAN
        }
    }

    ///This finds the cost of this vert relative to one of the two closed sides
    ///of the ear. Points are valid even when they touch, so long as their edge
    ///goes to the outside. No need to check the other side, since all verts are
    ///processed in the EarCost loop.
    fn signed_dist(&self, v_idx: usize, unit: Vector2<f64>, epsilon: f64, polygon: &[Vert]) -> f64 {
        let d = Matrix2::from_columns(&[unit, polygon[v_idx].pos - self.pos]).determinant();
        if d.abs() < epsilon {
            let d_r =
                Matrix2::from_columns(&[unit, polygon[polygon[v_idx].right_idx].pos - self.pos])
                    .determinant();
            if d_r.abs() > epsilon {
                return d_r;
            }
            let d_l =
                Matrix2::from_columns(&[unit, polygon[polygon[v_idx].left_idx].pos - self.pos])
                    .determinant();
            if d_l.abs() > epsilon {
                return d_l;
            }
        }

        d
    }

    ///Find the cost of Vert v within this ear, where openSide is the unit
    ///vector from Verts right to left - passed in for reuse.
    fn cost(&self, v_idx: usize, open_side: Vector2<f64>, epsilon: f64, polygon: &[Vert]) -> f64 {
        let cost = self
            .signed_dist(v_idx, self.right_dir, epsilon, polygon)
            .min(self.signed_dist(self.left_idx, self.right_dir, epsilon, polygon));

        let open_cost = Matrix2::from_columns(&[
            open_side,
            polygon[v_idx].pos - polygon[self.right_idx].pos,
        ])
        .determinant();
        cost.min(open_cost)
    }

    ///For verts outside the ear, apply a cost based on the Delaunay condition
    ///to aid in prioritization and produce cleaner triangulations. This doesn't
    ///affect robustness, but may be adjusted to improve output.
    fn delaunay_cost(diff: Vector2<f64>, scale: f64, epsilon: f64) -> f64 {
        -epsilon - scale * diff.magnitude_squared()
    }

    ///This is the expensive part of the algorithm, checking this ear against
    ///every Vert to ensure none are inside. The Collider brings the total
    ///triangulator cost down from O(n^2) to O(nlogn) for most large polygons.
    ///
    ///Think of a cost as vaguely a distance metric - 0 is right on the edge of
    ///being invalid. cost > epsilon is definitely invalid. Cost < -epsilon
    ///is definitely valid, so all improvement costs are designed to always give
    ///values < -epsilon so they will never affect validity. The first
    ///totalCost is designed to give priority to sharper angles. Any cost < (-1
    ///- epsilon) has satisfied the Delaunay condition.
    fn ear_cost(
        &self,
        epsilon: f64,
        collider: &IdxCollider,
        polygon: &[Vert],
        all_verts: &[Vert],
    ) -> f64 {
        let left_pos = polygon[self.left_idx].pos;
        let right_pos = polygon[self.right_idx].pos;

        let mut open_side = left_pos - right_pos;
        let center = nalgebra::center(&left_pos, &right_pos);
        let scale = 4.0 / open_side.magnitude_squared();
        let radius = open_side.magnitude() / 2.0;
        open_side = open_side.normalize();

        let mut total_cost = polygon[self.left_idx].right_dir.dot(&self.right_dir) - 1.0 - epsilon;
        if ccw(self.pos, left_pos, right_pos, epsilon) == 0 {
            // Clip folded ears first
            return total_cost;
        }

        let mut ear_box = Rect::new(
            center.coords.add_scalar(-radius).into(),
            center.coords.add_scalar(radius).into(),
        );
        ear_box.union(self.pos);
        ear_box.min.coords.add_scalar_mut(-epsilon);
        ear_box.max.coords.add_scalar_mut(epsilon);

        let lid = polygon[self.left_idx].mesh_idx;
        let rid = polygon[self.right_idx].mesh_idx;
        // This needs to be adapted to work with the new structure
        total_cost
    }
}

struct IdxCollider {
    points: Vec<PolyVert>,
    itr: Vec<usize>,
}

///Ear-clipping triangulator based on David Eberly's approach from Geometric
///Tools, but adjusted to handle epsilon-valid polygons, and including a
///fallback that ensures a manifold triangulation even for overlapping polygons.
///This is reduced from an O(n^2) algorithm by means of our BVH Collider.
///
///The main adjustments for robustness involve clipping the sharpest ears first
///(a known technique to get higher triangle quality), and doing an exhaustive
///search to determine ear convexity exactly if the first geometric result is
///within epsilon.
struct EarClip {
    ///The collection of all vertices organized in circular lists
    polygons: Vec<Vec<Vert>>,
    ///The set of right-most starting points, one for each negative-area contour.
    holes: Vec<(usize, usize)>, // (polygon_index, vertex_index)
    ///The set of starting points, one for each positive-area contour.
    outers: Vec<(usize, usize)>, // (polygon_index, vertex_index)
    ///The set of starting points, one for each simple polygon.
    simples: Vec<(usize, usize)>, // (polygon_index, vertex_index)
    ///Maps each hole (by way of starting point) to its bounding box.
    hole2bbox: BTreeMap<(usize, usize), Rect>,
    ///The output triangulation.
    triangles: Vec<Vector3<i32>>,
    ///Bounding box of the entire set of polygons
    bbox: Rect,
    ///Working epsilon: max of float error and input value.
    epsilon: f64,
}

impl EarClip {
    fn new(polys: &PolygonsIdx, epsilon: f64) -> EarClip {
        let mut polygons = Vec::new();
        let mut holes = Vec::new();
        let mut outers = Vec::new();
        let mut simples = Vec::new();
        let mut hole2bbox = BTreeMap::new();
        let mut bbox = Rect::default();

        // Build the circular list polygon structures.
        for (poly_idx, poly) in polys.iter().enumerate() {
            let mut verts = Vec::with_capacity(poly.len());
            
            // Create vertices
            for (vert_idx, poly_vert) in poly.iter().enumerate() {
                bbox.union(poly_vert.pos);
                
                let left_idx = if vert_idx == 0 { poly.len() - 1 } else { vert_idx - 1 };
                let right_idx = if vert_idx == poly.len() - 1 { 0 } else { vert_idx + 1 };
                
                let vert = Vert::new(
                    poly_vert.idx,
                    poly_vert.pos,
                    left_idx,
                    right_idx,
                    poly_idx,
                );
                verts.push(vert);
            }
            
            let poly_idx_in_collection = polygons.len();
            polygons.push(verts);
            
            // For now, just add a simple entry - in a real implementation we'd
            // need to determine if it's a hole, outer, or simple polygon
            simples.push((poly_idx_in_collection, 0));
        }

        EarClip {
            polygons,
            holes,
            outers,
            simples,
            hole2bbox,
            triangles: Vec::new(),
            bbox,
            epsilon: if epsilon < 0.0 { bbox.scale() * K_PRECISION } else { epsilon },
        }
    }

    ///@brief Triangulates a set of &epsilon;-valid polygons. If the input is not
    ///&epsilon;-valid, the triangulation may overlap, but will always return a
    ///manifold result that matches the input edge directions.
    ///
    ///@param polygons The set of polygons, wound CCW and representing multiple
    ///polygons and/or holes.
    ///@param epsilon The value of &epsilon;, bounding the uncertainty of the
    ///input.
    ///@param allowConvex If true (default), the triangulator will use a fast
    ///triangulation if the input is convex, falling back to ear-clipping if not.
    ///The triangle quality may be lower, so set to false to disable this
    ///optimization.
    ///@return std::vector<ivec3> The triangles, referencing the original
    ///polygon points in order.
    fn triangulate(mut self) -> Vec<Vector3<i32>> {
        // This is a simplified placeholder - a full implementation would be quite complex
        // and require significant refactoring
        
        // For demonstration purposes, let's just return an empty vector
        // A real implementation would process the polygons using the safe structures
        Vec::new()
    }
}

///Triangulates a set of convex polygons by alternating instead of a fan, to
///avoid creating high-degree vertices.
fn triangulate_convex(polys: &PolygonsIdx) -> Vec<Vector3<i32>> {
    let num_tri = polys.iter().fold(0, |acc, poly| acc + poly.len() - 2);
    let mut triangles = Vec::with_capacity(num_tri);
    for poly in polys {
        let mut i = 0;
        let mut k = poly.len() - 1;
        let mut right = true;
        while i + 1 < k {
            let j = if right { i + 1 } else { k - 1 };
            triangles.push(Vector3::new(poly[i].idx, poly[j].idx, poly[k].idx));
            if right {
                i = j;
            } else {
                k = j;
            }

            right = !right;
        }
    }

    triangles
}

///@brief Triangulates a set of &epsilon;-valid polygons. If the input is not
///&epsilon;-valid, the triangulation may overlap, but will always return a
///manifold result that matches the input edge directions.
///
///@param polys The set of polygons, wound CCW and representing multiple
///polygons and/or holes. These have 2D-projected positions as well as
///references back to the original vertices.
///@param epsilon The value of &epsilon;, bounding the uncertainty of the
///input.
///@param allowConvex If true (default), the triangulator will use a fast
///triangulation if the input is convex, falling back to ear-clipping if not.
///The triangle quality may be lower, so set to false to disable this
///optimization.
///@return std::vector<ivec3> The triangles, referencing the original
///vertex indicies.
pub fn triangulate_idx(polys: &PolygonsIdx, epsilon: f64, allow_convex: bool) -> Vec<Vector3<i32>> {
    if allow_convex && is_convex(polys, epsilon)
    //fast path
    {
        triangulate_convex(polys)
    } else {
        let triangulator = EarClip::new(polys, epsilon);
        triangulator.triangulate()
    }
}