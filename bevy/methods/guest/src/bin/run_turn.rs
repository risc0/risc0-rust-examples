// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]


use bevy_core::Outputs;
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::guest::{env, sha};
use core::num::NonZeroU32;
use getrandom::Error;

use getrandom::register_custom_getrandom;
// Some application-specific error code
const MY_CUSTOM_ERROR_CODE: u32 = Error::CUSTOM_START + 42;
pub fn always_fail(buf: &mut [u8]) -> Result<(), Error> {
    let code = NonZeroU32::new(MY_CUSTOM_ERROR_CODE).unwrap();
    Err(Error::from(code))
}
register_custom_getrandom!(always_fail);

use bevy_ecs::world::World;
use bevy_ecs::prelude::*;

risc0_zkvm::guest::entry!(main);

#[derive(Component)]
struct Position { x: f32, y: f32 }
#[derive(Component)]
struct Velocity { x: f32, y: f32 }
#[derive(StageLabel)]
pub struct UpdateLabel;


// This system moves each entity with a Position and Velocity component
fn movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

pub fn main() {
    let mut world = World::new();
    let entity = world
        .spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }))
        .id();

    let mut schedule = Schedule::default();

    schedule.add_stage(UpdateLabel, SystemStage::single_threaded()
        .with_system(movement)
    );
    {
        let entity_ref = world.entity(entity);
        let position = entity_ref.get::<Position>().unwrap();
        assert!(position.x == 0.0);
    }

    // Run ten timesteps
    for _i in 0..3 {
        schedule.run(&mut world);
    }
    {
        let entity_ref = world.entity(entity);
        let position = entity_ref.get::<Position>().unwrap();
        assert!(position.x == 3.0); // moved 3 units to the right!
        // dummy hash until state is transiting
        // let out = Outputs {
        //     position: position.x, 
        // };
        // env::commit(&out);
    }
}