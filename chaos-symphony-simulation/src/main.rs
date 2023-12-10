#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Simulation

use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    app.run();
}
