#![forbid(unsafe_code)]

#[path = "../examples/factory_shortcuts_acceptance/matrix.rs"]
mod accepted_maps;
#[path = "pair_presentation/config.rs"]
mod config;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "pair_presentation/qualification.rs"]
mod qualification;
#[path = "pair_presentation/rendering.rs"]
mod rendering;
#[path = "support/surface.rs"]
mod surface;
mod support {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
}
