use orfail::{Failure, OrFail};
use pagurus::image::Sprite;
use pagurus::spatial::{Position, Region, Size};
use pagurus::Result;

const PNG_ITEMS: &[u8] = include_bytes!("../assets/items.png");
const PNG_BUTTONS: &[u8] = include_bytes!("../assets/buttons.png");
const PNG_CURSORS: &[u8] = include_bytes!("../assets/cursors.png");
const PNG_BACKGROUND: &[u8] = include_bytes!("../assets/background.png");
const PNG_CHARS_SMALL: &[u8] = include_bytes!("../assets/chars-small.png");
const PNG_CHARS_LARGE: &[u8] = include_bytes!("../assets/chars-large.png");

fn decode_sprite(png: &[u8]) -> Result<Sprite> {
    let decoder = png::Decoder::new(png);
    let mut reader = decoder.read_info().or_fail()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).or_fail()?;
    let bytes = &buf[..info.buffer_size()];
    let size = Size::from_wh(info.width, info.height);
    (info.bit_depth == png::BitDepth::Eight)
        .or_fail_with(|()| format!("unsupported PNG bit depth: {:?}", info.bit_depth))?;

    match info.color_type {
        png::ColorType::Rgb => Sprite::from_rgb24_bytes(bytes, size).or_fail(),
        png::ColorType::Rgba => Sprite::from_rgba32_bytes(bytes, size).or_fail(),
        png::ColorType::Grayscale => Sprite::from_grayscale8_bytes(bytes, size).or_fail(),
        png::ColorType::GrayscaleAlpha => {
            Sprite::from_grayscale_alpha16_bytes(bytes, size).or_fail()
        }
        _ => Err(Failure::new(format!(
            "unsupported PNG color type: {:?}",
            info.color_type
        ))),
    }
}

#[derive(Debug)]
pub struct Assets {
    pub sprites: Sprites,
}

impl Assets {
    pub fn load() -> Result<Self> {
        Ok(Self {
            sprites: Sprites::load().or_fail()?,
        })
    }
}

#[derive(Debug)]
pub struct Sprites {
    pub background: Sprite,
    pub items: Items,
    pub buttons: Buttons,
    pub cursor: Cursor,
    pub numbers: Numbers,
    pub strings: Strings,
}

impl Sprites {
    fn load() -> Result<Self> {
        let numbers = Numbers::load().or_fail()?;
        Ok(Self {
            background: decode_sprite(PNG_BACKGROUND).or_fail()?,
            items: Items::load().or_fail()?,
            buttons: Buttons::load().or_fail()?,
            cursor: Cursor::load().or_fail()?,
            strings: Strings::load(numbers.small[0].original()).or_fail()?,
            numbers,
        })
    }
}

#[derive(Debug)]
pub struct Items {
    pub snake_head: Sprite,
    pub snake_tail: Sprite,
    pub apple: Sprite,
}

impl Items {
    fn load() -> Result<Self> {
        let sprite = decode_sprite(PNG_ITEMS).or_fail()?;
        let region = Region::new(Position::ORIGIN, Size::square(32));
        Ok(Self {
            snake_head: sprite.clip(region).or_fail()?,
            snake_tail: sprite.clip(region.shift_x(1)).or_fail()?,
            apple: sprite.clip(region.shift_x(2)).or_fail()?,
        })
    }
}

#[derive(Debug)]
pub struct Buttons {
    pub play: Button,
    pub exit: Button,
    pub retry: Button,
    pub title: Button,
}

impl Buttons {
    fn load() -> Result<Self> {
        let sprite = decode_sprite(PNG_BUTTONS).or_fail()?;
        let origin = Region::new(Position::ORIGIN, Button::SIZE);
        Ok(Self {
            play: Button::load(&sprite, origin.position).or_fail()?,
            exit: Button::load(&sprite, origin.shift_y(1).position).or_fail()?,
            retry: Button::load(&sprite, origin.shift_y(2).position).or_fail()?,
            title: Button::load(&sprite, origin.shift_y(3).position).or_fail()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Button {
    pub normal: Sprite,
    pub focused: Sprite,
    pub pressed: Sprite,
}

impl Button {
    pub const SIZE: Size = Size::from_wh(32 * 5, 33);

    fn load(sprite: &Sprite, offset: Position) -> Result<Self> {
        let region = Region::new(offset, Self::SIZE);
        Ok(Self {
            pressed: sprite.clip(region).or_fail()?,
            normal: sprite.clip(region.shift_x(1)).or_fail()?,
            focused: sprite.clip(region.shift_x(2)).or_fail()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub normal: Sprite,
    pub pressing: Sprite,
    pub select_up: Sprite,
    pub select_down: Sprite,
    pub select_right: Sprite,
    pub select_left: Sprite,
}

impl Cursor {
    fn load() -> Result<Self> {
        let sprite = decode_sprite(PNG_CURSORS).or_fail()?;
        let region = Region::new(Position::ORIGIN, Size::square(32));
        Ok(Self {
            normal: sprite.clip(region).or_fail()?,
            pressing: sprite.clip(region.shift_x(1)).or_fail()?,
            select_up: sprite.clip(region.shift_x(2)).or_fail()?,
            select_down: sprite.clip(region.shift_x(3)).or_fail()?,
            select_right: sprite.clip(region.shift_x(4)).or_fail()?,
            select_left: sprite.clip(region.shift_x(5)).or_fail()?,
        })
    }
}

#[derive(Debug)]
pub struct Numbers {
    pub small: Vec<Sprite>,
    pub large: Vec<Sprite>,
}

impl Numbers {
    fn load() -> Result<Self> {
        let sprite = decode_sprite(PNG_CHARS_SMALL).or_fail()?;
        let small_region = Region::new(Position::from_xy(0, 16), Size::from_wh(10, 16));
        let large_region = Region::new(Position::from_xy(0, 32), Size::square(16));
        Ok(Self {
            small: (0..10)
                .map(|i| sprite.clip(small_region.shift_x(i)).or_fail())
                .collect::<Result<_>>()?,
            large: (0..10)
                .map(|i| {
                    sprite
                        .clip(large_region.shift_x(i % 5).shift_y(i / 5))
                        .or_fail()
                })
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Debug)]
pub struct Strings {
    pub snake: Sprite,
    pub game: Sprite,
    pub over: Sprite,
    pub high_score: Sprite,
}

impl Strings {
    fn load(small_sprite: Sprite) -> Result<Self> {
        let large_sprite = decode_sprite(PNG_CHARS_LARGE).or_fail()?;
        let large_region = Region::new(Position::ORIGIN, Size::from_wh(256, 64));
        Ok(Self {
            snake: large_sprite.clip(large_region).or_fail()?,
            game: large_sprite.clip(large_region.shift_y(1)).or_fail()?,
            over: large_sprite.clip(large_region.shift_y(2)).or_fail()?,
            high_score: small_sprite
                .clip(Region::new(Position::ORIGIN, Size::from_wh(112, 16)))
                .or_fail()?,
        })
    }
}
