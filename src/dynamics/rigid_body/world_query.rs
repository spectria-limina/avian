#![allow(missing_docs)]

use crate::{prelude::*, utils::get_pos_translation};
use bevy::{
    ecs::query::QueryData,
    prelude::{Entity, Has, Ref},
};
use std::ops::{AddAssign, SubAssign};

/// A `WorldQuery` to make querying and modifying rigid bodies more convenient.
#[derive(QueryData)]
#[query_data(mutable)]
pub struct RigidBodyQuery {
    pub entity: Entity,
    pub rb: Ref<'static, RigidBody>,
    pub position: &'static mut Position,
    pub rotation: &'static mut Rotation,
    pub previous_rotation: &'static mut PreviousRotation,
    pub accumulated_translation: &'static mut AccumulatedTranslation,
    pub linear_velocity: &'static mut LinearVelocity,
    pub(crate) pre_solve_linear_velocity: &'static mut PreSolveLinearVelocity,
    pub angular_velocity: &'static mut AngularVelocity,
    pub(crate) pre_solve_angular_velocity: &'static mut PreSolveAngularVelocity,
    pub mass: &'static mut Mass,
    pub angular_inertia: &'static mut AngularInertia,
    pub center_of_mass: &'static mut CenterOfMass,
    pub friction: &'static Friction,
    pub restitution: &'static Restitution,
    pub locked_axes: Option<&'static LockedAxes>,
    pub dominance: Option<&'static Dominance>,
    pub time_sleeping: &'static mut TimeSleeping,
    pub is_sleeping: Has<Sleeping>,
    pub is_sensor: Has<Sensor>,
}

impl<'w> RigidBodyQueryItem<'w> {
    /// Computes the velocity at the given `point` relative to the center of the body.
    pub fn velocity_at_point(&self, point: Vector) -> Vector {
        #[cfg(feature = "2d")]
        {
            self.linear_velocity.0 + self.angular_velocity.0 * point.perp()
        }
        #[cfg(feature = "3d")]
        {
            self.linear_velocity.0 + self.angular_velocity.cross(point)
        }
    }

    /// Computes the effective inverse mass, taking into account any translation locking.
    pub fn effective_inv_mass(&self) -> Vector {
        if !self.rb.is_dynamic() {
            return Vector::ZERO;
        }

        let mut inv_mass = Vector::splat(self.mass.inverse);

        if let Some(locked_axes) = self.locked_axes {
            inv_mass = locked_axes.apply_to_vec(inv_mass);
        }

        inv_mass
    }

    /// Computes the effective world-space inverse inertia, taking into account any rotation locking.
    #[cfg(feature = "2d")]
    pub fn effective_world_inv_inertia(&self) -> Scalar {
        if !self.rb.is_dynamic() {
            return 0.0;
        }

        let mut inv_inertia = self.angular_inertia.inverse;

        if let Some(locked_axes) = self.locked_axes {
            inv_inertia = locked_axes.apply_to_rotation(inv_inertia);
        }

        inv_inertia
    }

    /// Computes the effective world-space inverse inertia tensor, taking into account any rotation locking.
    #[cfg(feature = "3d")]
    pub fn effective_world_inv_inertia(&self) -> Matrix3 {
        if !self.rb.is_dynamic() {
            return Matrix3::ZERO;
        }

        let mut inv_inertia = self.angular_inertia.rotated_inverse(self.rotation.0);

        if let Some(locked_axes) = self.locked_axes {
            inv_inertia = locked_axes.apply_to_rotation(inv_inertia);
        }

        inv_inertia
    }

    /// Returns the current position of the body. This is a sum of the [`Position`] and
    /// [`AccumulatedTranslation`] components.
    pub fn current_position(&self) -> Vector {
        self.position.0
            + get_pos_translation(
                &self.accumulated_translation,
                &self.previous_rotation,
                &self.rotation,
                &self.center_of_mass,
            )
    }

    /// Returns the [dominance](Dominance) of the body.
    ///
    /// If it isn't specified, the default of `0` is returned for dynamic bodies.
    /// For static and kinematic bodies, `i8::MAX` (`127`) is always returned instead.
    pub fn dominance(&self) -> i8 {
        if !self.rb.is_dynamic() {
            i8::MAX
        } else {
            self.dominance.map_or(0, |dominance| dominance.0)
        }
    }
}

impl<'w> RigidBodyQueryReadOnlyItem<'w> {
    /// Computes the velocity at the given `point` relative to the center of mass.
    pub fn velocity_at_point(&self, point: Vector) -> Vector {
        #[cfg(feature = "2d")]
        {
            self.linear_velocity.0 + self.angular_velocity.0 * point.perp()
        }
        #[cfg(feature = "3d")]
        {
            self.linear_velocity.0 + self.angular_velocity.cross(point)
        }
    }

    /// Returns the mass. If the rigid body is not dynamic, the returned mass is infinite.
    pub fn mass(&self) -> Mass {
        if self.rb.is_dynamic() {
            *self.mass
        } else {
            Mass::INFINITY
        }
    }

    /// Computes the effective inverse mass, taking into account any translation locking.
    pub fn effective_inverse_mass(&self) -> Vector {
        if !self.rb.is_dynamic() {
            return Vector::ZERO;
        }

        let mut inv_mass = Vector::splat(self.mass.inverse);

        if let Some(locked_axes) = self.locked_axes {
            inv_mass = locked_axes.apply_to_vec(inv_mass);
        }

        inv_mass
    }

    /// Returns the local angular inertia. If the rigid body is not dynamic, the returned angular inertia is infinite.
    pub fn angular_inertia(&self) -> AngularInertia {
        if self.rb.is_dynamic() {
            *self.angular_inertia
        } else {
            AngularInertia::INFINITY
        }
    }

    /// Computes the effective world-space angular inertia, taking into account any rotation locking.
    pub fn effective_world_angular_intertia(&self) -> AngularInertia {
        if !self.rb.is_dynamic() {
            return AngularInertia::INFINITY;
        }

        #[cfg(feature = "2d")]
        let mut inv_inertia = self.angular_inertia.inverse;
        #[cfg(feature = "3d")]
        let mut inv_inertia = self.angular_inertia.rotated_inverse(self.rotation.0);

        if let Some(locked_axes) = self.locked_axes {
            inv_inertia = locked_axes.apply_to_rotation(inv_inertia);
        }

        AngularInertia::from_inverse(inv_inertia)
    }

    /// Returns the current position of the body. This is a sum of the [`Position`] and
    /// [`AccumulatedTranslation`] components.
    pub fn current_position(&self) -> Vector {
        self.position.0
            + get_pos_translation(
                self.accumulated_translation,
                self.previous_rotation,
                self.rotation,
                self.center_of_mass,
            )
    }

    /// Returns the [dominance](Dominance) of the body.
    ///
    /// If it isn't specified, the default of `0` is returned for dynamic bodies.
    /// For static and kinematic bodies, `i8::MAX` (`127`) is always returned instead.
    pub fn dominance(&self) -> i8 {
        if !self.rb.is_dynamic() {
            i8::MAX
        } else {
            self.dominance.map_or(0, |dominance| dominance.0)
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct MassPropertiesQuery {
    pub mass: &'static mut Mass,
    pub angular_inertia: &'static mut AngularInertia,
    pub center_of_mass: &'static mut CenterOfMass,
}

impl<'w> AddAssign<ColliderMassProperties> for MassPropertiesQueryItem<'w> {
    fn add_assign(&mut self, rhs: ColliderMassProperties) {
        let mass1 = self.mass.value();
        let mass2 = rhs.mass.value();
        let new_mass = mass1 + mass2;

        if new_mass <= 0.0 {
            return;
        }

        let com1 = self.center_of_mass.0;
        let com2 = rhs.center_of_mass.0;

        // Compute the combined center of mass and combined inertia tensor
        let new_com = (com1 * mass1 + com2 * mass2) / new_mass;
        let i1 = self.angular_inertia.shifted(mass1, new_com - com1);
        let i2 = rhs.angular_inertia.shifted(mass2, new_com - com2);
        let new_inertia = i1 + i2;

        // Update mass properties
        self.mass.set(new_mass);
        self.angular_inertia.set(new_inertia);
        self.center_of_mass.0 = new_com;
    }
}

impl<'w> SubAssign<ColliderMassProperties> for MassPropertiesQueryItem<'w> {
    fn sub_assign(&mut self, rhs: ColliderMassProperties) {
        let mass1 = self.mass.value();
        let mass2 = rhs.mass.value();

        if mass1 + mass2 <= 0.0 {
            return;
        }

        let new_mass = (mass1 - mass2).max(0.0);
        let com1 = self.center_of_mass.0;
        let com2 = rhs.center_of_mass.0;

        // Compute the combined center of mass and combined inertia tensor
        let new_com = if new_mass > Scalar::EPSILON {
            (com1 * mass1 - com2 * mass2) / new_mass
        } else {
            com1
        };
        let i1 = self.angular_inertia.shifted(mass1, new_com - com1);
        let i2 = rhs.angular_inertia.shifted(mass2, new_com - com2);
        let new_inertia = i1 - i2;

        // Update mass properties
        self.mass.set(new_mass);
        self.angular_inertia.set(new_inertia);
        self.center_of_mass.0 = new_com;
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use approx::assert_relative_eq;
    use bevy::prelude::*;

    // TODO: Test if inertia values are correct
    #[test]
    fn mass_properties_add_assign_works() {
        // Create app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn an entity with mass properties
        app.world_mut().spawn(MassPropertiesBundle {
            mass: Mass::new(1.6),
            center_of_mass: CenterOfMass(Vector::NEG_X * 3.8),
            ..default()
        });

        // Create collider mass properties that will be added to the existing mass properties
        let collider_mass_props = ColliderMassProperties {
            mass: Mass::new(8.1),
            center_of_mass: CenterOfMass(Vector::X * 1.2 + Vector::Y),
            ..default()
        };

        // Get the mass properties and add the collider mass properties
        let mut query = app.world_mut().query::<MassPropertiesQuery>();
        let mut mass_props = query.single_mut(app.world_mut());
        mass_props += collider_mass_props;

        // Test if values are correct
        // (reference values were calculated by hand)
        assert_relative_eq!(mass_props.mass.value(), 9.7);
        assert_relative_eq!(mass_props.mass.inverse, 1.0 / 9.7);
        assert_relative_eq!(
            mass_props.center_of_mass.0,
            Vector::X * 0.375_257 + Vector::Y * 0.835_051,
            epsilon = 0.000_001
        );
    }

    // TODO: Test if inertia values are correct
    #[test]
    fn mass_properties_sub_assign_works() {
        // Create app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn an entity with mass properties
        app.world_mut().spawn(MassPropertiesBundle {
            mass: Mass::new(8.1),
            center_of_mass: CenterOfMass(Vector::NEG_X * 3.8),
            ..default()
        });

        // Create collider mass properties that will be subtracted from the existing mass properties
        let collider_mass_props = ColliderMassProperties {
            mass: Mass::new(1.6),
            center_of_mass: CenterOfMass(Vector::X * 1.2 + Vector::Y),
            ..default()
        };

        // Get the mass properties and subtract the collider mass properties
        let mut query = app.world_mut().query::<MassPropertiesQuery>();
        let mut mass_props = query.single_mut(app.world_mut());
        mass_props -= collider_mass_props;

        // Test if values are correct.
        // The reference values were calculated by hand.
        // The center of mass is computed as: (com1 * mass1 - com2 * mass2) / (mass1 - mass2).max(0.0)
        assert_relative_eq!(mass_props.mass.value(), 6.5);
        assert_relative_eq!(mass_props.mass.inverse, 1.0 / 6.5);
        assert_relative_eq!(
            mass_props.center_of_mass.0,
            Vector::NEG_X * 5.030_769 + Vector::NEG_Y * 0.246_153,
            epsilon = 0.000_001
        );
    }

    #[test]
    #[cfg(all(
        feature = "default-collider",
        any(feature = "parry-f32", feature = "parry-f64")
    ))]
    fn mass_properties_add_sub_works() {
        // Create app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let original_mass_props =
            MassPropertiesBundle::new_computed(&Collider::capsule(0.6, 2.4), 3.9);

        // Spawn an entity with mass properties
        app.world_mut().spawn(original_mass_props.clone());

        // Create collider mass properties
        let collider_mass_props = Collider::capsule(2.1, 7.4).mass_properties(14.3);

        // Get the mass properties and then add and subtract the collider mass properties
        let mut query = app.world_mut().query::<MassPropertiesQuery>();
        let mut mass_props = query.single_mut(app.world_mut());
        mass_props += collider_mass_props;
        mass_props -= collider_mass_props;

        // Test if values are correct. They should be equal to the original values.
        // Some epsilons reduced to make test pass on apple-m1
        // see: https://github.com/Jondolf/avian/issues/137
        assert_relative_eq!(
            mass_props.mass.value(),
            original_mass_props.mass.value(),
            epsilon = 0.001
        );
        assert_relative_eq!(
            mass_props.mass.inverse,
            original_mass_props.mass.inverse,
            epsilon = 0.000_001
        );
        assert_relative_eq!(
            mass_props.angular_inertia.value(),
            original_mass_props.angular_inertia.value(),
            epsilon = 0.001
        );
        assert_relative_eq!(
            mass_props.angular_inertia.inverse,
            original_mass_props.angular_inertia.inverse,
            epsilon = 0.001
        );
        assert_relative_eq!(
            mass_props.center_of_mass.0,
            original_mass_props.center_of_mass.0,
            epsilon = 0.000_001
        );
    }
}
