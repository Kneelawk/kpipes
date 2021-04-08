use cgmath::Vector3;

pub trait FromHSB {
    fn from_hsb(hue: f32, saturation: f32, brightness: f32) -> Self;
}

impl FromHSB for Vector3<f32> {
    fn from_hsb(hue: f32, saturation: f32, brightness: f32) -> Self {
        if saturation == 0f32 {
            Vector3 {
                x: brightness,
                y: brightness,
                z: brightness
            }
        } else {
            let sector = (hue - hue.floor()) * 6f32;
            let offset_in_sector = sector - sector.floor();
            let off = brightness * (1f32 - saturation);
            let fade_out = brightness * (1f32 - saturation * offset_in_sector);
            let fade_in = brightness * (1f32 - saturation * (1f32 - offset_in_sector));
            match sector as u32 {
                0 => Vector3 {
                    x: brightness,
                    y: fade_in,
                    z: off,
                },
                1 => Vector3 {
                    x: fade_out,
                    y: brightness,
                    z: off,
                },
                2 => Vector3 {
                    x: off,
                    y: brightness,
                    z: fade_in,
                },
                3 => Vector3 {
                    x: off,
                    y: fade_out,
                    z: brightness,
                },
                4 => Vector3 {
                    x: fade_in,
                    y: off,
                    z: brightness,
                },
                5 => Vector3 {
                    x: brightness,
                    y: off,
                    z: fade_out,
                },
                _ => unreachable!("Invalid color wheel sector from hue {}", hue),
            }
        }
    }
}
