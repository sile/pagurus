#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(packed)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const RED: Self = Self::new(255, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn alpha(self, a: u8) -> Rgba {
        Rgba {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(packed)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_rgb(self) -> Rgb {
        Rgb::new(self.r, self.g, self.b)
    }

    pub fn to_alpha_blend_rgb(self, dst: Rgb) -> Rgb {
        fn blend(s: u8, d: u8, a: u8) -> u8 {
            let v = u16::from(s) * u16::from(a) + u16::from(d) * (255 - u16::from(a));
            (v / 255) as u8
        }

        Rgb {
            r: blend(self.r, dst.r, self.a),
            g: blend(self.g, dst.g, self.a),
            b: blend(self.b, dst.b, self.a),
        }
    }
}
