use std::time::Duration;

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::PingEvent;

/// Network Keep Alive Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkKeepAlivePlugin;

impl Plugin for NetworkKeepAlivePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeepAliveTimer::new())
            .add_systems(Update, keep_alive);
    }
}

/// Keep Alive Timer.
#[derive(Resource)]
struct KeepAliveTimer {
    inner: Timer,
}

impl KeepAliveTimer {
    /// Creates a new [`KeepAliveTimer`].
    fn new() -> Self {
        Self {
            inner: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }
}

/// Keeps connection alive by periodically sending pings.
#[allow(clippy::needless_pass_by_value)]
fn keep_alive(
    time: Res<Time>,
    mut timer: ResMut<KeepAliveTimer>,
    query: Query<(Entity, &NetworkEndpoint)>,
) {
    if timer.inner.tick(time.delta()).just_finished() {
        query.for_each(|(entity, endpoint)| {
            let message = PingEvent::new(Uuid::new_v4());
            if message.try_send(endpoint).is_err() {
                let span = warn_span!(
                    "keep_alive",
                    entity =? entity,
                    id = endpoint.id(),
                    remote_address =% endpoint.remote_address()
                );
                let _guard = span.enter();

                warn!("unable to send ping");
            };
        });
    }
}
