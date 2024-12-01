use crate::util::*;
use sdl2::image::LoadTexture;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use std::collections::HashMap;
use std::path::Path;

pub struct Images<'a> {
    black: HashMap<PieceState, Texture<'a>>,
    white: HashMap<PieceState, Texture<'a>>,
    background: Texture<'a>,
}

impl<'a> Images<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        use PieceState::*;
        let mut black = HashMap::new();
        let mut white = HashMap::new();
        let pieces = [King, Queen, Rook, Bishop, Knight, Pawn];
        let colours = ["white", "black"];

        for piece in pieces {
            for colour in colours {
                let path = Path::new("pieces-basic-png").join(format!(
                    "{}-{}.png",
                    colour,
                    format!("{:?}", piece).to_lowercase()
                ));

                if colour == "black" {
                    black.insert(piece, texture_creator.load_texture(&path)?);
                } else {
                    white.insert(piece, texture_creator.load_texture(&path)?);
                }
            }
        }

        let background = texture_creator.load_texture(Path::new("brown.png"))?;
        Ok(Self {
            black,
            white,
            background,
        })
    }

    pub fn get(&self, piece: Piece) -> &Texture<'a> {
        if piece.color == Color::Black {
            self.black.get(&piece.state).expect(
                "Should never invariant of class is hashMap has values for every possible key",
            )
        } else {
            self.white.get(&piece.state).expect(
                "Should never invariant of class is hashMap has values for every possible key",
            )
        }
    }

    pub fn get_background(&self) -> &Texture<'a> {
        &self.background
    }
}
