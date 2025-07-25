use glam::{Mat4, Quat, Vec3};

/// A 3D transformation consisting of position, rotation, and scale.
///
/// This struct provides methods for manipulating objects in 3D space with
/// support for both absolute positioning and incremental transformations.
/// Rotation is handled using quaternions to avoid gimbal lock.
///
/// # Examples
///
/// ```
/// use map::Transform;
/// use glam::Vec3;
///
/// // Create a new transform at the origin
/// let mut transform = Transform::new();
///
/// // Set position and rotate 45 degrees around Y axis
/// transform.set_position(Vec3::new(1.0, 2.0, 3.0));
/// transform.rotate_degrees(0.0, 45.0, 0.0);
///
/// // Convert to matrix for GPU
/// let matrix = transform.to_matrix();
/// ```
#[derive(Debug, Clone)]
pub struct Transform {
    /// Position in 3D space (x, y, z)
    pub position: Vec3,
    /// Rotation as a quaternion (avoids gimbal lock)
    pub rotation: Quat,
    /// Scale factors for each axis (x, y, z)
    pub scale: Vec3,
}

impl Transform {
    /// Creates a new transform with default values.
    ///
    /// Position is set to origin (0, 0, 0), rotation to identity (no rotation),
    /// and scale to (1, 1, 1).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new transform at the specified position.
    ///
    /// # Arguments
    ///
    /// * `position` - The initial position in 3D space
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Converts the transform to a 4x4 transformation matrix.
    ///
    /// This matrix can be used directly in vertex shaders for GPU-based
    /// transformation. The matrix combines scale, rotation, and translation
    /// in the correct order.
    ///
    /// # Returns
    ///
    /// A 4x4 matrix representing the complete transformation
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    // === Position Methods ===

    /// Sets the absolute position.
    ///
    /// # Arguments
    ///
    /// * `position` - New position in 3D space
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Moves the transform by the given offset.
    ///
    /// # Arguments
    ///
    /// * `translation` - Offset to add to current position
    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
    }

    /// Moves the transform by the given offset using individual components.
    ///
    /// # Arguments
    ///
    /// * `x` - Offset along X-axis
    /// * `y` - Offset along Y-axis  
    /// * `z` - Offset along Z-axis
    pub fn translate_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.position += Vec3::new(x, y, z);
    }

    // === Scale Methods ===

    /// Sets the absolute scale.
    ///
    /// # Arguments
    ///
    /// * `scale` - New scale factors for each axis
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    /// Multiplies the current scale by the given factors.
    ///
    /// # Arguments
    ///
    /// * `scale` - Scale factors to multiply by
    pub fn scale(&mut self, scale: Vec3) {
        self.scale *= scale;
    }

    /// Multiplies the current scale by the given factors using individual components.
    ///
    /// # Arguments
    ///
    /// * `sx` - Scale factor for X-axis
    /// * `sy` - Scale factor for Y-axis
    /// * `sz` - Scale factor for Z-axis
    pub fn scale_xyz(&mut self, sx: f32, sy: f32, sz: f32) {
        self.scale *= Vec3::new(sx, sy, sz);
    }

    // === Rotation Methods ===

    /// Sets the rotation using a quaternion.
    ///
    /// This method provides direct control over rotation for advanced users.
    /// For most use cases, consider using [`set_rotation_euler_degrees`] instead.
    ///
    /// # Arguments
    ///
    /// * `rotation` - The new rotation as a quaternion
    ///
    /// [`set_rotation_euler_degrees`]: Transform::set_rotation_euler_degrees
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    /// Sets rotation using Euler angles in radians.
    ///
    /// Rotation order is X, then Y, then Z (XYZ order).
    ///
    /// # Arguments
    ///
    /// * `rx` - Rotation around X-axis in radians
    /// * `ry` - Rotation around Y-axis in radians
    /// * `rz` - Rotation around Z-axis in radians
    pub fn set_rotation_euler_radians(&mut self, rx: f32, ry: f32, rz: f32) {
        self.rotation = Quat::from_euler(glam::EulerRot::XYZ, rx, ry, rz);
    }

    /// Sets rotation using Euler angles in degrees.
    ///
    /// Rotation order is X, then Y, then Z (XYZ order).
    ///
    /// # Arguments
    ///
    /// * `rx` - Rotation around X-axis in degrees
    /// * `ry` - Rotation around Y-axis in degrees  
    /// * `rz` - Rotation around Z-axis in degrees
    ///
    /// # Examples
    ///
    /// ```
    /// # use map::Transform;
    /// let mut transform = Transform::new();
    /// transform.set_rotation_euler_degrees(0.0, 45.0, 0.0); // Turn 45° around Y
    /// ```
    pub fn set_rotation_euler_degrees(&mut self, rx: f32, ry: f32, rz: f32) {
        let radians = [rx.to_radians(), ry.to_radians(), rz.to_radians()];
        self.set_rotation_euler_radians(radians[0], radians[1], radians[2]);
    }

    /// Applies additional rotation using a quaternion.
    ///
    /// This rotation is applied relative to the current rotation.
    ///
    /// # Arguments
    ///
    /// * `rotation` - The additional rotation as a quaternion
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation *= rotation;
    }

    /// Applies additional rotation using Euler angles in radians.
    ///
    /// This rotation is applied relative to the current rotation.
    /// Rotation order is X, then Y, then Z (XYZ order).
    ///
    /// # Arguments
    ///
    /// * `rx` - Additional rotation around X-axis in radians
    /// * `ry` - Additional rotation around Y-axis in radians
    /// * `rz` - Additional rotation around Z-axis in radians
    pub fn rotate_radians(&mut self, rx: f32, ry: f32, rz: f32) {
        self.rotation *= Quat::from_euler(glam::EulerRot::XYZ, rx, ry, rz);
    }

    /// Applies additional rotation using Euler angles in degrees.
    ///
    /// This rotation is applied relative to the current rotation.
    /// Rotation order is X, then Y, then Z (XYZ order).
    ///
    /// # Arguments
    ///
    /// * `rx` - Additional rotation around X-axis in degrees
    /// * `ry` - Additional rotation around Y-axis in degrees
    /// * `rz` - Additional rotation around Z-axis in degrees
    ///
    /// # Examples
    ///
    /// ```
    /// # use map::Transform;
    /// let mut transform = Transform::new();
    /// transform.rotate_degrees(0.0, 10.0, 0.0); // Rotate 10° around Y
    /// transform.rotate_degrees(0.0, 10.0, 0.0); // Now rotated 20° total
    /// ```
    pub fn rotate_degrees(&mut self, rx: f32, ry: f32, rz: f32) {
        let radians = [rx.to_radians(), ry.to_radians(), rz.to_radians()];
        self.rotate_radians(radians[0], radians[1], radians[2]);
    }
}

/// Default implementation for Transform.
///
/// Creates a transform with:
/// - Position at origin (0, 0, 0)
/// - No rotation (identity quaternion)
/// - Scale of (1, 1, 1)
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
