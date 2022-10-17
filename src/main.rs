#![feature(unboxed_closures, fn_traits)]

use bevy::{math::vec3, prelude::*, sprite::Anchor, time::FixedTimestep};

mod game;

const UNIT_PX: f32 = 20.;
const BORDER_SIZE: f32 = 2.;
const WINDOW_HEIGHT: f32 = 440.;
const WINDOW_WIDTH: f32 = 320.;
const MARGIN_SIZE: f32 = 20.;

// colors
const BG_COLOR: Color = Color::BLACK;
const BORDER_COLOR: Color = Color::WHITE;

const TICK_DURATION: f32 = 1. / 60.;

#[derive(Default)]
struct RawInput {
    move_left: bool,
    move_right: bool,
    rotate_cw: bool,
    rotate_ccw: bool,
    fast_drop: bool,
    instant_drop: bool,
}

impl RawInput {
    fn reset(&mut self) {
        *self = RawInput::default();
    }
}

impl game::Input for RawInput {
    fn move_left(&self) -> bool {
        self.move_left
    }

    fn move_right(&self) -> bool {
        self.move_right
    }

    fn rotate_cw(&self) -> bool {
        self.rotate_cw
    }

    fn rotate_ccw(&self) -> bool {
        self.rotate_ccw
    }

    fn fast_drop(&self) -> bool {
        self.fast_drop
    }

    fn instant_drop(&self) -> bool {
        self.instant_drop
    }
}

struct UI {
    board: Entity,
}

#[derive(Component)]
struct PointComponent(game::Id);

#[derive(Component)]
struct BlockComponent(game::Id);

fn main() {
    App::new()
        .insert_resource(ClearColor(BG_COLOR))
        .insert_resource(WindowDescriptor {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            resizable: false,
            ..default()
        })
        .init_resource::<RawInput>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PreUpdate, check_input)
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TICK_DURATION as f64))
                .with_system(tick),
        )
        .add_system(update_block_points)
        .add_system(update_board_points)
        .add_system(bevy::window::close_on_esc)
        .run()
}

fn units_to_px(units: usize) -> f32 {
    units as f32 * UNIT_PX
}

fn pos_to_vec3(pos: game::Position) -> Vec3 {
    vec3(units_to_px(pos.0), units_to_px(pos.1), 0.)
}

fn setup(mut commands: Commands) {
    let ui = setup_ui(&mut commands);
    let game = setup_game(&mut commands, &ui);

    commands.insert_resource(ui);
    commands.insert_resource(game);
}

fn setup_ui(commands: &mut Commands) -> UI {
    commands.spawn_bundle(Camera2dBundle::default());

    // move (0, 0) to top / left and flip y axis
    let canvas = commands
        .spawn_bundle(SpatialBundle::from_transform(Transform {
            translation: vec3(-WINDOW_WIDTH / 2., WINDOW_HEIGHT / 2., 0.),
            scale: vec3(1., -1., 1.),
            ..default()
        }))
        .id();

    // board
    let board_width = units_to_px(game::BOARD_WIDTH);
    let board_height = units_to_px(game::VISIBLE_BOARD_HEIGHT);
    let board_with_border_width = board_width + BORDER_SIZE * 2.;
    let board_with_border_height = board_height + BORDER_SIZE * 2.;

    // board border
    let board_border = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: BORDER_COLOR,
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform {
                translation: vec3(MARGIN_SIZE, MARGIN_SIZE, 0.),
                scale: vec3(board_with_border_width, board_with_border_height, 1.),
                ..default()
            },
            ..default()
        })
        .id();

    // board bg
    let board_bg = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: BG_COLOR,
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform {
                translation: vec3(MARGIN_SIZE + BORDER_SIZE, MARGIN_SIZE + BORDER_SIZE, 0.),
                scale: vec3(board_width, board_height, 1.),
                ..default()
            },
            ..default()
        })
        .id();

    let board = commands
        .spawn_bundle(SpatialBundle::from_transform(Transform::from_xyz(
            MARGIN_SIZE + BORDER_SIZE,
            MARGIN_SIZE + BORDER_SIZE,
            0.,
        )))
        .id();

    commands
        .entity(canvas)
        .push_children(&[board_border, board_bg, board]);

    UI { board }
}

fn setup_game(commands: &mut Commands, ui: &UI) -> game::Game {
    let game = game::Game::new();
    spawn_block(
        commands,
        game.active_block(),
        game.active_block_position(),
        ui.board,
    );
    game
}

fn spawn_block(
    commands: &mut Commands,
    block: &game::Block,
    block_pos: game::Position,
    parent: Entity,
) {
    for point in block.points() {
        let point_pos = block.get_point_position(point.id).unwrap();
        let point_pos = game::add_positions(block_pos, point_pos);
        let point_entity = spawn_point(commands, point, point_pos, parent);
        commands
            .entity(point_entity)
            .insert(BlockComponent(block.id));
    }
}

fn spawn_point(
    commands: &mut Commands,
    point: &game::Point,
    point_pos: game::Position,
    parent: Entity,
) -> Entity {
    let point_entity = commands
        .spawn()
        .insert(PointComponent(point.id))
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: game::get_block_color(point.origin_block_type),
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform {
                translation: pos_to_vec3(point_pos),
                scale: vec3(UNIT_PX, UNIT_PX, 1.),
                ..default()
            },
            ..default()
        })
        .id();

    commands.entity(parent).add_child(point_entity);
    point_entity
}

fn check_input(bevy_input: Res<Input<KeyCode>>, mut input: ResMut<RawInput>) {
    if bevy_input.pressed(KeyCode::Left) {
        input.move_left = true;
    }
    if bevy_input.pressed(KeyCode::Right) {
        input.move_right = true;
    }
    if bevy_input.pressed(KeyCode::Down) {
        input.fast_drop = true;
    }
}

fn tick(
    mut commands: Commands,
    mut game: ResMut<game::Game>,
    ui: Res<UI>,
    mut input: ResMut<RawInput>,
    block_points: Query<Entity, With<BlockComponent>>,
) {
    let input = input.as_mut();
    let changes = game.tick(input);
    input.reset();

    for change in changes {
        use crate::game::TickChange::*;
        match change {
            BlockLocked => {
                for point_entity in block_points.iter() {
                    commands.entity(point_entity).remove::<BlockComponent>();
                }
            }
            NewBlock => {
                spawn_block(
                    &mut commands,
                    game.active_block(),
                    game.active_block_position(),
                    ui.board,
                );
            }
        }
    }
}

fn update_board_points(
    game: Res<game::Game>,
    mut board_points: Query<
        (&PointComponent, &mut Transform, &mut Visibility),
        Without<BlockComponent>,
    >,
) {
    for (point, mut transform, mut visibility) in board_points.iter_mut() {
        let point_pos = game.get_point_position(point.0).unwrap();
        update_point_view(point_pos, &mut transform, &mut visibility);
    }
}

fn update_block_points(
    game: Res<game::Game>,
    mut board_points: Query<
        (&PointComponent, &mut Transform, &mut Visibility),
        With<BlockComponent>,
    >,
) {
    let block = game.active_block();
    let block_pos = game.active_block_position();
    for (point, mut transform, mut visibility) in board_points.iter_mut() {
        let point_pos = block.get_point_position(point.0).unwrap();
        let point_pos = game::add_positions(block_pos, point_pos);
        update_point_view(point_pos, &mut transform, &mut visibility);
    }
}

fn update_point_view(
    point_pos: game::Position,
    transform: &mut Transform,
    visibility: &mut Visibility,
) {
    if point_pos.1 >= game::HIDDEN_BOARD_TOP {
        transform.translation = pos_to_vec3((point_pos.0, point_pos.1 - game::HIDDEN_BOARD_TOP));
        visibility.is_visible = true;
    } else {
        visibility.is_visible = false;
    }
}
