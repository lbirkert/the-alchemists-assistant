use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(GamePostUpdate, set_liquid_material);
    app.add_systems(GamePostUpdate, update_liquid_material);
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[type_path = "the_alchemist_assistant::liquid"]
pub struct LiquidController {
    pub color: Color,
    pub is_visible: bool,
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[component(on_remove)]
#[type_path = "the_alchemist_assistant::liquid"]
pub struct Liquid {
    material_handle: Option<Handle<StandardMaterial>>,
}

impl Liquid {
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let Some(cauldron_liquid) = world.get::<Liquid>(context.entity).cloned() else {
            return;
        };

        let mut commands = world.commands();

        if let Some(material) = cauldron_liquid.material_handle {
            commands.queue(move |world: &mut World| {
                world
                    .resource_mut::<Assets<StandardMaterial>>()
                    .remove(material.id());
            });
        }
    }
}

fn set_liquid_material(
    mut commands: Commands,
    mut liquids: Query<(
        Entity,
        &mut Liquid,
        Option<&MeshMaterial3d<StandardMaterial>>,
    )>,
    liquid_controllers: Query<&LiquidController>,
    hierarchy: Query<(Option<&ChildOf>, Has<LiquidController>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut potion_liquid, existing_material) in &mut liquids {
        if potion_liquid.material_handle.is_some() {
            continue;
        }

        let Some(controller) = crate::utils::find_parent::<LiquidController>(entity, &hierarchy)
        else {
            continue;
        };

        let Ok(controller) = liquid_controllers.get(controller) else {
            continue;
        };

        if !controller.is_visible {
            commands.queue(move |world: &mut World| {
                if let Ok(mut entity) = world.get_entity_mut(entity) {
                    entity.insert(Visibility::Hidden);
                }
            });
        }
        let mut material = existing_material
            .and_then(|handle| materials.get(&handle.0))
            .cloned()
            .unwrap_or_default();
        material.base_color = controller.color;
        material.emissive = controller.color.to_linear();
        let material_handle = materials.add(material);

        potion_liquid.material_handle = Some(material_handle.clone());

        commands
            .entity(entity)
            .insert(MeshMaterial3d(material_handle));
    }
}

fn update_liquid_material(
    mut commands: Commands,
    mut liquids: Query<(Entity, &Liquid)>,
    liquid_controllers: Query<&LiquidController, Changed<LiquidController>>,
    hierarchy: Query<(Option<&ChildOf>, Has<LiquidController>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, potion_liquid) in &mut liquids {
        let Some(ref material_handle) = potion_liquid.material_handle else {
            continue;
        };

        let Some(controller) = crate::utils::find_parent::<LiquidController>(entity, &hierarchy)
        else {
            continue;
        };

        let Ok(controller) = liquid_controllers.get(controller) else {
            continue;
        };

        let is_visible = controller.is_visible;
        commands.queue(move |world: &mut World| {
            if let Ok(mut entity) = world.get_entity_mut(entity) {
                entity.insert(match is_visible {
                    true => Visibility::Inherited,
                    false => Visibility::Hidden,
                });
            }
        });

        if let Some(mut material) = materials.get_mut(material_handle.id()) {
            material.base_color = controller.color;
            material.emissive = controller.color.to_linear();
        }
    }
}
