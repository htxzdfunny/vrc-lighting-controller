pub fn is_strobe_visible(strobe_on: bool, strobe_speed: f64, time_secs: f64) -> bool {
    if !strobe_on || strobe_speed <= 0.0 {
        return true;
    }
    let phase = (time_secs * strobe_speed * 2.0 * std::f64::consts::PI).sin();
    phase > 0.0
}
