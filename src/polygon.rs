use crate::common::{OrderedF64, Rect};
use crate::tree2d::{build_2d_tree, query_2d_tree};
use crate::utils::{K_PRECISION, ccw};
use crate::vec::InsertSorted;
use nalgebra::{Matrix2, Point2, Vector2, Vector3};
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::ops::Range;
use std::{collections::VecDeque, mem, ptr};

struct CutKeyholeParams<'a> {
    simples: &'a mut Vec<usize>,
    triangles: &'a mut Vec<Vector3<i32>>,
    polygon: &'a mut Vec<Vert>,
    polygon_range: &'a Range<usize>,
    outers: &'a Vec<usize>,
    hole2bbox: &'a BTreeMap<usize, Rect>,
    epsilon: f64,
}


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

/// Ear-clipping triangulator based on David Eberly's approach from Geometric
/// Tools, but adjusted to handle epsilon-valid polygons, and including a
/// fallback that ensures a manifold triangulation even for overlapping polygons.
/// This is reduced from an O(n^2) algorithm by means of our BVH Collider.
///
/// The main adjustments for robustness involve clipping the sharpest ears first
/// (a known technique to get higher triangle quality), and doing an exhaustive
/// search to determine ear convexity exactly if the first geometric result is
/// within epsilon.
/// 
/// # Safety
/// This struct maintains the safety invariant for the `Vert` instances it manages:
/// - All `Vert` instances are stored in the `polygon` vector which acts as an arena
/// - The vector is allocated with enough capacity upfront to avoid reallocation
/// - All vertices are dropped together when the `EarClip` instance is dropped
/// - The raw pointers in `Vert` instances only point to other vertices in the same vector
/// 
/// This ensures that the unsafe pointer dereferencing in the `Vert` accessor methods
/// is safe because the memory addresses remain valid throughout the lifetime of the
/// `EarClip` instance.
struct EarClip {
    /// The flat list where all the Verts are stored. Not used much for traversal.
    /// This vector acts as an arena, allocated with enough capacity to avoid reallocation.
    polygon: Vec<Vert>,
    //Pointers to first and last verts within self.polygon
    polygon_range: Range<usize>,
    /// The set of right-most starting points, one for each negative-area contour.
    //originally a c++ multiset
    holes: Vec<usize>,
    /// The set of starting points, one for each positive-area contour.
    outers: Vec<usize>,
    /// The set of starting points, one for each simple polygon.
    simples: Vec<usize>,
    /// Maps each hole (by way of starting point) to its bounding box.
    hole2bbox: BTreeMap<usize, Rect>,
    /// The output triangulation.
    triangles: Vec<Vector3<i32>>,
    /// Bounding box of the entire set of polygons
    bbox: Rect,
    /// Working epsilon: max of float error and input value.
    epsilon: f64,
}

impl EarClip {
    fn new(polys: &PolygonsIdx, epsilon: f64) -> EarClip {
        let mut num_vert = 0;
        for poly in polys {
            num_vert += poly.len();
        }

        let polygon: Vec<Vert> = Vec::with_capacity(num_vert + 2 * polys.len()); //must never reallocate or else all vert.left and vert.right break
        let polygon_first = polygon.as_ptr() as usize;
        let polygon_end = unsafe { polygon.as_ptr().add(polygon.capacity()) } as usize;
        let polygon_range = polygon_first..polygon_end;

        let mut ret = EarClip {
            polygon,
            polygon_range,
            holes: Vec::new(),
            outers: Vec::new(),
            simples: Vec::new(),
            hole2bbox: BTreeMap::new(),
            triangles: Vec::default(),
            bbox: Rect::default(),
            epsilon,
        };

        let starts = ret.initialize(polys);

        for v in 0..ret.polygon.len() {
            Self::clip_if_degenerate(
                v,
                &mut ret.polygon,
                &ret.polygon_range,
                &mut ret.triangles,
                ret.epsilon,
            );
        }

        for first in starts {
            ret.find_start(first);
        }

        ret
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
        for start in self.holes {
            let params = CutKeyholeParams {
                simples: &mut self.simples,
                triangles: &mut self.triangles,
                polygon: &mut self.polygon,
                polygon_range: &self.polygon_range,
                outers: &self.outers,
                hole2bbox: &self.hole2bbox,
                epsilon: self.epsilon,
            };
            Self::cut_keyhole(
                start,
                params,
            );
        }

        drop(self.outers);
        drop(self.hole2bbox);

        for start in self.simples {
            Self::triangulate_poly(
                start,
                &mut self.polygon,
                &self.polygon_range,
                &mut self.triangles,
                self.epsilon,
            );
        }

        self.triangles
    }

    fn safe_normalize(v: Vector2<f64>) -> Vector2<f64> {
        let n = v.normalize();
        if n.x.is_finite() {
            n
        } else {
            Vector2::new(0.0, 0.0)
        }
    }

    ///This function and JoinPolygons are the only functions that affect the
    ///circular list data structure. This helps ensure it remains circular.
    fn link(left: usize, right: usize, polygon: &mut Vec<Vert>) {
        polygon[left].right_idx = right;
        polygon[right].left_idx = left;
        polygon[left].right_dir = Self::safe_normalize(polygon[right].pos - polygon[left].pos);
    }

    ///When an ear vert is clipped, its neighbors get linked, so they get unlinked
    ///from it, but it is still linked to them.
    fn clipped(v: &Vert, polygon: &[Vert]) -> bool {
        !ptr::eq(v.right(polygon).left(polygon), v)
    }

    fn loop_verts(
        mut first: usize,
        polygon: &mut Vec<Vert>,
        polygon_range: &Range<usize>,
        mut func: impl FnMut(usize, &mut Vec<Vert>),
    ) -> Option<usize> {
        let mut v = first;
        loop {
            let mut ref_v = &polygon[v];
            if Self::clipped(ref_v, polygon) {
                // Update first to an un-clipped vert so we will return to it instead
                // of infinite-looping.
                let new_first = ref_v.right(polygon).left(polygon);
                first = new_first.ptr2index(polygon_range);
                if !Self::clipped(new_first, polygon) {
                    v = first;
                    ref_v = &polygon[v];
                    if ref_v.right_idx == ref_v.left_idx {
                        return None;
                    }

                    func(v, polygon);
                }
            } else {
                if ref_v.right_idx == ref_v.left_idx {
                    return None;
                }

                func(v, polygon);
            }

            v = polygon[v].right(polygon).ptr2index(polygon_range);
            if v == first {
                break;
            }
        }

        Some(v)
    }

    fn clip_ear(
        ear: usize,
        polygon: &mut Vec<Vert>,
        polygon_range: &Range<usize>,
        triangles: &mut Vec<Vector3<i32>>,
    ) {
        let ear_ref = &polygon[ear];
        let left_i = ear_ref.left(polygon).ptr2index(polygon_range);
        let right_i = ear_ref.right(polygon).ptr2index(polygon_range);
        Self::link(left_i, right_i, polygon);

        let ear_ref = &polygon[ear];
        let self_mesh = ear_ref.mesh_idx;
        let left_mesh = ear_ref.left(polygon).mesh_idx;
        let right_mesh = ear_ref.right(polygon).mesh_idx;

        if left_mesh != self_mesh && self_mesh != right_mesh && right_mesh != left_mesh {
            // Filter out topological degenerates, which can form in bad
            // triangulations of polygons with holes, due to vert duplication.
            triangles.push(Vector3::new(left_mesh, self_mesh, right_mesh));
        }
        //else Topological degenerate!
    }

    ///If an ear will make a degenerate triangle, clip it early to avoid
    ///difficulty in key-holing. This function is recursive, as the process of
    ///clipping may cause the neighbors to degenerate.
    fn clip_if_degenerate(
        ear: usize,
        polygon: &mut Vec<Vert>,
        polygon_range: &Range<usize>,
        triangles: &mut Vec<Vector3<i32>>,
        epsilon: f64,
    ) {
        let ear_ref = &polygon[ear];
        if Self::clipped(ear_ref, polygon) {
            return;
        }

        if ear_ref.left_idx == ear_ref.right_idx {
            return;
        }

        if ear_ref.is_short(epsilon, polygon)
            || (ccw(
                ear_ref.left(polygon).pos,
                ear_ref.pos,
                ear_ref.right(polygon).pos,
                epsilon,
            ) == 0
                && (ear_ref.left(polygon).pos - ear_ref.pos).dot(&(ear_ref.right(polygon).pos - ear_ref.pos))
                    > 0.0)
        {
            Self::clip_ear(ear, polygon, polygon_range, triangles);
            let ear_ref = &polygon[ear];
            let left = ear_ref.left(polygon).ptr2index(polygon_range);
            let right = ear_ref.right(polygon).ptr2index(polygon_range);
            Self::clip_if_degenerate(left, polygon, polygon_range, triangles, epsilon);
            Self::clip_if_degenerate(right, polygon, polygon_range, triangles, epsilon);
        }
    }

    ///Build the circular list polygon structures.
    fn initialize(&mut self, polys: &PolygonsIdx) -> Vec<usize> {
        let mut starts = Vec::new();
        for poly in polys {
            let vert = &poly[0];
            self.polygon.push(Vert {
                mesh_idx: vert.idx,
                cost: 0.0,
                ear: false,
                pos: vert.pos,
                right_dir: Vector2::new(0.0, 0.0),
                left_idx: 0, // Will be set properly in the link function
                right_idx: 0, // Will be set properly in the link function
                self_idx: self.polygon.len(), // Set to the current index in the vector
            });

            let first = self.polygon.last().unwrap();

            self.bbox.union(first.pos);
            let first = self.polygon.len() - 1;
            let mut last = first;
            // This is not the real rightmost start, but just an arbitrary vert for
            // now to identify each polygon.
            starts.push(first);

            for vert in &poly[1..] {
                self.bbox.union(vert.pos);

                self.polygon.push(Vert {
                    mesh_idx: vert.idx,
                    cost: 0.0,
                    ear: false,
                    pos: vert.pos,
                    right_dir: Vector2::new(0.0, 0.0),
                    left_idx: 0, // Will be set properly in the link function
                    right_idx: 0, // Will be set properly in the link function
                    self_idx: self.polygon.len(), // Set to the current index in the vector
                });

                let next = self.polygon.len() - 1;
                Self::link(last, next, &mut self.polygon);
                last = next
            }

            Self::link(last, first, &mut self.polygon);
        }

        if self.epsilon < 0.0 {
            self.epsilon = self.bbox.scale() * K_PRECISION;
        }

        // Slightly more than enough, since each hole can cause two extra triangles.
        self.triangles = Vec::with_capacity(self.polygon.len() + 2 * starts.len());
        starts
    }

    ///Find the actual rightmost starts after degenerate removal. Also calculate
    ///the polygon bounding boxes.
    fn find_start(&mut self, first: usize) {
        let origin = self.polygon[first].pos;

        let mut start = first;
        let mut max_x = f64::NEG_INFINITY;
        let mut bbox = Rect::default();
        // Kahan summation
        let mut area = 0.0;
        let mut area_compensation = 0.0;

        let add_point = |v: usize, polygon: &mut Vec<Vert>| {
            let v = &polygon[v];
            bbox.union(v.pos);
            let area1 =
                Matrix2::from_columns(&[v.pos - origin, v.right(polygon).pos - origin]).determinant();
            let t1 = area + area1;
            area_compensation += (area - t1) + area1;
            area = t1;

            if v.pos.x > max_x {
                max_x = v.pos.x;
                start = v.ptr2index(&self.polygon_range);
            }
        };

        if Self::loop_verts(first, &mut self.polygon, &self.polygon_range, add_point).is_none() {
            // No polygon left if all ears were degenerate and already clipped.
            return;
        }

        area += area_compensation;
        let size = bbox.size();
        let min_area = self.epsilon * size.x.max(size.y);

        if max_x.is_finite() && area < -min_area {
            self.holes
                .insert_sorted_by_key(start, |&hole| Reverse(OrderedF64(self.polygon[hole].pos.x))); //descending pos.x
            self.hole2bbox.entry(start).or_insert(bbox);
        } else {
            self.simples.push(start);
            if area > min_area {
                self.outers.push(start);
            }
        }
    }

    ///All holes must be key-holed (attached to an outer polygon) before ear
    ///clipping can commence. Instead of relying on sorting, which may be
    ///incorrect due to epsilon, we check for polygon edges both ahead and
    ///behind to ensure all valid options are found.
    fn cut_keyhole(
        start: usize,
        params: CutKeyholeParams,
    ) {
        let bbox = params.hole2bbox.get(&start).unwrap();
        let start_ref = &params.polygon[start];
        let on_top = if start_ref.pos.y >= bbox.max.y - params.epsilon {
            1
        } else if start_ref.pos.y <= bbox.min.y + params.epsilon {
            -1
        } else {
            0
        };

        let mut connector: Option<usize> = None;

        let mut check_edge = |edge: usize, polygon: &mut Vec<Vert>| {
            let edge = &polygon[edge];
            let start_ref = &polygon[start];
            let x = edge.interp_y2x(start_ref.pos, on_top, params.epsilon, polygon);
            if x.is_finite()
                && start_ref.inside_edge(edge, params.epsilon, true, polygon)
                && (connector.as_ref().map_or(true, |&connector| {
                    ccw(
                        Point2::new(x, start_ref.pos.y),
                        polygon[connector].pos,
                        polygon[connector].right(polygon).pos,
                        params.epsilon,
                    ) == 1
                        || (if polygon[connector].pos.y < edge.pos.y {
                            edge.inside_edge(&polygon[connector], params.epsilon, false, polygon)
                        } else {
                            !polygon[connector].inside_edge(edge, params.epsilon, false, polygon)
                        })
                }))
            {
                connector = Some(edge.ptr2index(params.polygon_range));
            }
        };

        for &first in params.outers.iter() {
            Self::loop_verts(first, params.polygon, params.polygon_range, &mut check_edge);
        }

        if connector.is_none() {
            //hole did not find an outer contour!
            params.simples.push(start);
            return;
        }

        let connector = Self::find_closer_bridge(
            start,
            connector.unwrap(),
            params.polygon,
            params.polygon_range,
            params.outers,
            params.epsilon,
        );

        Self::join_polygons(start, connector, params.polygon, params.polygon_range, params.triangles, params.epsilon);
    }

    ///This converts the initial guess for the keyhole location into the final one
    ///and returns it. It does so by finding any reflex verts inside the triangle
    ///containing the best connection and the initial horizontal line.
    fn find_closer_bridge(
        start: usize,
        edge: usize,
        polygon: &mut Vec<Vert>,
        polygon_range: &Range<usize>,
        outers: &Vec<usize>,
        epsilon: f64,
    ) -> usize {
        let start_ref = &polygon[start];
        let edge_ref = &polygon[edge];
        let connector_ref = if edge_ref.pos.x < start_ref.pos.x {
            edge_ref.right(polygon)
        } else if edge_ref.right(polygon).pos.x < start_ref.pos.x {
            edge_ref
        } else if edge_ref.right(polygon).pos.y - start_ref.pos.y > start_ref.pos.y - edge_ref.pos.y {
            edge_ref
        } else {
            edge_ref.right(polygon)
        };

        let mut connector = connector_ref.ptr2index(polygon_range);
        if (connector_ref.pos.y - start_ref.pos.y).abs() <= epsilon {
            return connector;
        }

        let above_i32 = if connector_ref.pos.y > start_ref.pos.y { 1 } else { -1 };
        let above_f64 = above_i32 as f64;

        let mut check_vert = |vert: usize, polygon: &mut Vec<Vert>| {
            let vert = &polygon[vert];
            let start_ref = &polygon[start];
            let edge_ref = &polygon[edge];
            let connector_ref = &polygon[connector];

            let inside =
                above_i32 * ccw(start_ref.pos, vert.pos, connector_ref.pos, epsilon);
            if vert.pos.x > start_ref.pos.x - epsilon
                && vert.pos.y * above_f64 > start_ref.pos.y * above_f64 - epsilon
                && (inside > 0
                    || (inside == 0
                        && vert.pos.x < connector_ref.pos.x
                        && vert.pos.y * above_f64 < connector_ref.pos.y * above_f64))
                && vert.inside_edge(edge_ref, epsilon, true, polygon)
                && vert.is_reflexive(epsilon, polygon)
            {
                connector = vert.ptr2index(polygon_range);
            }
        };

        for &first in outers {
            Self::loop_verts(first, polygon, polygon_range, &mut check_vert);
        }

        connector
    }

    ///Creates a keyhole between the start vert of a hole and the connector vert
    ///of an outer polygon. To do this, both verts are duplicated and reattached.
    ///This process may create degenerate ears, so these are clipped if necessary
    ///to keep from confusing subsequent key-holing operations.
    fn join_polygons(
        start: usize,
        connector: usize,
        polygon: &mut Vec<Vert>,
        polygon_range: &Range<usize>,
        triangles: &mut Vec<Vector3<i32>>,
        epsilon: f64,
    ) {
        let new_start = polygon.len();
        polygon.push(polygon[start].clone());
        let new_connector = polygon.len();
        polygon.push(polygon[connector].clone());

        // Update the links between vertices using indices
        polygon[start].right_idx = new_start;
        polygon[connector].left_idx = new_connector;
        Self::link(start, connector, polygon);
        Self::link(new_connector, new_start, polygon);

        Self::clip_if_degenerate(start, polygon, polygon_range, triangles, epsilon);
        Self::clip_if_degenerate(new_start, polygon, polygon_range, triangles, epsilon);
        Self::clip_if_degenerate(connector, polygon, polygon_range, triangles, epsilon);
        Self::clip_if_degenerate(new_connector, polygon, polygon_range, triangles, epsilon);
    }

    fn process_ear(
        v: usize,
        collider: &IdxCollider,
        ears_queue: &mut VecDeque<usize>,
        polygon: &mut Vec<Vert>,
        epsilon: f64,
    ) {
        if polygon[v].ear {
            ears_queue.remove(ears_queue.iter().position(|&ball| ball == v).unwrap());
            polygon[v].ear = false;
        }

        if polygon[v].is_short(epsilon, polygon) {
            polygon[v].cost = K_BEST;
            polygon[v].ear = true;
            ears_queue.insert_sorted_by_key(v, |&ear| OrderedF64(polygon[ear].cost)); //ascending cost
        } else if polygon[v].is_convex(2.0 * epsilon, polygon) {
            polygon[v].cost = polygon[v].ear_cost(epsilon, collider, polygon);
            polygon[v].ear = true;
            ears_queue.insert_sorted_by_key(v, |&ear| OrderedF64(polygon[ear].cost)); //ascending cost
        } else {
            polygon[v].cost = 1.0; // not used, but marks reflex verts for debug
        }
    }

    ///Create a collider of all vertices in this polygon, each expanded by
    ///epsilon_. Each ear uses this BVH to quickly find a subset of vertices to
    ///check for cost.
    fn vert_collider(start: usize, polygon: &mut Vec<Vert>, polygon_range: &Range<usize>) -> IdxCollider {
        let mut itr = Vec::new();
        let mut points = Vec::new();
        Self::loop_verts(start, polygon, polygon_range, |v, polygon| {
            points.push(PolyVert {
                pos: polygon[v].pos,
                idx: itr.len() as i32,
            });

            itr.push(v);
        });

        build_2d_tree(&mut points);
        IdxCollider { points, itr }
    }

    ///The main ear-clipping loop. This is called once for each simple polygon -
    ///all holes have already been key-holed and joined to an outer polygon.
    fn triangulate_poly(
        start: usize,
        polygon: &mut Vec<Vert>,
        polygon_range: &Range<usize>,
        triangles: &mut Vec<Vector3<i32>>,
        epsilon: f64,
    ) {
        let vert_collider = Self::vert_collider(start, polygon, polygon_range);

        if vert_collider.itr.is_empty() {
            //empty poly
            return;
        }

        // A simple polygon always creates two fewer triangles than it has verts.
        let mut num_tri = -2;

        // A priority queue of valid ears - the multiset allows them to be updated.
        //c++ uses multiset here, whose big o complexity is probably more desirable here
        let mut ears_queue = VecDeque::new();

        let queue_vert = |v, polygon: &mut Vec<Vert>| {
            Self::process_ear(v, &vert_collider, &mut ears_queue, polygon, epsilon);
            num_tri += 1;
        };

        let v = Self::loop_verts(start, polygon, polygon_range, queue_vert);
        if v.is_none() {
            return;
        }
        let mut v = v.unwrap();

        while num_tri > 0 {
            let ear = ears_queue.front();
            if let Some(&ear) = ear {
                v = ear;
                // Cost should always be negative, generally < -epsilon.
                ears_queue.pop_front();
            } else {
                //no ear found!
            }

            Self::clip_ear(v, polygon, polygon_range, triangles);
            num_tri -= 1;

            let ear_left = polygon[v].left(polygon).ptr2index(polygon_range);
            Self::process_ear(ear_left, &vert_collider, &mut ears_queue, polygon, epsilon);
            let ear_right = polygon[v].right(polygon).ptr2index(polygon_range);
            Self::process_ear(ear_right, &vert_collider, &mut ears_queue, polygon, epsilon);
            // This is a backup vert that is used if the queue is empty (geometrically
            // invalid polygon), to ensure manifoldness.
            v = ear_right;
        }

        debug_assert!(polygon[v].right_idx == polygon[v].left_idx, "Triangulator error!");
        //finished poly
    }
}

struct IdxCollider {
    points: Vec<PolyVert>,
    itr: Vec<usize>,
}

/// A vertex in a circular doubly-linked list representing the polygon(s) that 
/// still need to be triangulated.
/// 
/// This struct forms a circular doubly-linked list where each vertex has indices
/// to its left and right neighbors. The list gets smaller as ears are clipped 
/// during triangulation until it degenerates to two points and terminates.
/// 
/// The `left_idx` and `right_idx` fields are indices into the polygon vector
/// that stores all vertices. This allows for safe access patterns without raw pointers.
/// The `self_idx` field stores the index of this vertex in the polygon vector.
/// 
/// Note that `left_idx` and `right_idx` could refer to the same vertex index, as 
/// evidenced by the C++ code: `if (v->right == v->left)`.
#[derive(Clone)]
struct Vert {
    /// The mesh index this vertex belongs to
    mesh_idx: i32,
    /// The cost associated with this vertex (used in triangulation)
    cost: f64,
    /// Whether this vertex forms an ear (used in triangulation)
    ear: bool,
    /// The 2D position of this vertex
    pos: Point2<f64>,
    /// The direction vector to the right neighbor
    right_dir: Vector2<f64>,
    /// Index of the left neighbor vertex in the polygon vector
    left_idx: usize,
    /// Index of the right neighbor vertex in the polygon vector
    right_idx: usize,
    /// Index of this vertex in the polygon vector
    self_idx: usize,
    //note left_idx and right_idx could refer to the same vert,
    //as evidenced by c++: if (v->right == v->left)
}

impl Vert {
    /// Returns a reference to the left neighbor vertex.
    /// 
    /// # Arguments
    /// * `polygon` - Reference to the polygon vector containing all vertices
    /// 
    /// # Returns
    /// A reference to the left neighbor vertex
    fn left<'a>(&self, polygon: &'a [Self]) -> &'a Self {
        &polygon[self.left_idx]
    }

    /// Returns a reference to the right neighbor vertex.
    /// 
    /// # Arguments
    /// * `polygon` - Reference to the polygon vector containing all vertices
    /// 
    /// # Returns
    /// A reference to the right neighbor vertex
    fn right<'a>(&self, polygon: &'a [Self]) -> &'a Self {
        &polygon[self.right_idx]
    }

    /// Returns a mutable reference to the left neighbor vertex.
    /// 
    /// # Arguments
    /// * `polygon` - Mutable reference to the polygon vector containing all vertices
    /// 
    /// # Returns
    /// A mutable reference to the left neighbor vertex
    #[allow(dead_code)]
    fn left_mut<'a>(&mut self, polygon: &'a mut [Self]) -> &'a mut Self {
        &mut polygon[self.left_idx]
    }

    /// Returns a mutable reference to the right neighbor vertex.
    /// 
    /// # Arguments
    /// * `polygon` - Mutable reference to the polygon vector containing all vertices
    /// 
    /// # Returns
    /// A mutable reference to the right neighbor vertex
    #[allow(dead_code)]
    fn right_mut<'a>(&mut self, polygon: &'a mut [Self]) -> &'a mut Self {
        &mut polygon[self.right_idx]
    }

    //safety: this is only meant for assigning left+right fields, which should
    //only be dereferenced through the accessor methods in order to let the
    //borrow checker do its job
    #[allow(dead_code)]
    fn index2ptr(index: usize, polygon_range: &Range<usize>) -> *mut Vert {
        assert!(index < polygon_range.len());
        (polygon_range.start + index * mem::size_of::<Vert>()) as *mut Vert
    }

    fn ptr2index(&self, _polygon_range: &Range<usize>) -> usize {
        // Simply return the stored index of this vertex
        self.self_idx
    }

    fn is_short(&self, epsilon: f64, polygon: &[Vert]) -> bool {
        let edge = self.right(polygon).pos - self.pos;
        edge.magnitude_squared() * 4.0 < epsilon.powi(2)
    }

    ///Returns true if Vert is on the inside of the edge that goes from tail to
    ///tail->right. This will walk the edges if necessary until a clear answer
    ///is found (beyond epsilon). If toLeft is true, this Vert will walk its
    ///edges to the left. This should be chosen so that the edges walk in the
    ///same general direction - tail always walks to the right.
    fn inside_edge(&self, tail: &Vert, epsilon: f64, to_left: bool, polygon: &[Vert]) -> bool {
        let p2 = epsilon.powi(2);
        let mut next_l = self.left(polygon).right(polygon);
        let mut next_r = tail.right(polygon);
        let mut center = tail;
        let mut last = center;

        while !ptr::eq(next_l, next_r) && !ptr::eq(tail, next_r) &&
                !ptr::eq(next_l, if to_left { self.right(polygon) } else { self.left(polygon) })
        {
            let edge_l = next_l.pos - center.pos;
            let l2 = edge_l.magnitude_squared();
            if l2 <= p2 {
                next_l = if to_left { next_l.left(polygon) } else { next_l.right(polygon) };
                continue;
            }

            let edge_r = next_r.pos - center.pos;
            let r2 = edge_r.magnitude_squared();
            if r2 <= p2 {
                next_r = next_r.right(polygon);
                continue;
            }

            let vec_lr = next_r.pos - next_l.pos;
            let lr2 = vec_lr.magnitude_squared();
            if lr2 <= p2 {
                last = center;
                center = next_l;
                next_l = if to_left { next_l.left(polygon) } else { next_l.right(polygon) };
                if ptr::eq(next_l, next_r) { break; }
                next_r = next_r.right(polygon);
                continue;
            }

            let mut convexity = ccw(next_l.pos, center.pos, next_r.pos, epsilon);
            if !ptr::eq(center, last) {
                convexity += ccw(last.pos, center.pos, next_l.pos, epsilon) +
                        ccw(next_r.pos, center.pos, last.pos, epsilon);
            }

            if convexity != 0 { return convexity > 0; }

            if l2 < r2 {
                center = next_l;
                next_l = if to_left { next_l.left(polygon) } else { next_l.right(polygon) };
            } else {
                center = next_r;
                next_r = next_r.right(polygon);
            }

            last = center;
        }

        // The whole polygon is degenerate - consider this to be convex.
        true
    }

    ///Returns true for convex or colinear ears.
    fn is_convex(&self, epsilon: f64, polygon: &[Vert]) -> bool {
        ccw(self.left(polygon).pos, self.pos, self.right(polygon).pos, epsilon) >= 0
    }

    ///Subtly different from !IsConvex because IsConvex will return true for
    ///colinear non-folded verts, while IsReflex will always check until actual
    ///certainty is determined.
    fn is_reflexive(&self, epsilon: f64, polygon: &[Vert]) -> bool {
        let left = self.left(polygon);
        !left.inside_edge(left.right(polygon), epsilon, true, polygon)
    }

    fn interp_y2x(&self, start: Point2<f64>, on_top: i32, epsilon: f64, polygon: &[Vert]) -> f64 {
        let right_pos_y = self.right(polygon).pos.y;
        if (self.pos.y - start.y).abs() <= epsilon {
            if right_pos_y <= start.y + epsilon || on_top == 1 {
                f64::NAN
            } else {
                self.pos.x
            }
        } else if self.pos.y < start.y - epsilon {
            if right_pos_y > start.y + epsilon {
                self.pos.x
                    + (start.y - self.pos.y) * (self.right(polygon).pos.x - self.pos.x)
                        / (right_pos_y - self.pos.y)
            } else if right_pos_y < start.y - epsilon || on_top == -1 {
                f64::NAN
            } else {
                self.right(polygon).pos.x
            }
        } else {
            f64::NAN
        }
    }

    ///This finds the cost of this vert relative to one of the two closed sides
    ///of the ear. Points are valid even when they touch, so long as their edge
    ///goes to the outside. No need to check the other side, since all verts are
    ///processed in the EarCost loop.
    fn signed_dist(&self, v: &Vert, unit: Vector2<f64>, epsilon: f64, polygon: &[Vert]) -> f64 {
        let d = Matrix2::from_columns(&[unit, v.pos - self.pos]).determinant();
        if d.abs() < epsilon {
            let d_r = Matrix2::from_columns(&[unit, v.right(polygon).pos - self.pos]).determinant();
            if d_r.abs() > epsilon {
                return d_r;
            }
            let d_l = Matrix2::from_columns(&[unit, v.left(polygon).pos - self.pos]).determinant();
            if d_l.abs() > epsilon {
                return d_l;
            }
        }

        d
    }

    ///Find the cost of Vert v within this ear, where openSide is the unit
    ///vector from Verts right to left - passed in for reuse.
    fn cost(&self, v: &Vert, open_side: Vector2<f64>, epsilon: f64, polygon: &[Vert]) -> f64 {
        let cost = self
            .signed_dist(v, self.right_dir, epsilon, polygon)
            .min(self.signed_dist(v, self.left(polygon).right_dir, epsilon, polygon));

        let open_cost = Matrix2::from_columns(&[open_side, v.pos - self.right(polygon).pos]).determinant();
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
    fn ear_cost(&self, epsilon: f64, collider: &IdxCollider, polygon: &[Vert]) -> f64 {
        let left_pos = self.left(polygon).pos;
        let right_pos = self.right(polygon).pos;

        let mut open_side = left_pos - right_pos;
        let center = nalgebra::center(&left_pos, &right_pos);
        let scale = 4.0 / open_side.magnitude_squared();
        let radius = open_side.magnitude() / 2.0;
        open_side = open_side.normalize();

        let mut total_cost = self.left(polygon).right_dir.dot(&self.right_dir) - 1.0 - epsilon;
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

        let lid = self.left(polygon).mesh_idx;
        let rid = self.right(polygon).mesh_idx;
        query_2d_tree(&collider.points, ear_box, |point| {
            let test = &polygon[collider.itr[point.idx as usize]];
            if !EarClip::clipped(test, polygon)
                && test.mesh_idx != self.mesh_idx
                && test.mesh_idx != lid
                && test.mesh_idx != rid
            {
                // Skip duplicated verts
                let mut cost = self.cost(test, open_side, epsilon, polygon);
                if cost < -epsilon {
                    cost = Self::delaunay_cost(test.pos - center, scale, epsilon);
                }

                if cost > total_cost {
                    total_cost = cost;
                }
            }
        });

        total_cost
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