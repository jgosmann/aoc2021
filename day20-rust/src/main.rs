use std::{
    collections::HashSet,
    error::Error,
    fmt::Display,
    io::{self, BufRead},
};

#[derive(Clone, Debug)]
struct Image {
    light_pixels: HashSet<(isize, isize)>,
    surround: bool,
}

impl Image {
    fn new(light_pixels: HashSet<(isize, isize)>) -> Self {
        Self {
            light_pixels,
            surround: false,
        }
    }
}

#[derive(Debug)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("parse error")
    }
}

impl Error for ParseError {}

struct Input {
    algorithm: [bool; 512],
    image: Image,
}

impl Input {
    fn read<R: BufRead>(reader: &mut R) -> Result<Self, Box<dyn Error>> {
        let mut buf = String::with_capacity(512);
        reader.read_line(&mut buf)?;
        let algorithm: Vec<bool> = buf
            .chars()
            .filter_map(|c| match c {
                '.' => Some(false),
                '#' => Some(true),
                _ => None,
            })
            .collect();
        buf.clear();
        reader.read_line(&mut buf)?;
        let image = Image::new(
            reader
                .lines()
                .map(|line| line.expect("error reading input"))
                .enumerate()
                .flat_map(|(i, line)| {
                    line.chars()
                        .enumerate()
                        .filter_map(|(j, c)| match c {
                            '#' => Some((j as isize, i as isize)),
                            _ => None,
                        })
                        .collect::<Vec<(isize, isize)>>()
                })
                .collect(),
        );

        Ok(Self {
            algorithm: algorithm.try_into().map_err(|_| ParseError)?,
            image,
        })
    }
}

fn enhance(image: &Image, algorithm: &[bool; 512]) -> Image {
    if image.light_pixels.len() == 0 {
        return image.clone();
    }

    let x_lb = image.light_pixels.iter().map(|c| c.0).min().unwrap();
    let x_ub = image.light_pixels.iter().map(|c| c.0).max().unwrap();
    let y_lb = image.light_pixels.iter().map(|c| c.1).min().unwrap();
    let y_ub = image.light_pixels.iter().map(|c| c.1).max().unwrap();

    let light_pixels = (x_lb - 1..=x_ub + 1)
        .flat_map(|x| {
            (y_lb - 1..=y_ub + 1).filter_map(move |y| {
                let index = (0..9)
                    .map(|i| {
                        let pos = (x + (i % 3) - 1, y + (i / 3) - 1);
                        if image.light_pixels.contains(&pos) {
                            1
                        } else if x_lb <= pos.0 && pos.0 <= x_ub && y_lb <= pos.1 && pos.1 <= y_ub {
                            0
                        } else {
                            if image.surround {
                                1
                            } else {
                                0
                            }
                        }
                    })
                    .fold(0, |accum, item| (accum << 1) | item as usize);
                if algorithm[index] {
                    Some((x, y))
                } else {
                    None
                }
            })
        })
        .collect();

    Image {
        light_pixels,
        surround: if image.surround {
            algorithm[511]
        } else {
            algorithm[0]
        },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let input = Input::read(&mut stdin.lock())?;
    let mut result = enhance(&enhance(&input.image, &input.algorithm), &input.algorithm);
    println!("Enhancing twice: {}", result.light_pixels.len());
    for _ in 0..48 {
        result = enhance(&result, &input.algorithm);
    }
    println!("Enhancing 50 times: {}", result.light_pixels.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT: &[u8] = include_bytes!("../test.input");

    #[test]
    fn test() {
        let mut buf = TEST_INPUT;
        let input = Input::read(&mut buf).unwrap();
        let mut result = enhance(&enhance(&input.image, &input.algorithm), &input.algorithm);
        assert_eq!(result.light_pixels.len(), 35);
        for _ in 0..48 {
            result = enhance(&result, &input.algorithm);
        }
        assert_eq!(result.light_pixels.len(), 3351);
    }
}
