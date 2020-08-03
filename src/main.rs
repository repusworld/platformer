#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate serde_derive;

use std::env;
use std::path;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{KeyCode, KeyMods};
use ggez::graphics::{Color, Mesh, MeshBuilder, Text};
use ggez::*;
use itertools::Itertools;

use common::*;
use components::*;

mod camera;
mod common;
mod components;
mod config;
mod controls;
mod game_state;
mod level;
mod physics;

impl ggez::event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.tick += 1;

            if self.change_level.is_some() {
                let target = self.change_level.clone().unwrap();
                self.change_level = None;
                self.change_level(ctx, target)?;
                break;
            }

            if self.restart_level {
                self.restart_level = false;
                self.restart_level(ctx)?;
                break;
            }

            self.do_movement(ctx)?;
            self.apply_physics(ctx)?;
            self.move_camera(ctx)?;
            self.collision_detection(ctx)?;
            self.reset_pressed_state();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());

        for (_id, (BoundingBox(bbox), mesh, _z_order)) in &mut self
            .world
            .query::<(&BoundingBox, &Mesh, &ZOrder)>()
            .iter()
            .sorted_by_key(|(_id, (_pos, _mesh, z_order))| -z_order.0)
        // sort by z-order, descending
        {
            graphics::draw(
                ctx,
                &*mesh,
                (relative_point(
                    self.camera.center,
                    Point2::new(bbox.x, bbox.y),
                ),),
            )?;
        }

        for (_id, (pos, mesh, _z_order)) in &mut self
            .world
            .query::<(&Position, &Mesh, &ZOrder)>()
            .iter()
            .sorted_by_key(|(_id, (_pos, _mesh, z_order))| -z_order.0)
        // sort by z-order, descending
        {
            graphics::draw(ctx, &*mesh, (relative_point(self.camera.center, pos.0),))?;
        }

        for (_id, (pos, text, col)) in
            &mut self.world.query::<(&Position, &TextContainer, &Color)>()
        {
            graphics::draw(
                ctx,
                &Text::new(
                    graphics::TextFragment::new(text.value.clone())
                        .color(*col)
                        .scale(graphics::Scale::uniform(text.size)),
                ),
                (relative_point(self.camera.center, pos.0),),
            )?;
        }

        if self.config.debug.draw_bounds {
            let mut mb = MeshBuilder::new();
            // bounds with pos
            for (_id, (pos, BoundingBox(bbox))) in
                &mut self.world.query::<(&Position, &BoundingBox)>()
            {
                const BBOX_WIDTH: f32 = 2.0;
                const HALF_BBOX_WIDTH: f32 = BBOX_WIDTH / 2.0;
                mb.line(
                    &[
                        Point2::new(
                            pos.0.x + bbox.x + HALF_BBOX_WIDTH,
                            pos.0.y + bbox.y + HALF_BBOX_WIDTH,
                        ),
                        Point2::new(
                            pos.0.x + bbox.x + bbox.w - HALF_BBOX_WIDTH,
                            pos.0.y + bbox.y + HALF_BBOX_WIDTH,
                        ),
                    ],
                    BBOX_WIDTH,
                    Color::from_rgb(255, 0, 0),
                )?;
                mb.line(
                    &[
                        Point2::new(
                            pos.0.x + bbox.x + HALF_BBOX_WIDTH,
                            pos.0.y + bbox.y + bbox.h - HALF_BBOX_WIDTH,
                        ),
                        Point2::new(
                            pos.0.x + bbox.x + bbox.w - HALF_BBOX_WIDTH,
                            pos.0.y + bbox.y + bbox.h - HALF_BBOX_WIDTH,
                        ),
                    ],
                    BBOX_WIDTH,
                    Color::from_rgb(255, 0, 0),
                )?;
                mb.line(
                    &[
                        Point2::new(
                            pos.0.x + bbox.x + HALF_BBOX_WIDTH,
                            pos.0.y + bbox.y + HALF_BBOX_WIDTH,
                        ),
                        Point2::new(
                            pos.0.x + bbox.x + HALF_BBOX_WIDTH,
                            pos.0.y + bbox.y + bbox.h - HALF_BBOX_WIDTH,
                        ),
                    ],
                    BBOX_WIDTH,
                    Color::from_rgb(255, 0, 0),
                )?;
                mb.line(
                    &[
                        Point2::new(
                            pos.0.x + bbox.x + bbox.w - HALF_BBOX_WIDTH,
                            pos.0.y + bbox.y + HALF_BBOX_WIDTH,
                        ),
                        Point2::new(
                            pos.0.x + bbox.x + bbox.w - HALF_BBOX_WIDTH,
                            pos.0.y + bbox.y + bbox.h - HALF_BBOX_WIDTH,
                        ),
                    ],
                    BBOX_WIDTH,
                    Color::from_rgb(255, 0, 0),
                )?;
            }

            // bounds without pos
            for (_id, BoundingBox(bbox)) in
                &mut self.world.query::<Without<Position, &BoundingBox>>()
            {
                const BBOX_WIDTH: f32 = 2.0;
                const HALF_BBOX_WIDTH: f32 = BBOX_WIDTH / 2.0;
                mb.line(
                    &[
                        Point2::new(bbox.x + HALF_BBOX_WIDTH, bbox.y + HALF_BBOX_WIDTH),
                        Point2::new(bbox.x + bbox.w - HALF_BBOX_WIDTH, bbox.y + HALF_BBOX_WIDTH),
                    ],
                    BBOX_WIDTH,
                    Color::from_rgb(255, 0, 0),
                )?;
                mb.line(
                    &[
                        Point2::new(bbox.x + HALF_BBOX_WIDTH, bbox.y + bbox.h - HALF_BBOX_WIDTH),
                        Point2::new(
                            bbox.x + bbox.w - HALF_BBOX_WIDTH,
                            bbox.y + bbox.h - HALF_BBOX_WIDTH,
                        ),
                    ],
                    BBOX_WIDTH,
                    Color::from_rgb(255, 0, 0),
                )?;
                mb.line(
                    &[
                        Point2::new(bbox.x + HALF_BBOX_WIDTH, bbox.y + HALF_BBOX_WIDTH),
                        Point2::new(bbox.x + HALF_BBOX_WIDTH, bbox.y + bbox.h - HALF_BBOX_WIDTH),
                    ],
                    BBOX_WIDTH,
                    Color::from_rgb(255, 0, 0),
                )?;
                mb.line(
                    &[
                        Point2::new(bbox.x + bbox.w - HALF_BBOX_WIDTH, bbox.y + HALF_BBOX_WIDTH),
                        Point2::new(
                            bbox.x + bbox.w - HALF_BBOX_WIDTH,
                            bbox.y + bbox.h - HALF_BBOX_WIDTH,
                        ),
                    ],
                    BBOX_WIDTH,
                    Color::from_rgb(255, 0, 0),
                )?;
            }

            let mesh = mb.build(ctx)?;
            graphics::draw(
                ctx,
                &mesh,
                (relative_point(self.camera.center, Point2::new(0.0, 0.0)),),
            )?;
        }

        if self.tick % 50 == 0 {
            graphics::set_window_title(ctx, &format!("{:.0} FPS", timer::fps(ctx)));
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) {
        self.map_key_down_event(ctx, keycode, keymods, repeat);
    }

    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods) {
        self.map_key_up_event(ctx, keycode, keymods);
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("platformer", "ggez")
        .add_resource_path(resource_dir)
        .window_mode(WindowMode {
            width: WIDTH,
            height: HEIGHT,
            ..Default::default()
        })
        .window_setup(WindowSetup {
            vsync: false,
            ..Default::default()
        });
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut GameState::new(ctx)?;
    ggez::event::run(ctx, event_loop, state)
}
