#![warn(clippy::nursery)]
#![warn(clippy::perf)]
use crate::conway::*;
use anyhow::Result;
use coffee::graphics::WindowSettings;
use coffee::Game;

mod conway;

fn main() -> Result<()> {
	Conway::run(WindowSettings {
		title: "Conway's game of life!".into(),
		size: (WINDOW_SIZE_X as u32, WINDOW_SIZE_Y as u32),
		resizable: false,
		fullscreen: false,
		maximized: false,
	})?;
	Ok(())
}
