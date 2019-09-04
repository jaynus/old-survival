#![cfg(test)]
extern crate survival;

use amethyst::{ecs::Builder, GameData, SimpleState, SimpleTrans, StateData, Trans};
use amethyst_test::AmethystApplication;

use survival::components::TimeAvailable;
use survival::systems::{time::TimeState, TimeSystem};
#[derive(Default)]
struct TestState {
    iters: u32,
}
impl SimpleState for TestState {
    fn on_start(&mut self, _: StateData<'_, GameData<'_, '_>>) {
        //let StateData { world, .. } = data;
    }
    fn update(&mut self, _: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        self.iters += 1;

        if self.iters > 10 {
            return Trans::Quit;
        }

        Trans::None
    }
}

#[test]
fn time_system() -> amethyst::Result<()> {
    use slog::Drain;

    // Make sure to save the guard, see documentation for more information
    let decorator = slog_term::TermDecorator::new().force_color().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let async_drain = slog_async::Async::new(drain).build().fuse();
    let root_log = slog::Logger::root(async_drain, slog::slog_o!());

    assert!(AmethystApplication::blank()
        .with_setup(move |world| {
            world.create_entity().with(TimeAvailable::default()).build();
        })
        .with_resource(survival::settings::Context {
            spritesheet: None,
            logs: survival::settings::Logs { root: root_log }
        })
        .with_resource(TimeState::default())
        .with_system(TimeSystem, "time_system", &[])
        // WTF? .with_state(|| TestState::default())
        .run()
        .is_ok());

    Ok(())
}
