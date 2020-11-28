use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use dmi::icon;
use std::env;
use std::fs::File;
use std::path::Path;
use image::imageops;
use image::DynamicImage;
use image::ImageBuffer;

fn main() {
	let mut args: Vec<String> = env::args().collect();

	let _self_path = args.remove(0);
	if let Err(e) = resize_icons(args) {
		println!("Unable to resize icon: {:#?}", e);
		dont_disappear::any_key_to_continue::default();
	};
}

fn resize_icons(args: Vec<String>) -> Result<()> {
	for file_path_string in args.iter() {
		let path = Path::new(&file_path_string);
		let file;
		match File::open(&path) {
			Ok(f) => file = f,
			Err(e) => {
				println!("Wrong file path");
				dont_disappear::any_key_to_continue::default();
				bail!("{:#?}", e)
			}
		}
		let mut dmi =
			icon::Icon::load(&file).context(format!("Unable to load {} as dmi", file_path_string))?;

		let new_file_name_suffix = Some("-output");
		let new_width = 64;
		let new_height = 64;
		let new_x_offset = 16;
		let new_y_offset = 16;

		let initial_pixel = image::Rgba{0: [0,0,0,0]}; //Black color, no alpha. That's how BYOND starts images.
		let mut new_states = vec![];
		for icon_state in &dmi.states {
			let mut image_index = 0;
			let mut resized_images = vec![];
			for _dir in 0..icon_state.dirs {
				for _frame in 0..icon_state.frames {
					let mut new_frame = ImageBuffer::from_pixel(new_width, new_height, initial_pixel);
					imageops::replace(&mut new_frame, &icon_state.images[image_index], new_x_offset, new_y_offset);
					let new_frame = DynamicImage::ImageRgba8(new_frame);
					resized_images.push(new_frame);
					image_index += 1;
				}
			}
			let resized_state = icon::IconState {
				name: icon_state.name.clone(),
				dirs: icon_state.dirs,
				frames: icon_state.frames,
				images: resized_images,
				delay: icon_state.delay.clone(),
				loop_flag: icon_state.loop_flag,
				rewind: icon_state.rewind,
				movement: icon_state.movement,
				hotspot: icon_state.hotspot,
				unknown_settings: icon_state.unknown_settings.clone(),
			};
			new_states.push(resized_state);
		}

		dmi.width = new_width;
		dmi.height = new_height;
		dmi.states = new_states;

		let new_file_name = match new_file_name_suffix {
			None => file_path_string.clone(), //No change.
			Some(thing) => {
				let dot_offset = file_path_string.find(".dmi").unwrap_or(file_path_string.len());
				//Here we remove everything after the dot.
				let mut new_name: String = file_path_string.clone().drain(..dot_offset).collect();
				new_name.push_str(thing);
				new_name.push_str(".dmi");
				new_name
			}
		};
		let path = Path::new(&new_file_name);
		let mut file = File::create(&path).context("Unable to create file output path")?;
		dmi.save(&mut file).context("Unable to write output dmi")?;
	}
	Ok(())
}
