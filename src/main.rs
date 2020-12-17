use std::io;

fn main() -> io::Result<()> {
    let handle = io::stdin();
    let info_list: Vec<image_bg_extender::ImageInfo> = serde_json::from_reader(handle)?;
    for info in info_list {
        match image_bg_extender::compile_image(&info) {
            Ok(dest) => println!("Image saved to {}", dest),
            Err(e) => {
                if let Some(err) = e.downcast_ref::<io::Error>() {
                    eprintln!("IO error: {:?}", err)
                } else if let Some(err) = e.downcast_ref::<image::ImageError>() {
                    eprintln!("Image error: {:?}", err)
                }
            }
        }
    }
    Ok(())
}
