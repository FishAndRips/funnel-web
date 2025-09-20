//! Provides functions for traversing BSPs.

use crate::vector::{Plane2D, Plane3D, Vector3D};

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

    /// Test a point to see if it is in the BSP.
    ///
    /// Return values:
    /// - `Ok(Some(true))` if the point is in the BSP
    /// - `Ok(Some(false))` if the point is outside the BSP
    /// - `Err(_)` if the BSP is malformed
    fn point_inside_bsp(&self, point: &Vector3D) -> Result<bool, CollisionBSPError> {
        self.leaf_index_for_point(point)
            .map(|e| e.is_some())
    }

    /// Return the leaf index for the given point.
    ///
    /// Return values:
    /// - `Ok(Some(_))` if the point is in the BSP
    /// - `Ok(None)` if the point is outside the BSP
    /// - `Err(_)` if the BSP is malformed
    fn leaf_index_for_point(&self, point: &Vector3D) -> Result<Option<usize>, CollisionBSPError> {
        let mut index = 0usize;

        for _ in 0..self.get_3d_node_count().max(1) {
            let node = self.get_3d_node(index)
                .ok_or(CollisionBSPError::Missing3DNode(index))?;
            let plane = self.get_plane(node.plane_index)
                .ok_or(CollisionBSPError::MissingPlane(node.plane_index))?;

            let next = if plane.distance_to_point(*point) >= 0.0 {
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

    /// Check the BSP for out-of-bounds errors.
    ///
    /// Returns `Ok(())` if no errors are detected and `Err(_)` if an error was found.
    fn bounds_check(&self) -> Result<(), CollisionBSPError> {
        let bsp_3d_node_count = self.get_3d_node_count();

        for node in 0..bsp_3d_node_count.max(1) {
            let node = self
                .get_3d_node(node)
                .ok_or(CollisionBSPError::Missing3DNode(node))?;

            self.get_plane(node.plane_index)
                .ok_or(CollisionBSPError::MissingPlane(node.plane_index))?;

            let check = |a: CollisionBSP3DNodeIndex| {
                match a.as_tuple() {
                    Some((CollisionBSP3DNodeIndexType::Node, b)) => self
                        .get_3d_node(b)
                        .ok_or(CollisionBSPError::Missing3DNode(b))
                        .map(|_| ()),
                    Some((CollisionBSP3DNodeIndexType::Leaf, b)) => self
                        .get_leaf(b)
                        .ok_or(CollisionBSPError::MissingLeaf(b))
                        .map(|_| ()),
                    None => Ok(())
                }
            };

            check(node.front_child)?;
            check(node.back_child)?;
        }

        for p in 0..self.get_plane_count() {
            self.get_plane(p)
                .ok_or(CollisionBSPError::MissingPlane(p))?;
        }

        for l in 0..=self.get_leaf_count() {
            let leaf = self.get_leaf(l)
                .ok_or(CollisionBSPError::MissingLeaf(l))?;

            let end = leaf
                .bsp_2d_node_reference_start
                .checked_add(leaf.bsp_2d_node_reference_count)
                .ok_or(CollisionBSPError::BadLeaf(l))?;

            for r in leaf.bsp_2d_node_reference_start..end {
                self.get_2d_node_reference(r).ok_or(CollisionBSPError::Missing2DNodeReference(r))?;
            }
        }

        for r in 0..self.get_2d_node_reference_count() {
            let reference = self.get_2d_node_reference(r).ok_or(CollisionBSPError::Missing2DNodeReference(r))?;
            self.get_plane(reference.plane).ok_or(CollisionBSPError::MissingPlane(reference.plane))?;
            match reference.node.as_tuple() {
                (CollisionBSP2DNodeIndexType::Node, n) => self.get_2d_node(n).ok_or(CollisionBSPError::Missing2DNode(n))?,
                (CollisionBSP2DNodeIndexType::Surface, _) => return Err(CollisionBSPError::Bad2DReference(r))
            };
        }

        for node in 0..self.get_2d_node_count() {
            let node = self.get_2d_node(node).ok_or(CollisionBSPError::Missing2DNode(node))?;

            let check = |a: CollisionBSP2DNodeIndex| {
                match a.as_tuple() {
                    (CollisionBSP2DNodeIndexType::Node, b) => self
                        .get_2d_node(b)
                        .ok_or(CollisionBSPError::Missing2DNode(b))
                        .map(|_| ()),
                    (CollisionBSP2DNodeIndexType::Surface, b) => self
                        .get_surface(b)
                        .ok_or(CollisionBSPError::MissingSurface(b))
                        .map(|_| ()),
                }
            };

            check(node.left_child)?;
            check(node.right_child)?;
        }

        for s in 0..self.get_surface_count() {
            let surface = self.get_surface(s).ok_or(CollisionBSPError::MissingSurface(s))?;
            self.get_plane(surface.plane).ok_or(CollisionBSPError::MissingPlane(surface.plane))?;
        }

        Ok(())
    }
}

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
    MissingSurface(usize)
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
    /// The node/leaf in front of this node.
    pub front_child: CollisionBSP3DNodeIndex,
    /// The node/leaf behind this node.
    pub back_child: CollisionBSP3DNodeIndex,
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
    pub const fn as_tuple(self) -> (CollisionBSP2DNodeIndexType, usize) {
        let index = self.0 & 0x7FFFFFFF;
        let index_type = if index == self.0 {
            CollisionBSP2DNodeIndexType::Node
        }
        else {
            CollisionBSP2DNodeIndexType::Surface
        };

        (index_type, index as usize)
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
#[repr(C)]
pub struct CollisionBSPSurface {
    /// Plane index the surface lies on.
    pub plane: usize,

    /// Material index.
    // TODO: This should be an enum?
    pub material: u16
}
