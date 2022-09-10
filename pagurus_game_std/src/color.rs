#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Rgb(Rgb),
    Rgba(Rgba),
}

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb(Rgb::new(r, g, b))
    }

    pub fn alpha(self, a: u8) -> Self {
        match self {
            Color::Rgb(x) => Self::Rgba(x.alpha(a)),
            Color::Rgba(mut x) => {
                x.a = a;
                Self::Rgba(x)
            }
        }
    }

    pub fn to_rgba(self) -> Rgba {
        match self {
            Color::Rgb(x) => x.alpha(255),
            Color::Rgba(x) => x,
        }
    }
}

impl From<Rgb> for Color {
    fn from(x: Rgb) -> Self {
        Self::Rgb(x)
    }
}

impl From<Rgba> for Color {
    fn from(x: Rgba) -> Self {
        Self::Rgba(x)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn alpha_blend(self, dst: Self) -> Self {
        fn blend(s: u32, d: u32, a: u32) -> u32 {
            s + d - (d * a / (0xFF * 0xFF))
        }

        let a = blend(
            u32::from(self.a) * 0xFF,
            u32::from(dst.a) * 0xFF,
            u32::from(self.a) * 0xFF,
        );
        let r = blend(
            u32::from(self.r) * u32::from(self.a),
            u32::from(dst.r) * u32::from(dst.a),
            u32::from(self.a) * 0xFF,
        );
        let g = blend(
            u32::from(self.g) * u32::from(self.a),
            u32::from(dst.g) * u32::from(dst.a),
            u32::from(self.a) * 0xFF,
        );
        let b = blend(
            u32::from(self.b) * u32::from(self.a),
            u32::from(dst.b) * u32::from(dst.a),
            u32::from(self.a) * 0xFF,
        );
        Self {
            r: (r * 0xFF * 0xFF / a / 0xFF) as u8,
            g: (g * 0xFF * 0xFF / a / 0xFF) as u8,
            b: (b * 0xFF * 0xFF / a / 0xFF) as u8,
            a: (a / 0xFF) as u8,
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn alpha_blend_works() {
        let black = Rgba::new(0, 0, 0, 255);
        assert_eq!(black, black.alpha_blend(black));
    }
}
