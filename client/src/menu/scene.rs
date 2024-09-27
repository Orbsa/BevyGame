use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        }, view::RenderLayers,
    },
};

use super::MenuItem;

#[derive(Component, Debug)]
pub enum MenuButton {
    Connect,
    ConnectLocal,
    Quit,
}

#[derive(Component)]
pub struct SelectedButton;

impl MenuButton {
    fn spawn(
        self,
        transform: Transform,
        commands: &mut Commands,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(Cuboid {
                    half_size: Vec3::splat(0.5),
                })),
                material: materials.add(Color::hex("#3090b0").unwrap()),
                transform,
                ..default()
            },
            self,
            MenuItem,
        ));
    }
}

pub fn spawn_menu_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    // let size = 30.;
    // Ground
    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Plane {
    //             size: size * 2.0,
    //             subdivisions: 10,
    //         })),
    //         material: materials.add(Color::hex("#1f7840").unwrap().into()),
    //         transform: Transform::from_xyz(0.0, -0.01, 0.0),
    //         ..default()
    //     },
    //     Name::new("Plane"),
    //     RigidBody::Static,
    //     //MenuItem, // TODO MenuItem
    // ));
    //.with_children(|commands| {
    //commands.spawn((
    //Collider::cuboid(size, 1., size),
    //TransformBundle::from(Transform::from_xyz(0., -1., 0.)),
    //));
    //});
    // Sun
    commands.spawn((
        DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(
                -std::f32::consts::FRAC_PI_2,
            ))
            .mul_transform(Transform::from_rotation(Quat::from_rotation_y(
                -std::f32::consts::FRAC_PI_4,
            ))),
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 0.9, 0.8),
                illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                shadows_enabled: true,
                
                ..default()
            },
            ..Default::default()
        },
        RenderLayers::all() ,
        Name::new("Sun"),
        //MenuItem, // TODO MenuItem
    ));
    // House
    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("sprytilebrickhouse.gltf#Scene0"),
                transform: Transform::from_xyz(-5., -0.9, -20.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                ..default()
            },
            Name::new("House"),
            //MenuItem,
        ))
        .with_children(|commands| {
            commands.spawn((
                SpatialBundle::from_transform(Transform::from_xyz(-5., 0., -5.)),
                //Collider::cuboid(5., 1.0, 6.),
            ));
        });

    MenuButton::Quit.spawn(
        Transform::from_xyz(0.0, 1.5, -3.0),
        &mut commands,
        &mut materials,
        &mut meshes,
    );
    MenuButton::ConnectLocal.spawn(
        Transform::from_xyz(3.0, 1.0, 0.0),
        &mut commands,
        &mut materials,
        &mut meshes,
    );
    MenuButton::Connect.spawn(
        Transform::from_xyz(0.0, 1.0, 3.0),
        &mut commands,
        &mut materials,
        &mut meshes,
    );

    commands.spawn((
        TextBundle::from_section(
            "GameState: Menu",
            TextStyle {
                font: asset_server.load("fonts/ttf/JetBrainsMono-Regular.ttf"),
                font_size: 14.0,
                color: Color::WHITE,
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        }),
        MenuItem,
    ));
}

fn _test_sub_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    commands.spawn((
        TextBundle::from_section(
            "GameState: MASDFASDF",
            TextStyle {
                font: asset_server.load("fonts/fonts/ttf/JetBrainsMono-Regular.ttf"),
                font_size: 14.0,
                color: Color::WHITE,
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            ..default()
        }),
        MenuItem,
    ));

    let cam_size = Extent3d {
        width: 1000,
        height: 1000,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: cam_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(cam_size);

    let image_handle = images.add(image);

    let camera = Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        ..default()
    };

    commands.spawn((camera, MenuItem));

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    //let cube_size = 1.0;
    let cube_handle = meshes.add(Mesh::from(Plane3d {
        normal: Direction3d::Y,
    }));

    // Main pass cube, with material containing the rendered first pass texture.
    commands.spawn((
        PbrBundle {
            mesh: cube_handle,
            material: material_handle,
            transform: Transform::from_xyz(-3.0, 1.0, 0.0)
                .with_rotation(Quat::from_rotation_x(PI / 2.0))
                .with_rotation(Quat::from_rotation_y(PI / 2.0)),
            ..default()
        },
        MenuItem,
    ));
}
