mod memory;
mod player;
mod sound;

use anyhow::Result;
use cpal::traits::{DeviceTrait, StreamTrait};
use inline_colorization::*;
use log::*;
use player::Player;
use sid_file::SidFile;
use sound::Sound;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let filename = std::env::args().nth(1).unwrap_or("".to_string());
    let path = std::path::Path::new(&filename);
    let data = std::fs::read(path)?;

    let mut sound = Sound::new()?;
    let _ = sound.stream.play();

    let mut player = Player::new(&data)?;
    player.play();

    print_info(player.sid_file());
    print_sound_info(&sound)?;

    loop {
        if !player.playing {
            break;
        }

        if let Some(data) = player.data() {
            sound.write_blocking(&data[..]);
        }
    }

    Ok(())
}

pub fn print_info(sid_file: &SidFile) {
    println!("------------------------------------");
    println!(
        "{color_yellow}Song:     {color_blue}{}{color_reset}",
        sid_file.name
    );
    println!(
        "{color_yellow}Author:   {color_blue}{}{color_reset}",
        sid_file.author
    );
    println!(
        "{color_yellow}Released: {color_blue}{}{color_reset}",
        sid_file.released
    );
    println!(
        "{color_yellow}Songs:    {color_blue}{}{color_reset}",
        sid_file.songs
    );
    println!("------------------------------------");
    println!(
        "{color_cyan}Data length:  {color_green}{}{color_reset}",
        sid_file.data.len()
    );
    println!(
        "{color_cyan}Init address: {color_green}0x{:04x}{color_reset}",
        sid_file.init_address
    );
    println!(
        "{color_cyan}Play address: {color_green}0x{:04x}{color_reset}",
        sid_file.play_address
    );
    println!(
        "{color_cyan}Load address: {color_green}0x{:04x}{color_reset}",
        sid_file.load_address
    );
    println!(
        "{color_cyan}Real load address: {color_green}0x{:04x}{color_reset}",
        sid_file.real_load_address
    );
    println!("------------------------------------");
    if let Some(flags) = sid_file.flags {
        println!(
            "{color_cyan}Clock speed: {color_green}{:?}{color_reset}",
            flags.clock
        );
        println!(
            "{color_cyan}SID model 1: {color_blue}{:?}{color_reset}",
            flags.sid_model
        );
        println!(
            "{color_cyan}SID model 2: {color_blue}{:?}{color_reset}",
            flags.second_sid_model
        );
        println!(
            "{color_cyan}SID model 3: {color_blue}{:?}{color_reset}",
            flags.third_sid_model
        );
        println!("------------------------------------");
    }
}

pub fn print_sound_info(sound: &Sound) -> Result<()> {
    eprintln!("Output device: {}", sound.device.name()?);
    eprintln!(
        "Supported stream config: {:?}",
        sound.device.default_output_config()?
    );
    eprintln!("Stream config: {:?}", sound.config);

    Ok(())
}
