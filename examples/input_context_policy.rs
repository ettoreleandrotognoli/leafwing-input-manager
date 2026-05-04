//! Demonstrates how [`InputContextPolicy`] can prevent stale releases when switching contexts.
//!
//! Reproduction steps:
//! 1. Start in gameplay.
//! 2. Hold `Space` so the gameplay input map treats it as `Jump`.
//! 3. While still holding `Space`, press `F2` to switch to the menu input map.
//! 4. Release `Space`.
//!
//! Expected result:
//! - The legacy menu action state will emit a stale `Confirm` release.
//! - The protected menu action state will ignore that stale release.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<GameplayAction>::default())
        .add_plugins(InputManagerPlugin::<LegacyMenuAction>::default())
        .add_plugins(
            InputManagerPlugin::<ProtectedMenuAction>::default()
                .with_input_context_policy(InputContextPolicy::RequireFreshPressCycle),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                switch_context,
                log_gameplay_actions,
                log_legacy_menu_actions,
                log_protected_menu_actions,
            ),
        )
        .run();
}

#[derive(Resource)]
struct DemoEntities {
    gameplay: Entity,
    legacy_menu: Entity,
    protected_menu: Entity,
}

#[derive(Actionlike, Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
enum GameplayAction {
    Jump,
}

#[derive(Actionlike, Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
enum LegacyMenuAction {
    Confirm,
}

#[derive(Actionlike, Clone, Copy, Debug, Eq, Hash, PartialEq, Reflect)]
enum ProtectedMenuAction {
    Confirm,
}

#[derive(Component)]
struct GameplayReceiver;

#[derive(Component)]
struct LegacyMenuReceiver;

#[derive(Component)]
struct ProtectedMenuReceiver;

fn gameplay_input_map() -> InputMap<GameplayAction> {
    InputMap::new([(GameplayAction::Jump, KeyCode::Space)])
}

fn legacy_menu_input_map() -> InputMap<LegacyMenuAction> {
    InputMap::new([(LegacyMenuAction::Confirm, KeyCode::Space)])
}

fn protected_menu_input_map() -> InputMap<ProtectedMenuAction> {
    InputMap::new([(ProtectedMenuAction::Confirm, KeyCode::Space)])
}

fn setup(mut commands: Commands) {
    let gameplay = commands
        .spawn((
            Name::new("Gameplay Input"),
            GameplayReceiver,
            gameplay_input_map(),
        ))
        .id();

    let legacy_menu = commands
        .spawn((Name::new("Legacy Menu Input"), LegacyMenuReceiver))
        .id();

    let protected_menu = commands
        .spawn((Name::new("Protected Menu Input"), ProtectedMenuReceiver))
        .id();

    commands.insert_resource(DemoEntities {
        gameplay,
        legacy_menu,
        protected_menu,
    });

    info!("Input context policy demo");
    info!("Press F1 for gameplay, F2 for menu.");
    info!("Hold Space in gameplay, press F2 while still holding it, then release Space.");
    info!("The legacy menu input will print a stale Confirm release.");
    info!("The protected menu input will ignore that stale release.");
}

fn switch_context(
    keys: Res<ButtonInput<KeyCode>>,
    demo_entities: Res<DemoEntities>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::F1) {
        commands
            .entity(demo_entities.gameplay)
            .insert(gameplay_input_map());
        commands
            .entity(demo_entities.legacy_menu)
            .remove::<InputMap<LegacyMenuAction>>();
        commands
            .entity(demo_entities.protected_menu)
            .remove::<InputMap<ProtectedMenuAction>>();

        info!("Switched to gameplay. Space is bound to Jump.");
    }

    if keys.just_pressed(KeyCode::F2) {
        commands
            .entity(demo_entities.gameplay)
            .remove::<InputMap<GameplayAction>>();
        commands
            .entity(demo_entities.legacy_menu)
            .insert(legacy_menu_input_map());
        commands
            .entity(demo_entities.protected_menu)
            .insert(protected_menu_input_map());

        info!("Switched to menu. Space is now bound to Confirm.");
        info!(
            "Because the protected menu plugin uses RequireFreshPressCycle, reinserting its InputMap automatically marks it fresh."
        );
        info!("Releasing Space now should only trigger the legacy menu release log.");
    }
}

fn log_gameplay_actions(
    action_state: Single<&ActionState<GameplayAction>, With<GameplayReceiver>>,
) {
    if action_state.just_pressed(&GameplayAction::Jump) {
        info!("Gameplay: Jump just_pressed");
    }

    if action_state.just_released(&GameplayAction::Jump) {
        info!("Gameplay: Jump just_released");
    }
}

fn log_legacy_menu_actions(
    action_state: Single<&ActionState<LegacyMenuAction>, With<LegacyMenuReceiver>>,
) {
    if action_state.just_released(&LegacyMenuAction::Confirm) {
        warn!("Legacy menu: stale Confirm just_released fired");
    }
}

fn log_protected_menu_actions(
    action_state: Single<&ActionState<ProtectedMenuAction>, With<ProtectedMenuReceiver>>,
) {
    if action_state.just_released(&ProtectedMenuAction::Confirm) {
        info!("Protected menu: Confirm just_released fired");
    }
}
