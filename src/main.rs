use std::fmt::{Debug, Formatter};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub const TETRAMINO_SIZE: f32 = 48.0;
pub const FALL_TIMER_TICK: f32 = 1.0;
pub const NEW_MINO_TIMER_TICK: f32 = 5.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_event::<SpawnNewTetraminoEvent>()
        .add_event::<TetraminoDownEvent>()
        .init_resource::<Grid>()
        .init_resource::<MinoAtlas>()
        .init_resource::<FallTetraminosTimer>()
        .init_resource::<MoveMinoTimer>()
        .add_startup_system(setup)
        .add_system(spawn_new_mino)
        .add_system(move_tetramino)
        .add_system(fall_tetraminos.after(move_tetramino))
        .add_system(clear_completed_rows)
        .add_system(spawn_new_tetramino)
        .run();
}

pub struct SpawnNewTetraminoEvent {}

pub struct TetraminoDownEvent {}

#[derive(Resource)]
pub struct FallTetraminosTimer {
    pub timer: Timer,
}

impl Default for FallTetraminosTimer {
    fn default() -> Self {
        FallTetraminosTimer { timer: Timer::from_seconds(FALL_TIMER_TICK, TimerMode::Repeating) }
    }
}

#[derive(Resource)]
pub struct MoveMinoTimer {
    pub timer: Timer,
}

impl Default for MoveMinoTimer {
    fn default() -> Self {
        MoveMinoTimer { timer: Timer::from_seconds(0.05, TimerMode::Repeating) }
    }
}

#[derive(Component)]
pub struct TetraminoFalling {}

#[derive(Component)]
pub struct Tetramino {
    pub col: usize,
    pub row: usize,
    pub delta_coords: [(i32, i32); 4],
    pub width: i32,
    pub height: i32,
    pub sprite_index: usize,
}

#[derive(Component)]
pub struct TetraminoPart {
    pub col: usize,
    pub row: usize,
}

#[derive(Resource)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<Option<Entity>>>,
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for row in (0..self.height).rev() {
            for col in 0..self.width {
                if self.grid[col][row].is_some() {
                    write!(f, " {:03}", self.grid[col][row].unwrap().index())?;
                } else {
                    write!(f, "   .")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            width: 10,
            height: 20,
            grid: vec![vec![None; 20]; 10],
        }
    }
}

#[derive(Resource)]
pub struct MinoAtlas {
    pub atlas_handle: Handle<TextureAtlas>,
}

impl Default for MinoAtlas {
    fn default() -> Self {
        Self {
            atlas_handle: Handle::default(),
        }
    }
}

pub fn spawn_new_tetramino(
    mut tetramino_down_event_reader: EventReader<TetraminoDownEvent>,
    mut spawn_new_mino_writer: EventWriter<SpawnNewTetraminoEvent>,
) {
    if tetramino_down_event_reader.is_empty() {
        return;
    }

    spawn_new_mino_writer.send(SpawnNewTetraminoEvent {});

    tetramino_down_event_reader.clear();
}

pub fn clear_completed_rows(
    mut tetramino_down_event_reader: EventReader<TetraminoDownEvent>,
    mut grid: ResMut<Grid>,
    mut commands: Commands,
    mut mino_part_query: Query<&mut Transform, With<TetraminoPart>>,
) {
    if tetramino_down_event_reader.is_empty() {
        return;
    }

    let mut has_completed_rows = true;
    while has_completed_rows {
        // we search a completed line
        let mut completed_row: Option<usize> = None;
        for row in 0..grid.height {
            completed_row = Some(row);
            for col in 0..grid.width {
                if grid.grid[col][row].is_none() {
                    completed_row = None;
                    break;
                }
            }
            if completed_row.is_some() {
                break;
            }
        }
        if completed_row.is_some() {
            for col in 0..grid.width {
                let tetramino_part_entity = grid.grid[col][completed_row.unwrap()].unwrap();
                commands.entity(tetramino_part_entity).despawn_recursive();
                for row in completed_row.unwrap()..(grid.height - 1) {
                    grid.grid[col][row] = grid.grid[col][row + 1];
                    grid.grid[col][row + 1] = None;
                    if let Some(part_entity) = grid.grid[col][row] {
                        if let Ok(mut part_transform) = mino_part_query.get_mut(part_entity) {
                            part_transform.translation.y -= TETRAMINO_SIZE;
                        }
                    }
                }
            }
        } else {
            has_completed_rows = false;
        }
    }

    tetramino_down_event_reader.clear();
}

pub fn spawn_new_mino(
    mut spawn_new_mino_reader: EventReader<SpawnNewTetraminoEvent>,
    mut commands: Commands,
    mino_atlas: Res<MinoAtlas>,
    mut grid: ResMut<Grid>,
) {
    for _ in spawn_new_mino_reader.iter() {
        match thread_rng().gen_range(0..7) {
            0 => {
                spawn_tetramino_l(grid.width / 2, grid.height - 1, &mut commands, &mut grid, &mino_atlas.atlas_handle);
            }
            1 => {
                spawn_tetramino_i(grid.width / 2, grid.height - 1, &mut commands, &mut grid, &mino_atlas.atlas_handle);
            }
            2 => {
                spawn_tetramino_o(grid.width / 2, grid.height - 1, &mut commands, &mut grid, &mino_atlas.atlas_handle);
            }
            3 => {
                spawn_tetramino_t(grid.width / 2, grid.height - 1, &mut commands, &mut grid, &mino_atlas.atlas_handle);
            }
            4 => {
                spawn_tetramino_j(grid.width / 2, grid.height - 1, &mut commands, &mut grid, &mino_atlas.atlas_handle);
            }
            5 => {
                spawn_tetramino_z(grid.width / 2, grid.height - 1, &mut commands, &mut grid, &mino_atlas.atlas_handle);
            }
            6 => {
                spawn_tetramino_s(grid.width / 2, grid.height - 1, &mut commands, &mut grid, &mino_atlas.atlas_handle);
            }
            _ => {}
        }
    }
}

pub fn fall_tetraminos(
    mut commands: Commands,
    mut tetramino_query: Query<(Entity, &mut Transform, &mut Tetramino), With<TetraminoFalling>>,
    mut fall_tetraminos_timer: ResMut<FallTetraminosTimer>,
    mut grid: ResMut<Grid>,
    mut tetramino_down_writer: EventWriter<TetraminoDownEvent>,
    mino_atlas: Res<MinoAtlas>,
    time: Res<Time>,
) {
    fall_tetraminos_timer.timer.tick(time.delta());
    if !fall_tetraminos_timer.timer.finished() {
        return;
    }

    if let Ok((tetramino_entity, mut tetramino_transform, mut tetramino)) = tetramino_query.get_single_mut() {
        // check if nobody is below
        let can_fall = can_tetramino_fall(&grid, tetramino_entity, &tetramino);
        if !can_fall {
            let tetramino_half_size = TETRAMINO_SIZE / 2.0;

            for (delta_col, delta_row) in tetramino.delta_coords.iter() {
                let col = tetramino.col as i32 + *delta_col;
                let row = tetramino.row as i32 + *delta_row;
                let id = commands.spawn((
                    TetraminoPart { col: col as usize, row: row as usize },
                    SpriteSheetBundle {
                        texture_atlas: mino_atlas.atlas_handle.clone(),
                        transform: Transform::from_xyz(col as f32 * TETRAMINO_SIZE + tetramino_half_size, row as f32 * TETRAMINO_SIZE + tetramino_half_size, 0.0),
                        sprite: TextureAtlasSprite::new(tetramino.sprite_index),
                        ..default()
                    }
                )).id();
                grid.grid[col as usize][row as usize] = Some(id);
            }
            commands.entity(tetramino_entity).despawn_recursive();

            tetramino_down_writer.send(TetraminoDownEvent {});

            return;
        }

        let row_new = tetramino.row - 1;
        tetramino_transform.translation.y -= TETRAMINO_SIZE;
        for (delta_col, delta_row) in tetramino.delta_coords {
            let col = (tetramino.col as i32 + delta_col) as usize;
            let row = (tetramino.row as i32 + delta_row) as usize;
            grid.grid[col][row] = None;
        }
        tetramino.row = row_new;
        for (delta_col, delta_row) in tetramino.delta_coords {
            let col = (tetramino.col as i32 + delta_col) as usize;
            let row = (tetramino.row as i32 + delta_row) as usize;
            grid.grid[col][row] = Some(tetramino_entity);
        }
    }
}

fn can_tetramino_fall(grid: &ResMut<Grid>, tetramino_entity: Entity, tetramino: &Mut<Tetramino>) -> bool {
    let mut can_fall = true;
    for (delta_col, delta_row) in tetramino.delta_coords {
        let col = (tetramino.col as i32 + delta_col) as usize;
        let row = (tetramino.row as i32 + delta_row) as usize;
        if row <= 0 {
            can_fall = false;
            break;
        }
        match grid.grid[col][row - 1] {
            None => continue,
            Some(entity) => {
                if entity != tetramino_entity { // not itself
                    can_fall = false;
                    break;
                }
            }
        }
    }
    can_fall
}

fn can_tetramino_left(grid: &ResMut<Grid>, tetramino_entity: Entity, tetramino: &Mut<Tetramino>) -> bool {
    let mut can_left = true;
    for (delta_col, delta_row) in tetramino.delta_coords {
        let col = tetramino.col as i32 + delta_col;
        let row = tetramino.row as i32 + delta_row;
        if col <= 0 {
            can_left = false;
            break;
        }
        match grid.grid[col as usize - 1][row as usize] {
            None => continue,
            Some(entity) => {
                if entity != tetramino_entity { // not itself
                    can_left = false;
                    break;
                }
            }
        }
    }
    can_left
}

fn can_tetramino_right(grid: &ResMut<Grid>, tetramino_entity: Entity, tetramino: &Mut<Tetramino>) -> bool {
    let mut can_right = true;
    for (delta_col, delta_row) in tetramino.delta_coords {
        let col = tetramino.col as i32 + delta_col;
        let row = tetramino.row as i32 + delta_row;
        if col + 1 >= grid.width as i32 {
            can_right = false;
            break;
        }
        match grid.grid[col as usize + 1][row as usize] {
            None => continue,
            Some(entity) => {
                if entity != tetramino_entity { // not itself
                    can_right = false;
                    break;
                }
            }
        }
    }
    can_right
}

pub fn setup(
    asset_server: Res<AssetServer>,
    grid: Res<Grid>,
    mut commands: Commands,
    mut mino_atlas: ResMut<MinoAtlas>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut spawn_new_mino_writer: EventWriter<SpawnNewTetraminoEvent>,
) {
    let mut window = window_query.get_single_mut().unwrap();
    window.resolution.set(grid.width as f32 * TETRAMINO_SIZE, grid.height as f32 * TETRAMINO_SIZE);
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });

    let texture_handle = asset_server.load("sprites/minos00.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::splat(TETRAMINO_SIZE), 8, 1, Some(Vec2::new(8.0, 8.0)), Some(Vec2::new(4.0, 4.0)));
    mino_atlas.atlas_handle = texture_atlases.add(texture_atlas);

    spawn_new_mino_writer.send(SpawnNewTetraminoEvent {}); // first tetramino
}

pub fn move_tetramino(
    mut move_mino_timer: ResMut<MoveMinoTimer>,
    mut grid: ResMut<Grid>,
    mut tetramino_query: Query<(Entity, &mut Transform, &mut Tetramino), With<TetraminoFalling>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    move_mino_timer.timer.tick(time.delta());
    if !move_mino_timer.timer.finished() {
        return;
    }

    if let Ok((mino_entity, mut mino_transform, mut tetramino)) = tetramino_query.get_single_mut() {
        let mut row_new = tetramino.row as i32;
        let mut col_new = tetramino.col as i32;
        let mut translation_new = mino_transform.translation;
        if keyboard_input.pressed(KeyCode::Down) && can_tetramino_fall(&grid, mino_entity, &tetramino) {
            translation_new.y -= TETRAMINO_SIZE;
            row_new -= 1;
        } else {
            if keyboard_input.pressed(KeyCode::Right) && can_tetramino_right(&grid, mino_entity, &tetramino) {
                translation_new.x += TETRAMINO_SIZE;
                col_new += 1;
            }
            if keyboard_input.pressed(KeyCode::Left) && can_tetramino_left(&grid, mino_entity, &tetramino) {
                translation_new.x -= TETRAMINO_SIZE;
                col_new -= 1;
            }
        }
        if col_new == tetramino.col as i32 && row_new == tetramino.row as i32 {
            // no change
            return;
        }
        for (delta_col, delta_row) in tetramino.delta_coords {
            let col = (tetramino.col as i32 + delta_col) as usize;
            let row = (tetramino.row as i32 + delta_row) as usize;
            grid.grid[col][row] = None;
        }
        tetramino.col = col_new as usize;
        tetramino.row = row_new as usize;
        mino_transform.translation = translation_new;
        for (delta_col, delta_row) in tetramino.delta_coords {
            let col = (tetramino.col as i32 + delta_col) as usize;
            let row = (tetramino.row as i32 + delta_row) as usize;
            grid.grid[col][row] = Some(mino_entity);
        }
    }
}

pub fn spawn_tetramino_l(col: usize, row: usize, commands: &mut Commands, grid: &mut ResMut<Grid>, atlas_handle: &Handle<TextureAtlas>) {
    if col + 1 >= grid.width || row < 2 {
        warn!("spawn_tetramino_l: out of bounds ({}, {})", col, row);
        return;
    }

    let delta_coords: [(i32, i32); 4] = [
        (0, 0),
        (0, -1),
        (0, -2),
        (1, -2),
    ];

    spawn_tetramino(col, row, commands, grid, atlas_handle, 3, delta_coords, 2, 3);
}

pub fn spawn_tetramino_j(col: usize, row: usize, commands: &mut Commands, grid: &mut ResMut<Grid>, atlas_handle: &Handle<TextureAtlas>) {
    if col + 2 >= grid.width || row < 1 {
        warn!("spawn_tetramino_j: out of bounds ({}, {})", col, row);
        return;
    }

    let delta_coords: [(i32, i32); 4] = [
        (0, 0),
        (0, -1),
        (0, -2),
        (1, 0),
    ];

    spawn_tetramino(col, row, commands, grid, atlas_handle, 1, delta_coords, 2, 3);
}

pub fn spawn_tetramino_i(col: usize, row: usize, commands: &mut Commands, grid: &mut ResMut<Grid>, atlas_handle: &Handle<TextureAtlas>) {
    if col >= grid.width || row < 3 {
        warn!("spawn_tetramino_i: out of bounds ({}, {})", col, row);
        return;
    }

    let delta_coords: [(i32, i32); 4] = [
        (0, 0),
        (0, -1),
        (0, -2),
        (0, -3),
    ];

    spawn_tetramino(col, row, commands, grid, atlas_handle, 6, delta_coords, 1, 4);
}

pub fn spawn_tetramino_o(col: usize, row: usize, commands: &mut Commands, grid: &mut ResMut<Grid>, atlas_handle: &Handle<TextureAtlas>) {
    if col + 1 >= grid.width || row < 1 {
        warn!("spawn_tetramino_o: out of bounds ({}, {})", col, row);
        return;
    }

    let delta_coords: [(i32, i32); 4] = [
        (0, 0),
        (0, -1),
        (1, 0),
        (1, -1),
    ];

    spawn_tetramino(col, row, commands, grid, atlas_handle, 7, delta_coords, 2, 2);
}

pub fn spawn_tetramino_t(col: usize, row: usize, commands: &mut Commands, grid: &mut ResMut<Grid>, atlas_handle: &Handle<TextureAtlas>) {
    if col + 2 >= grid.width || row < 1 {
        warn!("spawn_tetramino_t: out of bounds ({}, {})", col, row);
        return;
    }

    let delta_coords: [(i32, i32); 4] = [
        (0, 0),
        (1, 0),
        (2, 0),
        (1, -1),
    ];

    spawn_tetramino(col, row, commands, grid, atlas_handle, 4, delta_coords, 3, 2);
}

pub fn spawn_tetramino_z(col: usize, row: usize, commands: &mut Commands, grid: &mut ResMut<Grid>, atlas_handle: &Handle<TextureAtlas>) {
    if col + 2 >= grid.width || row < 1 {
        warn!("spawn_tetramino_z: out of bounds ({}, {})", col, row);
        return;
    }

    let delta_coords: [(i32, i32); 4] = [
        (0, 0),
        (1, 0),
        (1, -1),
        (2, -1),
    ];

    spawn_tetramino(col, row, commands, grid, atlas_handle, 5, delta_coords, 3, 2);
}

pub fn spawn_tetramino_s(col: usize, row: usize, commands: &mut Commands, grid: &mut ResMut<Grid>, atlas_handle: &Handle<TextureAtlas>) {
    if col + 2 >= grid.width || row < 1 {
        warn!("spawn_tetramino_s: out of bounds ({}, {})", col, row);
        return;
    }

    let delta_coords: [(i32, i32); 4] = [
        (1, 0),
        (2, 0),
        (0, -1),
        (1, -1),
    ];

    spawn_tetramino(col, row, commands, grid, atlas_handle, 2, delta_coords, 3, 2);
}

fn spawn_tetramino(
    col: usize,
    row: usize,
    commands: &mut Commands,
    grid: &mut ResMut<Grid>,
    atlas_handle: &Handle<TextureAtlas>,
    sprite_index: usize,
    delta_coords: [(i32, i32); 4],
    width: i32,
    height: i32,
) {
    let tetramino_half_size = TETRAMINO_SIZE / 2.0;

    let id = commands.spawn(
        (
            Tetramino { col, row, delta_coords, width, height, sprite_index },
            TetraminoFalling {},
            SpatialBundle {
                transform: Transform::from_xyz(col as f32 * TETRAMINO_SIZE + tetramino_half_size, row as f32 * TETRAMINO_SIZE + tetramino_half_size, 0.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for (delta_col, delta_row) in delta_coords.iter() {
                parent.spawn(
                    SpriteSheetBundle {
                        texture_atlas: atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(sprite_index),
                        transform: Transform::from_xyz(*delta_col as f32 * TETRAMINO_SIZE, *delta_row as f32 * TETRAMINO_SIZE, 0.0),
                        ..default()
                    },
                );
            }
        })
        .id();

    for (delta_col, delta_row) in delta_coords.iter() {
        grid.grid[(col as i32 + delta_col) as usize][(row as i32 + delta_row) as usize] = Some(id);
    }
}
