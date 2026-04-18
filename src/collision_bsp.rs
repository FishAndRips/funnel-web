//! Provides functions for traversing BSPs.

use tinyvec::ArrayVec;
use crate::float::FloatOps;
use crate::vector::{Plane2D, Plane3D, Vector2D, Vector3D, Vector3DComponent};

/// Functions for traversing BSPs.
///
/// Simply implement all required functions for your struct.
pub trait CollisionBSPFunctions {
    /// Get the 3D node at the given index `node`.
    fn get_3d_node(&self, node: usize) -> Option<CollisionBSP3DNode>;

    /// Return the number of BSP3D nodes.
    fn get_3d_node_count(&self) -> usize;

    /// Get the plane at the given index `plane`.
    fn get_plane(&self, plane: usize) -> Option<Plane3D>;

    /// Return the number of planes.
    fn get_plane_count(&self) -> usize;

    /// Get the leaf at the given index `leaf`.
    fn get_leaf(&self, leaf: usize) -> Option<CollisionBSPLeaf>;

    /// Return the number of leaves.
    fn get_leaf_count(&self) -> usize;

    /// Get the 2D node reference at the given index `node`.
    fn get_2d_node_reference(&self, node: usize) -> Option<BSP2DNodeReference>;

    /// Get the number of 2D node references.
    fn get_2d_node_reference_count(&self) -> usize;

    /// Get the 2D node at the given index `node`.
    fn get_2d_node(&self, node: usize) -> Option<CollisionBSP2DNode>;

    /// Get the number of 2D nodes.
    fn get_2d_node_count(&self) -> usize;

    /// Get the surface at the given index `surface`.
    fn get_surface(&self, surface: usize) -> Option<CollisionBSPSurface>;

    /// Get the number of surfaces.
    fn get_surface_count(&self) -> usize;

    /// Get the edge at the given index `edge`.
    fn get_edge(&self, edge: usize) -> Option<CollisionBSPEdge>;

    /// Get the number of edges.
    fn get_edge_count(&self) -> usize;

    /// Get the vertex at the given index `vertex`.
    fn get_vertex(&self, vertex: usize) -> Option<CollisionBSPVertex>;

    /// Get the number of vertices.
    fn get_vertex_count(&self) -> usize;

    /// Test a point to see if it is in the BSP.
    ///
    /// Return values:
    /// - `Ok(Some(true))` if the point is in the BSP
    /// - `Ok(Some(false))` if the point is outside the BSP
    /// - `Err(_)` if the BSP is malformed
    fn point_inside_bsp(&self, point: Vector3D) -> Result<bool, CollisionBSPError> {
        self.leaf_index_for_point_3d(point)
            .map(|e| e.is_some())
    }

    /// Return the leaf index for the given point.
    ///
    /// Return values:
    /// - `Ok(Some(_))` if the point is in the BSP
    /// - `Ok(None)` if the point is outside the BSP
    /// - `Err(_)` if the BSP is malformed
    fn leaf_index_for_point_3d(&self, point: Vector3D) -> Result<Option<usize>, CollisionBSPError> {
        let mut index = 0usize;

        for _ in 0..self.get_3d_node_count().max(1) {
            let node = checked_get_bsp_3d_node(self, index)?;
            let plane = checked_get_bsp_plane(self, node.plane_index)?;

            let next = if plane.distance_to_point(point) >= 0.0 {
                node.front_child
            }
            else {
                node.back_child
            };

            match next.as_tuple() {
                Some((CollisionBSP3DNodeIndexType::Node, next_index)) => index = next_index,
                Some((CollisionBSP3DNodeIndexType::Leaf, leaf)) => return Ok(Some(leaf)),
                None => return Ok(None)
            }
        }

        Err(CollisionBSPError::BSP3DNodeLoop(index))
    }

    /// Get the leaf index.
    fn leaf_index_for_point_2d(&self, point: Vector2D, starting_index: CollisionBSP2DNodeIndex) -> Result<Option<usize>, CollisionBSPError> {
        let mut index = starting_index;
        loop {
            let Some((index_type, index_value)) = index.as_tuple() else {
                return Ok(None)
            };

            if index_type != CollisionBSP2DNodeIndexType::Node {
                return Ok(Some(index_value))
            }

            let node = checked_get_bsp_2d_node(self, index_value)?;
            let distance = node.plane.distance_to_point(point);
            index = if distance >= 0.0 { node.right_child } else { node.left_child }
        }
    }

    /// Check the BSP for out-of-bounds errors.
    ///
    /// Returns `Ok(())` if no errors are detected and `Err(_)` if an error was found.
    fn bounds_check(&self) -> Result<(), CollisionBSPError> {
        let bsp_3d_node_count = self.get_3d_node_count();

        for node in 0..bsp_3d_node_count.max(1) {
            let node = checked_get_bsp_3d_node(self, node)?;
            let _plane = checked_get_bsp_plane(self, node.plane_index)?;

            let check = |a: CollisionBSP3DNodeIndex| {
                match a.as_tuple() {
                    Some((CollisionBSP3DNodeIndexType::Node, b)) => checked_get_bsp_3d_node(self, b).map(|_| ()),
                    Some((CollisionBSP3DNodeIndexType::Leaf, b)) => checked_get_bsp_leaf(self, b).map(|_| ()),
                    None => Ok(())
                }
            };

            check(node.front_child)?;
            check(node.back_child)?;
        }

        for p in 0..self.get_plane_count() {
            let _plane = checked_get_bsp_plane(self, p)?;
        }

        for l in 0..=self.get_leaf_count() {
            let leaf = checked_get_bsp_leaf(self, l)?;

            let end = leaf
                .bsp_2d_node_reference_start
                .checked_add(leaf.bsp_2d_node_reference_count)
                .ok_or(CollisionBSPError::BadLeaf(l))?;

            for r in leaf.bsp_2d_node_reference_start..end {
                let _reference = checked_get_bsp_2d_node_reference(self, r)?;
            }
        }

        for r in 0..self.get_2d_node_reference_count() {
            let reference = checked_get_bsp_2d_node_reference(self, r)?;
            let _plane = checked_get_bsp_plane(self, reference.plane)?;

            let _ = match reference.node.as_tuple() {
                Some((CollisionBSP2DNodeIndexType::Node, n)) => checked_get_bsp_2d_node(self, n).map(|_| ())?,
                Some((CollisionBSP2DNodeIndexType::Surface, _)) => return Err(CollisionBSPError::Bad2DReference(r)),
                None => ()
            };
        }

        for node in 0..self.get_2d_node_count() {
            let node = checked_get_bsp_2d_node(self, node)?;

            let check = |a: CollisionBSP2DNodeIndex| {
                match a.as_tuple() {
                    Some((CollisionBSP2DNodeIndexType::Node, b)) => checked_get_bsp_2d_node(self, b).map(|_| ()),
                    Some((CollisionBSP2DNodeIndexType::Surface, b)) => checked_get_bsp_surface(self, b).map(|_| ()),
                    None => Ok(())
                }
            };

            check(node.left_child)?;
            check(node.right_child)?;
        }

        for s in 0..self.get_surface_count() {
            let surface = checked_get_bsp_surface(self, s)?;
            let _plane = checked_get_bsp_plane(self, surface.plane)?;
            let _edge = checked_get_bsp_edge(self, surface.first_edge)?;
        }

        for e in 0..self.get_edge_count() {
            let edge = checked_get_bsp_edge(self, e)?;
            let _start_vertex = checked_get_bsp_vertex(self, edge.start_vertex)?;
            let _end_vertex = checked_get_bsp_vertex(self, edge.start_vertex)?;
            let _forward_edge = checked_get_bsp_edge(self, edge.forward_edge)?;
            let _reverse_edge = checked_get_bsp_edge(self, edge.reverse_edge)?;
            let _left_surface = checked_get_bsp_surface(self, edge.left_surface)?;
            let _right_surface = checked_get_bsp_surface(self, edge.right_surface)?;

        }

        Ok(())
    }

    /// Find a point where a line intersects on a BSP if it does.
    ///
    /// ## Remarks
    ///
    /// * `vector` should be premultiplied with distance (unit vectors only go to 1 world unit)
    /// * `relative_distance_max` is clamped between 0 and 1 (use 1 for the full distance)
    fn test_vector(
        &self,
        flags: CollisionBSPTestVectorFlags,
        breakable_surface_flags: &[bool],
        point: Vector3D,
        vector: Vector3D,
        relative_distance_max: f32
    ) -> Result<CollisionBSPTestVectorResult, CollisionBSPError> {
        let mut state = CollisionBSPTestVectorState {
            flags,
            breakable_surface_flags,
            point,
            vector,
            last_leaf_index: None,
            last_contents: ContentsState::Unknown,
            last_plane_index: None,
            result: CollisionBSPTestVectorResult {
                relative_distance: relative_distance_max.fw_floor(),
                hit_surface: None,
                leaf_indices: ArrayVec::new()
            },
        };

        let hit = test_vector_recursive(self, &mut state, CollisionBSP3DNodeIndex(0), 0.0, relative_distance_max.clamp(0.0, 1.0))?;

        if !hit {
            state.result.hit_surface = None;
        }
        else {
            assert!(state.result.hit_surface.is_some(), "should have a hit surface");
        }

        Ok(state.result)
    }
}

macro_rules! wrap_getter {
    ($getter_fn_name:tt, $inner_getter_name:tt, $expected_type:ty, $error_enum:tt) => {
        #[must_use]
        fn $getter_fn_name<BSP: CollisionBSPFunctions + ?Sized>(bsp: &BSP, index: usize) -> Result<$expected_type, CollisionBSPError> {
            bsp.$inner_getter_name(index).ok_or_else(|| CollisionBSPError::$error_enum(index))
        }
    };
}

wrap_getter!(checked_get_bsp_3d_node, get_3d_node, CollisionBSP3DNode, Missing3DNode);
wrap_getter!(checked_get_bsp_plane, get_plane, Plane3D, MissingPlane);
wrap_getter!(checked_get_bsp_leaf, get_leaf, CollisionBSPLeaf, MissingLeaf);
wrap_getter!(checked_get_bsp_2d_node_reference, get_2d_node_reference, BSP2DNodeReference, Missing2DNodeReference);
wrap_getter!(checked_get_bsp_2d_node, get_2d_node, CollisionBSP2DNode, Missing2DNode);
wrap_getter!(checked_get_bsp_surface, get_surface, CollisionBSPSurface, MissingSurface);
wrap_getter!(checked_get_bsp_edge, get_edge, CollisionBSPEdge, MissingEdge);
wrap_getter!(checked_get_bsp_vertex, get_vertex, CollisionBSPVertex, MissingVertex);

fn test_vector_recursive<BSP: CollisionBSPFunctions + ?Sized>(
    bsp: &BSP,
    data: &mut CollisionBSPTestVectorState,
    child_index: CollisionBSP3DNodeIndex,
    relative_distance_min: f32,
    relative_distance_max: f32
) -> Result<bool, CollisionBSPError> {
    let (child_index_type, child_index) = match child_index.as_tuple() {
        Some(n) => (n.0, Some(n.1)),
        None => (CollisionBSP3DNodeIndexType::Leaf, None)
    };

    match child_index_type {
        CollisionBSP3DNodeIndexType::Node => {
            let node = checked_get_bsp_3d_node(bsp, child_index.expect("child_index should be valid for Node"))?;
            let plane = checked_get_bsp_plane(bsp, node.plane_index)?;
            let plane_distance = plane.distance_to_point(data.point);
            let scale = data.vector.dot(plane.vector);

            let distance_back = plane_distance + scale * relative_distance_min;
            let distance_front = plane_distance + scale * relative_distance_max;
            let below = distance_back < 0.0 || distance_front < 0.0;
            let above = distance_back >= 0.0 || distance_front >= 0.0;

            if below && above {
                let reversed = scale > 0.0;

                let (before, after) = if reversed {
                    (node.back_child, node.front_child)
                } else {
                    (node.front_child, node.back_child)
                };

                let relative_distance = -plane_distance / scale;

                if test_vector_recursive(bsp, data, before, relative_distance_min, relative_distance)? {
                    return Ok(true)
                }

                if data.result.relative_distance <= relative_distance {
                    return Ok(false)
                }

                data.last_plane_index = Some(node.plane_index);
                test_vector_recursive(bsp, data, after, relative_distance, relative_distance_max)
            }
            else {
                let test = if above { node.back_child } else { node.front_child };
                test_vector_recursive(bsp, data, test, relative_distance_min, relative_distance_max)
            }
        },
        CollisionBSP3DNodeIndexType::Leaf => {
            let leaf_index = child_index;
            let mut contents = ContentsState::Solid;
            let mut test_leaf_index = None;
            let mut test_surface_flag = false;

            if let Some(leaf_index) = leaf_index {
                contents = match checked_get_bsp_leaf(bsp, leaf_index)?.contains_double_sided_surfaces {
                    true => ContentsState::SemiEmpty,
                    false => ContentsState::Empty
                }
            }

            if data.flags.test_front_facing_surfaces && leaf_index.is_none() && matches!(data.last_contents, ContentsState::Empty | ContentsState::SemiEmpty) {
                test_leaf_index = data.last_leaf_index;
            }
            else if data.flags.test_back_facing_surfaces && leaf_index.is_some() && data.last_contents == ContentsState::Solid {
                test_leaf_index = leaf_index;
            }
            else if !data.flags.ignore_two_sided_surfaces && contents == ContentsState::SemiEmpty && data.last_contents == ContentsState::SemiEmpty {
                test_leaf_index = match data.flags.test_front_facing_surfaces {
                    true => data.last_leaf_index,
                    false => leaf_index
                };
                test_surface_flag = true
            };

            if let Some(test_leaf_index) = test_leaf_index {
                if let Some(surface_index) = test_leaf_vector(bsp, data.breakable_surface_flags, data.point, data.vector, test_leaf_index, data.last_plane_index, relative_distance_min, test_surface_flag)? {
                    let surface = checked_get_bsp_surface(bsp, surface_index)?;

                    if (!surface.flags.invisible || !data.flags.ignore_invisible_surfaces) && (!surface.flags.breakable || !data.flags.ignore_breakable_surfaces) {
                        let plane_index = data.last_plane_index.expect("should have a plane index if hit");

                        data.result.relative_distance = relative_distance_min;
                        data.result.hit_surface = Some(CollisionBSPTestVectorSurfaceIndices {
                            surface_index,
                            plane_index
                        });

                        return Ok(true);
                    }
                }
            }

            if let Some(leaf_index) = leaf_index {
                if data.result.leaf_indices.try_push(leaf_index).is_none() {
                    *data.result.leaf_indices.last_mut().expect("reached the end") = leaf_index;
                }
            }

            data.last_leaf_index = leaf_index;
            data.last_contents = contents;

            Ok(false)
        }
    }
}

fn test_leaf_vector<BSP: CollisionBSPFunctions + ?Sized>(
    bsp: &BSP,
    breakable_surfaces: &[bool],
    point: Vector3D,
    vector: Vector3D,
    leaf_index: usize,
    plane_index: Option<usize>,
    relative_distance: f32,
    test_surface_flag: bool
) -> Result<Option<usize>, CollisionBSPError> {
    let leaf = checked_get_bsp_leaf(bsp, leaf_index)?;
    let Some(plane_index) = plane_index else {
        return Ok(None)
    };
    let plane = checked_get_bsp_plane(bsp, plane_index)?;
    let point = point.apply_offset(vector, relative_distance);

    for reference in (leaf.bsp_2d_node_reference_start..).take(leaf.bsp_2d_node_reference_count) {
        let reference = checked_get_bsp_2d_node_reference(bsp, reference)?;
        if reference.plane != plane_index {
            continue
        }

        let projection = plane.vector.projection();
        let projection_positive = plane.vector.component_is_positive(projection);
        let project = point.project(projection, projection_positive);
        let surface_index = bsp.leaf_index_for_point_2d(project, reference.node)?;

        let Some(surface_index) = surface_index else {
            // Official, release builds of the game do UB (accesses surface[-1]) if this happens,
            // and then you die.
            //
            // We'll just say it's outside of the BSP...
            return Ok(None)
        };

        if !test_surface_flag || surface_test_point(bsp, breakable_surfaces, surface_index, projection, projection_positive, project)? {
            return Ok(Some(surface_index))
        }
    }

    Ok(None)
}

fn surface_test_point<BSP: CollisionBSPFunctions + ?Sized>(
    bsp: &BSP,
    breakable_surfaces: &[bool],
    surface_index: usize,
    projection: Vector3DComponent,
    projection_positive: bool,
    point: Vector2D
) -> Result<bool, CollisionBSPError> {
    let surface = checked_get_bsp_surface(bsp, surface_index)?;

    if surface.flags.breakable
        && let Some(breakable_surface_index) = surface.breakable_surface_index
        && breakable_surfaces.get(breakable_surface_index).copied() == Some(false) {
        return Ok(false)
    }

    // TODO: we should probably have infinite loop protection in the checker function instead
    let mut infinite_loop_protection = 1000 * 1000 * 1000;

    let mut edge_index = surface.first_edge;
    loop {
        infinite_loop_protection -= 1;
        if infinite_loop_protection == 0 {
            return Err(CollisionBSPError::OtherError("infinite edge loop"))
        }

        let edge = checked_get_bsp_edge(bsp, edge_index)?;
        let reversed = edge.right_surface == surface_index;

        let mut start_vertex = checked_get_bsp_vertex(bsp, edge.start_vertex)?;
        let mut end_vertex = checked_get_bsp_vertex(bsp, edge.end_vertex)?;

        if reversed {
            core::mem::swap(&mut start_vertex, &mut end_vertex);
        }

        let project_start = start_vertex.point.project(projection, projection_positive);
        let project_end = end_vertex.point.project(projection, projection_positive);

        let vector_start = point - project_start;
        let vector_end = project_end - project_start;

        if vector_start.cross_product(vector_end) > 0.0 {
            return Ok(false)
        }

        edge_index = if reversed { edge.reverse_edge } else { edge.forward_edge };

        if edge_index == surface.first_edge {
            break
        }
    }

    Ok(true)
}

#[repr(u8)]
#[derive(PartialEq)]
enum ContentsState {
    Unknown,
    Empty,
    SemiEmpty,
    Solid
}

/// Result of [`CollisionBSPFunctions::test_vector`].
pub struct CollisionBSPTestVectorResult {
    /// Relative distance along the vector between 0.0 and 1.0
    pub relative_distance: f32,

    /// Surface indices
    pub hit_surface: Option<CollisionBSPTestVectorSurfaceIndices>,

    /// Tested leaf indices
    pub leaf_indices: ArrayVec<[usize; 256]>
}

/// Surface indices of a hit surface when testing a vector.
#[derive(Copy, Clone, PartialEq)]
pub struct CollisionBSPTestVectorSurfaceIndices {
    /// Surface index hit
    pub surface_index: usize,

    /// Plane index hit
    pub plane_index: usize
}

struct CollisionBSPTestVectorState<'a> {
    flags: CollisionBSPTestVectorFlags,
    breakable_surface_flags: &'a [bool],
    point: Vector3D,
    vector: Vector3D,
    last_leaf_index: Option<usize>,
    last_contents: ContentsState,
    last_plane_index: Option<usize>,
    result: CollisionBSPTestVectorResult,
}

#[derive(Default)]
#[expect(missing_docs)]
pub struct CollisionBSPTestVectorFlags {
    pub test_front_facing_surfaces: bool,
    pub test_back_facing_surfaces: bool,
    pub ignore_two_sided_surfaces: bool,
    pub ignore_invisible_surfaces: bool,
    pub ignore_breakable_surfaces: bool
}

/*
boolean collision_bsp_test_vector(unsigned long flags, const struct collision_bsp *bsp, short breakable_surface_count, const byte *breakable_surface_flags, const real_point3d *point, const real_vector3d *vector, real maximum_t, struct collision_bsp_test_vector_result *result);
static boolean collision_bsp_test_vector_recursive(struct test_vector_data *data, long child_index, real t0, real t1);
 */

/// An error returned from [`CollisionBSPFunctions`].
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum CollisionBSPError {
    /// An infinite loop occurred. The BSP is malformed.
    BSP3DNodeLoop(usize),

    /// A 3D node is missing. The BSP is malformed.
    Missing3DNode(usize),

    /// A 2D node is missing. The BSP is malformed.
    Missing2DNode(usize),

    /// A 2D node reference is missing. The BSP is malformed.
    Missing2DNodeReference(usize),

    /// The given leaf has a bad (overflowing) reference. The BSP is malformed.
    BadLeaf(usize),

    /// The given 2D reference has a bad reference. The BSP is malformed.
    Bad2DReference(usize),

    /// A plane is missing. The BSP is malformed.
    MissingPlane(usize),

    /// A leaf is missing. The BSP is malformed.
    MissingLeaf(usize),

    /// A surface is missing. The BSP is malformed.
    MissingSurface(usize),

    /// An edge is missing. The BSP is malformed.
    MissingEdge(usize),

    /// A vertex is missing. The BSP is malformed.
    MissingVertex(usize),

    /// An unspecified error occurred.
    OtherError(&'static str)
}

impl core::fmt::Display for CollisionBSPError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CollisionBSPError::BSP3DNodeLoop(n) => f.write_fmt(format_args!("An infinite BSP3D node loop occurred in the BSP including 3D node #{n}; BSP is malformed")),
            CollisionBSPError::BadLeaf(n) => f.write_fmt(format_args!("Leaf #{n} has a bad reference; BSP is malformed")),
            CollisionBSPError::Bad2DReference(n) => f.write_fmt(format_args!("2D reference #{n} has a bad reference; BSP is malformed")),
            CollisionBSPError::Missing3DNode(n) => f.write_fmt(format_args!("3D node #{n} not found in the BSP; BSP is malformed")),
            CollisionBSPError::Missing2DNode(n) => f.write_fmt(format_args!("2D node #{n} not found in the BSP; BSP is malformed")),
            CollisionBSPError::Missing2DNodeReference(n) => f.write_fmt(format_args!("2D node reference #{n} not found in the BSP; BSP is malformed")),
            CollisionBSPError::MissingPlane(n) => f.write_fmt(format_args!("Plane #{n} not found in the BSP; BSP is malformed")),
            CollisionBSPError::MissingLeaf(n) => f.write_fmt(format_args!("Leaf #{n} not found in the BSP; BSP is malformed")),
            CollisionBSPError::MissingSurface(n) => f.write_fmt(format_args!("Surface #{n} not found in the BSP; BSP is malformed")),
            CollisionBSPError::MissingEdge(n) => f.write_fmt(format_args!("Edge #{n} not found in the BSP; BSP is malformed")),
            CollisionBSPError::MissingVertex(n) => f.write_fmt(format_args!("Vertex #{n} not found in the BSP; BSP is malformed")),
            CollisionBSPError::OtherError(o) => f.write_str(o),
        }
    }
}

/// A struct containing all fields for a collision BSP 3D node.
///
/// This does not correspond, bitwise, to the actual type, but it has all of its fields.
///
/// When implementing [`CollisionBSPFunctions`], you will need to implement a function that returns
/// this.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct CollisionBSP3DNode {
    /// The node/leaf behind this node.
    pub back_child: CollisionBSP3DNodeIndex,
    /// The node/leaf in front of this node.
    pub front_child: CollisionBSP3DNodeIndex,
    /// The index of the plane this node is on.
    pub plane_index: usize
}

/// An index for 3D nodes.
///
/// The uppermost bit is used to identify if the remaining 31 bits are used for a node or leaf
/// index.
///
/// `0xFFFFFFFF` has a special meaning in that it refers to nothing.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct CollisionBSP3DNodeIndex(pub u32);
impl CollisionBSP3DNodeIndex {
    /// Split the index into a tuple.
    ///
    /// Returns `None` if this is a null index.
    #[must_use] 
    pub const fn as_tuple(self) -> Option<(CollisionBSP3DNodeIndexType, usize)> {
        if self.0 == 0xFFFFFFFF {
            return None
        }

        let index = self.0 & 0x7FFFFFFF;
        let index_type = if index == self.0 {
            CollisionBSP3DNodeIndexType::Node
        }
        else {
            CollisionBSP3DNodeIndexType::Leaf
        };

        Some((index_type, index as usize))
    }
}

/// An enum returned from [`CollisionBSP3DNodeIndex::as_tuple`].
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum CollisionBSP3DNodeIndexType {
    /// Represents another 3D node.
    Node,
    /// Represents a 3D leaf
    Leaf
}

/// An index for 2D nodes.
///
/// The uppermost bit is used to identify if the remaining 31 bits are used for a node or surface
/// index.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct CollisionBSP2DNodeIndex(pub u32);
impl CollisionBSP2DNodeIndex {
    /// Split the index into a tuple.
    #[must_use] 
    pub const fn as_tuple(self) -> Option<(CollisionBSP2DNodeIndexType, usize)> {
        if self.0 == 0xFFFFFFFF {
            return None
        }

        let index = self.0 & 0x7FFFFFFF;
        let index_type = if index == self.0 {
            CollisionBSP2DNodeIndexType::Node
        }
        else {
            CollisionBSP2DNodeIndexType::Surface
        };

        Some((index_type, index as usize))
    }
}

/// An enum returned from [`CollisionBSP2DNodeIndex::as_tuple`].
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum CollisionBSP2DNodeIndexType {
    /// Represents a 2D node.
    Node,
    /// Represents a surface.
    Surface
}

/// A struct containing all fields for a collision BSP leaf.
///
/// This does not correspond, bitwise, to the actual type, but it has all of its fields.
///
/// When implementing [`CollisionBSPFunctions`], you will need to implement a function that returns
/// this.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
#[expect(missing_docs)] // TODO
pub struct CollisionBSPLeaf {
    pub contains_double_sided_surfaces: bool,
    pub bsp_2d_node_reference_start: usize,
    pub bsp_2d_node_reference_count: usize
}

/// A struct containing all fields for a collision BSP 2D node reference.
///
/// This does not correspond, bitwise, to the actual type, but it has all of its fields.
///
/// When implementing [`CollisionBSPFunctions`], you will need to implement a function that returns
/// this.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct BSP2DNodeReference {
    /// Represents the plane.
    pub plane: usize,

    /// Represents the first node.
    pub node: CollisionBSP2DNodeIndex
}

/// A struct containing all fields for a collision BSP 2D node.
///
/// This does not correspond, bitwise, to the actual type, but it has all of its fields.
///
/// When implementing [`CollisionBSPFunctions`], you will need to implement a function that returns
/// this.
#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
#[expect(missing_docs)] // TODO
pub struct CollisionBSP2DNode {
    pub plane: Plane2D,
    pub left_child: CollisionBSP2DNodeIndex,
    pub right_child: CollisionBSP2DNodeIndex,
}


/// A struct containing all fields for a collision BSP surface.
///
/// This does not correspond, bitwise, to the actual type, but it has all of its fields.
///
/// When implementing [`CollisionBSPFunctions`], you will need to implement a function that returns
/// this.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CollisionBSPSurface {
    /// Plane index the surface lies on.
    pub plane: usize,

    /// First edge for this surface.
    pub first_edge: usize,

    /// Flags for BSP surfaces.
    pub flags: CollisionBSPSurfaceFlags,

    /// Material index.
    // TODO: This should be an enum?
    pub material: u16,

    /// Breakable surface index.
    pub breakable_surface_index: Option<usize>,
}

/// Flags for [`CollisionBSPSurface`].
#[derive(Copy, Clone, PartialEq, Debug)]
#[expect(missing_docs)]
pub struct CollisionBSPSurfaceFlags {
    pub two_sided: bool,
    pub invisible: bool,
    pub climbable: bool,
    pub breakable: bool
}

/// Edge in a collision BSP.
#[derive(Copy, Clone, PartialEq, Debug)]
#[expect(missing_docs)]
pub struct CollisionBSPEdge {
    pub start_vertex: usize,
    pub end_vertex: usize,
    pub forward_edge: usize,
    pub reverse_edge: usize,
    pub left_surface: usize,
    pub right_surface: usize
}

/// Vertex in a collision BSP.
#[derive(Copy, Clone, PartialEq, Debug)]
#[expect(missing_docs)]
pub struct CollisionBSPVertex {
    pub point: Vector3D,
    pub first_edge: usize,
}
