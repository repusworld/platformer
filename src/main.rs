#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate serde_derive;

use std::env;
use std::path;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{KeyCode, KeyMods};
use ggez::graphics::{Color, DrawMode, Mesh, MeshBuilder, StrokeOptions};
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
mod physics;

impl ggez::event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.tick += 1;

            self.do_movement(ctx);
            self.apply_physics(ctx);
            self.move_camera(ctx);

            self.reset_pressed_state();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());

        for (_id, (pos, mesh, _z_order)) in &mut self
            .world
            .query::<(&Position, &Mesh, &ZOrder)>()
            .iter()
            .sorted_by_key(|(_id, (_pos, _mesh, z_order))| -z_order.0)
        // sort by z-order, descending
        {
            graphics::draw(ctx, &*mesh, (relative_point(self.camera.center, pos.0),))?;
        }

        for (_id, (pos, BoundingBox(bbox))) in &mut self.world.query::<(&Position, &BoundingBox)>()
        {
            let mut mb = MeshBuilder::new();
            mb.line(
                &[
                    Point2::new(bbox.x, bbox.y),
                    Point2::new(bbox.x + bbox.w, bbox.y),
                ],
                1.0,
                Color::from_rgb(255, 0, 0),
            )?;
            mb.line(
                &[
                    Point2::new(bbox.x, bbox.y - bbox.h),
                    Point2::new(bbox.x + bbox.w, bbox.y - bbox.h),
                ],
                1.0,
                Color::from_rgb(255, 0, 0),
            )?;
            mb.line(
                &[
                    Point2::new(bbox.x, bbox.y),
                    Point2::new(bbox.x, bbox.y - bbox.h),
                ],
                1.0,
                Color::from_rgb(255, 0, 0),
            )?;
            mb.line(
                &[
                    Point2::new(bbox.x + bbox.w, bbox.y),
                    Point2::new(bbox.x + bbox.w, bbox.y - bbox.h),
                ],
                1.0,
                Color::from_rgb(255, 0, 0),
            )?;
            let mesh = mb.build(ctx)?;
            graphics::draw(ctx, &mesh, (relative_point(self.camera.center, pos.0),))?;
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
