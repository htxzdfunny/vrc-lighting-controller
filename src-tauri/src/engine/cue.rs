use crate::state::fixture::{Fixture, Color};

pub fn interpolate_fixtures(
    from: &[Fixture],
    to: &[Fixture],
    t: f64,
) -> Vec<Fixture> {
    let len = from.len().max(to.len());
    (0..len)
        .map(|i| {
            let f = from.get(i);
            let t_fix = to.get(i);
            match (f, t_fix) {
                (Some(a), Some(b)) => interpolate_fixture(a, b, t),
                (Some(a), None) => a.clone(),
                (None, Some(b)) => b.clone(),
                (None, None) => Fixture::new(i),
            }
        })
        .collect()
}

fn interpolate_fixture(a: &Fixture, b: &Fixture, t: f64) -> Fixture {
    Fixture {
        id: a.id,
        name: a.name.clone(),
        pan: lerp(a.pan, b.pan, t),
        tilt: lerp(a.tilt, b.tilt, t),
        color: Color {
            r: lerp(a.color.r, b.color.r, t),
            g: lerp(a.color.g, b.color.g, t),
            b: lerp(a.color.b, b.color.b, t),
        },
        dimmer: lerp(a.dimmer, b.dimmer, t),
        strobe_on: if t < 0.5 { a.strobe_on } else { b.strobe_on },
        strobe_speed: lerp(a.strobe_speed, b.strobe_speed, t),
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}
