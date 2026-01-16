use bevy::prelude::*;

//plugin stuffs ==
pub struct CodePlugin;

impl Plugin for CodePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, touch)
            //.add_observer()
            ;
    }
}

//components ===========

#[derive(Component,Reflect)]
pub struct d1_position {
    pub pos: f32,
}

#[derive(Component,Reflect)]
pub struct d1_size {
    pub size: f32,
}


//systems ============

fn touch(
    changed: Query<(Entity, Option<&d1_position>, Option<&d1_size>), Changed<d1_position>>,
    all: Query<(Entity, Option<&d1_position>, Option<&d1_size>)>,
) {
    //let me 

    for (entity, d1_position, d1_size) in &all {


    }
}