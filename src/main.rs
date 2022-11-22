mod args; // declare as module
use args::Args;
use image::{
  imageops::FilterType::Triangle, io::Reader, DynamicImage, GenericImageView, ImageError,
  ImageFormat,
};
use std::convert::TryInto;

#[derive(Debug)]
enum ImageDataErrors {
  DifferentImageFormats,
  BufferTooSmall,
  UnableToReadImageFromPath(std::io::Error),
  UnableToFormatImage(String),
  UnableToDecodeImage(ImageError),
  UnableToSaveImage(ImageError),
}

// holds metadata of image
struct FloatingImage {
  width: u32,
  height: u32,
  data: Vec<u8>, // pixel values 0-255
  name: String,
}

impl FloatingImage {
  fn new(width: u32, height: u32, name: String) -> Self {
    // reserve space for data
    // let buffer_capacity = 3655744;
    let buffer_capacity = height * width * 4; // we use rgba values
    let buffer = Vec::with_capacity(buffer_capacity.try_into().unwrap());

    FloatingImage {
      width,
      height,
      data: buffer,
      name,
    }
  }

  fn set_data(&mut self, data: Vec<u8>) -> Result<(), ImageDataErrors> {
    if data.len() > self.data.capacity() {
      return Err(ImageDataErrors::BufferTooSmall);
    }

    self.data = data;
    Ok(())
  }
}

fn main() -> Result<(), ImageDataErrors> {
  let args: Args = Args::new();
  let (image_1, image_format_1): (DynamicImage, ImageFormat) = find_image_from_path(args.image_1)?;
  let (image_2, image_format_2): (DynamicImage, ImageFormat) = find_image_from_path(args.image_2)?;

  if image_format_1 != image_format_2 {
    return Err(ImageDataErrors::DifferentImageFormats);
  }

  let (image_1, image_2): (DynamicImage, DynamicImage) = standardize_size(image_1, image_2);
  let mut output: FloatingImage =
    FloatingImage::new(image_1.width(), image_1.height(), args.output);

  let combined_data: Vec<u8> = combine_images(image_1, image_2);
  output.set_data(combined_data)?;

  if let Err(e) = image::save_buffer_with_format(
    output.name,
    &output.data,
    output.width,
    output.height,
    image::ColorType::Rgba8,
    image_format_1,
  ) {
    Err(ImageDataErrors::UnableToSaveImage(e))
  } else {
    Ok(())
  }
}

fn find_image_from_path(path: String) -> Result<(DynamicImage, ImageFormat), ImageDataErrors> {
  match Reader::open(&path) {
    Ok(image_reader) => {
      if let Some(image_format) = image_reader.format() {
        match image_reader.decode() {
          Ok(image) => Ok((image, image_format)),
          Err(e) => Err(ImageDataErrors::UnableToDecodeImage(e)),
        }
      } else {
        return Err(ImageDataErrors::UnableToFormatImage(path));
      }
    }
    Err(e) => Err(ImageDataErrors::UnableToReadImageFromPath(e)),
  }
}

fn get_smallest_dimensions(dim_1: (u32, u32), dim_2: (u32, u32)) -> (u32, u32) {
  // compare number of pixels per image
  let pix_1 = dim_1.0 * dim_1.1;
  let pix_2 = dim_2.0 * dim_2.1;

  return match pix_1 < pix_2 {
    true => dim_1,
    false => dim_2,
  };
}

fn standardize_size(image_1: DynamicImage, image_2: DynamicImage) -> (DynamicImage, DynamicImage) {
  let (width, height) = get_smallest_dimensions(image_1.dimensions(), image_2.dimensions());
  println!("width: {}, height: {}\n", width, height);

  // image 2 is smaller; resize image 1
  if image_2.dimensions() == (width, height) {
    (image_1.resize_exact(width, height, Triangle), image_2)
  } else {
    (image_1, image_2.resize_exact(width, height, Triangle))
  }
}

fn combine_images(image_1: DynamicImage, image_2: DynamicImage) -> Vec<u8> {
  let vec_1: Vec<u8> = image_1.to_rgba8().into_vec();
  let vec_2: Vec<u8> = image_2.to_rgba8().into_vec();

  alternate_pixels(vec_1, vec_2)
}

fn alternate_pixels(vec_1: Vec<u8>, vec_2: Vec<u8>) -> Vec<u8> {
  // if vec1.len == n, -> [00, 01, 02... 0n]
  let mut combined_data = vec![0u8; vec_1.len()];

  let mut i = 0;
  while i < vec_1.len() {
    if i % 8 == 0 {
      combined_data.splice(i..=i + 3, set_rgba(&vec_1, i, i + 3));
    } else {
      combined_data.splice(i..=i + 3, set_rgba(&vec_2, i, i + 3));
    }
    i += 4; // we use rgba
  }

  return combined_data;
}

fn set_rgba(vec: &Vec<u8>, start: usize, end: usize) -> Vec<u8> {
  let mut rgba: Vec<u8> = Vec::new();
  for i in start..=end {
    let val: u8 = match vec.get(i) {
      Some(d) => *d,
      None => panic!("index out of bounds"),
    };
    rgba.push(val);
  }
  return rgba;
}
