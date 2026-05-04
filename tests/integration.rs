#![cfg(feature = "keyboard")]

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum Action {
    PayRespects,
}

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum PolicyAction {
    Confirm,
}

// A resource that represents whether respects have been paid or not
#[derive(Resource, Default, PartialEq, Debug)]
struct Respect(bool);

fn pay_respects(
    player_action_state: Query<&ActionState<Action>, With<Player>>,
    global_action_state: Query<&ActionState<Action>, Without<Player>>,
    mut respect: ResMut<Respect>,
) {
    if let Ok(action_state) = player_action_state.single() {
        if action_state.pressed(&Action::PayRespects) {
            respect.0 = true;
        }
    }
    // Previously a global resource `ActionState`; now a second, non-player entity.
    if let Ok(action_state) = global_action_state.single() {
        if action_state.pressed(&Action::PayRespects) {
            respect.0 = true;
        }
    }
}

/// Spawns the "global" (non-player) input entity that previously lived as a resource.
fn spawn_global_input(mut commands: Commands) {
    commands.spawn(InputMap::new([(Action::PayRespects, KeyCode::KeyF)]));
}

fn respect_fades(mut respect: ResMut<Respect>) {
    respect.0 = false;
}

fn remove_input_map(mut commands: Commands, query: Query<Entity, With<InputMap<Action>>>) {
    for entity in query.iter() {
        commands.entity(entity).remove::<InputMap<Action>>();
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn(InputMap::new([(Action::PayRespects, KeyCode::KeyF)]))
        .insert(Player);
}

fn spawn_policy_player(mut commands: Commands) {
    commands
        .spawn(InputMap::new([(PolicyAction::Confirm, KeyCode::Space)]))
        .insert(Player);
}

fn spawn_empty_policy_player(mut commands: Commands) {
    commands.spawn(Player);
}

#[test]
fn disable_input() {
    use bevy::input::InputPlugin;

    let mut app = App::new();

    // Here we spawn a player and a separate global input entity to check if [`DisableInput`]
    // releases correctly on both.
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, (spawn_player, spawn_global_input))
        .init_resource::<Respect>()
        .add_systems(Update, pay_respects)
        .add_systems(PreUpdate, respect_fades);

    // Press F to pay respects
    KeyCode::KeyF.press(app.world_mut());
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Disable the global input
    let mut action_state = app
        .world_mut()
        .query_filtered::<&mut ActionState<Action>, Without<Player>>()
        .single_mut(app.world_mut())
        .expect("global ActionState not found");
    action_state.disable_all_actions();

    // But the player is still paying respects
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Disable the player's input too
    let mut action_state = app
        .world_mut()
        .query_filtered::<&mut ActionState<Action>, With<Player>>()
        .single_mut(app.world_mut())
        .expect("ActionState not found");
    action_state.disable_all_actions();

    // Now, all respect has faded
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // And even pressing F cannot bring it back
    KeyCode::KeyF.press(app.world_mut());
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // Re-enable the global input
    let mut action_state = app
        .world_mut()
        .query_filtered::<&mut ActionState<Action>, Without<Player>>()
        .single_mut(app.world_mut())
        .expect("global ActionState not found");
    action_state.enable_all_actions();

    // And it will start paying respects again
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(true));
}

#[test]
fn release_when_input_map_removed() {
    use bevy::input::InputPlugin;

    let mut app = App::new();

    // Spawn a player carrying its own input map.
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        .init_resource::<Respect>()
        .add_systems(Update, pay_respects)
        .add_systems(PreUpdate, respect_fades);

    // Press F to pay respects
    KeyCode::KeyF.press(app.world_mut());
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(true));

    // Remove the InputMap component from the player
    app.add_systems(Update, remove_input_map);
    // Needs an extra frame for the removed-component detection to release inputs
    app.update();

    // Now, all respect has faded
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(false));

    // And even pressing F cannot bring it back
    KeyCode::KeyF.press(app.world_mut());
    app.update();
    let respect = app.world().resource::<Respect>();
    assert_eq!(*respect, Respect(false));
}

#[test]
fn plugin_default_input_context_policy_is_applied_to_resources_and_components() {
    use bevy::input::InputPlugin;

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(
            InputManagerPlugin::<PolicyAction>::default()
                .with_input_context_policy(InputContextPolicy::IgnoreStaleReleases),
        )
        .add_systems(Startup, spawn_policy_player);

    app.update();

    app.world_mut().spawn(InputMap::<PolicyAction>::new([(
        PolicyAction::Confirm,
        KeyCode::Space,
    )]));


    let action_state = app
        .world_mut()
        .query_filtered::<&ActionState<PolicyAction>, With<Player>>()
        .single(app.world())
        .expect("ActionState not found");
    assert_eq!(
        action_state.input_context_policy(),
        InputContextPolicy::IgnoreStaleReleases
    );
}

#[test]
fn reinserting_input_map_marks_action_state_fresh_for_non_legacy_policies() {
    use bevy::input::InputPlugin;

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(
            InputManagerPlugin::<PolicyAction>::default()
                .with_input_context_policy(InputContextPolicy::IgnoreStaleReleases),
        )
        .add_systems(Startup, spawn_empty_policy_player);

    app.update();

    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .single(app.world())
        .expect("Player not found");

    KeyCode::Space.press(app.world_mut());
    app.update();

    app.world_mut()
        .entity_mut(player)
        .insert(InputMap::new([(PolicyAction::Confirm, KeyCode::Space)]));
    app.update();

    let action_state = app
        .world_mut()
        .query_filtered::<&ActionState<PolicyAction>, With<Player>>()
        .single(app.world())
        .expect("ActionState not found");
    assert!(action_state.pressed(&PolicyAction::Confirm));
    assert!(!action_state.pressed_since_fresh(&PolicyAction::Confirm));

    KeyCode::Space.release(app.world_mut());
    app.update();

    let action_state = app
        .world_mut()
        .query_filtered::<&ActionState<PolicyAction>, With<Player>>()
        .single(app.world())
        .expect("ActionState not found");
    assert!(!action_state.just_released(&PolicyAction::Confirm));
}

#[cfg(feature = "timing")]
#[test]
fn duration() {
    use bevy::input::InputPlugin;
    use core::time::Duration;

    const RESPECTFUL_DURATION: Duration = Duration::from_millis(5);

    fn hold_f_to_pay_respects(
        action_state: Single<&ActionState<Action>>,
        mut respect: ResMut<Respect>,
    ) {
        if action_state.pressed(&Action::PayRespects)
            // Unrealistically disrespectful, but makes the tests faster
            && action_state.current_duration(&Action::PayRespects) > RESPECTFUL_DURATION
        {
            respect.0 = true;
        }
    }

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        .init_resource::<Respect>()
        .add_systems(Update, hold_f_to_pay_respects);

    // Initializing
    app.update();

    // Press
    KeyCode::KeyF.press(app.world_mut());

    // Hold
    std::thread::sleep(2 * RESPECTFUL_DURATION);

    // Check
    app.update();
    let action_state = app
        .world_mut()
        .query::<&ActionState<Action>>()
        .single(app.world())
        .unwrap();
    assert!(action_state.pressed(&Action::PayRespects));
}
