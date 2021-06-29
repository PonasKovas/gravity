#![feature(proc_macro_hygiene)]

mod shared;

use rand::{thread_rng, Rng};
use sdl2::pixels::Color;
use sdl2::event::Event;
use std::time::{Instant, Duration};
use sdl2::rect::{Point, Rect};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator};
use std::collections::{HashSet, HashMap};
use sdl2::video::WindowContext;
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Scancode;
use sdl2::rwops::RWops;

use shared::{Space, Planet, Position, Velocity};

const CAMERA_SPEED: f64 = 500.; // pixels per second

#[derive(PartialEq, Debug, Clone, Copy)]
enum PlanetCreation {
    NotStarted,
    Mass(Instant, f64, f64), // when started, and coordinates
    Velocity(f64, f64, f64), // coordinates and mass
}

fn load_assets(texture_creator: &TextureCreator<WindowContext>) -> HashMap<&str, Texture> {
    let sprites_files = embeddir::embed!("assets/sprites");

    let mut sprites = HashMap::new();

    for (name, bytes) in sprites_files {
        let dimensions = (u16::from_le_bytes([bytes[12],bytes[13]]) as u32,
                          u16::from_le_bytes([bytes[14],bytes[15]]) as u32);
        let mut texture = texture_creator.create_texture_target(Some(PixelFormatEnum::ABGR8888), dimensions.0, dimensions.1).unwrap();
        texture.update(None, &bytes[18..], dimensions.0 as usize*4).unwrap();
        texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        sprites.insert(name, texture);
    }

    sprites
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    // Initialize a font
    let font = ttf_context.load_font_from_rwops(
        RWops::from_bytes(include_bytes!("../assets/Roboto-Regular.ttf")).unwrap(),
        18).unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Gravity", 800, 600)
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let sprites = load_assets(&texture_creator);
    let planet_texture = &sprites["black.tga"];
    let new_planet_texture = &sprites["new.tga"];

    let mut trail_texture = texture_creator.create_texture_target(
        Some(PixelFormatEnum::ABGR8888),
        2048, 1080
    ).unwrap();
    trail_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.with_texture_canvas(&mut trail_texture, |texture_canvas| {
        texture_canvas.set_draw_color(Color::RGBA(12, 3, 20, 255));
        texture_canvas.clear();
    }).unwrap();

    // construct the space
    let mut space = Space::new();

    let mut planet_creation = PlanetCreation::NotStarted;

    let mut paused = false;
    let mut simulation_speed = 1f64;
    let mut screenshake: f32 = 20.;
    let mut camera_pos = (0f64, 0f64);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut prev_buttons = HashSet::new();
    let mut prev_keys = HashSet::new();

    let mut last_frame = Instant::now();
    let mut last_simulation_step = Instant::now();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                _ => {}
            }
        }

        let window_size = canvas.window().drawable_size();

        let mouse_state = event_pump.mouse_state();
        let mouse_x = (-camera_pos.0 as i32 + mouse_state.x() - window_size.0 as i32/2) as f64 * 100.;
        let mouse_y = (-camera_pos.1 as i32 + mouse_state.y() - window_size.1 as i32/2) as f64 * 100.;

        if !paused {
            screenshake += space.simulate(simulation_speed * last_simulation_step.elapsed().as_secs_f64());
            if screenshake > 20. {
                screenshake = 20.;
            }
        }
        last_simulation_step = Instant::now();

        if last_frame.elapsed() > Duration::new(0, 1_000_000_000/60) {
            last_frame = Instant::now();

            // calculate the random shake for this frame, if screenshake is more than 0
            let mut rng = thread_rng();
            let shake = (
                (4.*screenshake*(rng.gen::<f32>()*2.-1.)).round() as i32,
                (4.*screenshake*(rng.gen::<f32>()*2.-1.)).round() as i32,
            );
            screenshake -= 1.;
            if screenshake < 0. {
                screenshake = 0.;
            }

            canvas.set_draw_color(Color::RGBA(12, 3, 20, 255));
            // render trails
            canvas.copy_ex(
                &trail_texture,
                None,
                Some(
                    Rect::new(
                        camera_pos.0 as i32+shake.0+((window_size.0 as i32/2)-1024),
                        camera_pos.1 as i32+shake.1+((window_size.1 as i32/2)-540),
                        2048,1080
                    )
                ),
                0.,
                None,
                false,
                false
            ).unwrap();

            // render the planets and trails
            for planet in &mut space.planets {
                let last_pos = Point::new((planet.last_position.x/100.) as i32+1024, (planet.last_position.y/100.) as i32+540);
                let pos = Point::new((planet.position.x/100.) as i32+1024, (planet.position.y/100.) as i32+540);
                // render a trail line
                canvas.with_texture_canvas(&mut trail_texture, |texture_canvas| {
                    texture_canvas.set_draw_color(planet.trail_color);
                    texture_canvas.draw_line(last_pos, pos).unwrap();
                }).unwrap();
                planet.last_position = planet.position;
                let size: u32 = (Planet::radius(planet.mass)*2./100.).round() as u32;
                canvas.copy_ex(
                    planet_texture,
                    None,
                    Some(
                        Rect::new(
                            camera_pos.0 as i32+shake.0+window_size.0 as i32/2+(planet.position.x/100.).round() as i32-(size/2) as i32,
                            camera_pos.1 as i32+shake.1+window_size.1 as i32/2+(planet.position.y/100.).round() as i32-(size/2) as i32,
                            size, size)
                    ),
                    0.,//90.+planet.velocity.y.atan2(planet.velocity.x)*180./PI,
                    None,
                    false,
                    false
                ).unwrap();
            }

            let buttons = mouse_state.pressed_mouse_buttons().collect();
            let pressed_buttons = &buttons - &prev_buttons;
            let released_buttons = &prev_buttons - &buttons;
            prev_buttons = buttons;

            match planet_creation {
                PlanetCreation::NotStarted => {
                    // check if left pressed
                    for button in &pressed_buttons {
                        if *button == MouseButton::Left {
                            // Start creation
                            planet_creation = PlanetCreation::Mass(Instant::now(), mouse_x, mouse_y);
                            break;
                        }
                    }
                }
                PlanetCreation::Mass(instant, x, y) => {
                    let mass: f64 = ((instant.elapsed().as_secs_f64()+1.25).powi(2))*16e9;
                    let size: u32 = (Planet::radius(mass)*2./100.).round() as u32;
                    // render it
                    canvas.copy_ex(
                        new_planet_texture,
                        None,
                        Some(
                            Rect::new(
                                camera_pos.0 as i32+shake.0+window_size.0 as i32/2 + (x/100.) as i32 - (size/2) as i32,
                                camera_pos.1 as i32+shake.1+window_size.1 as i32/2 + (y/100.) as i32 - (size/2) as i32,
                                size, size)
                        ),
                        0.,
                        None,
                        false,
                        false
                    ).unwrap();
                    // check if left released
                    for button in &released_buttons {
                        if *button == MouseButton::Left {
                            // Set mass and continue creation
                            planet_creation = PlanetCreation::Velocity(x, y, mass);
                            break;
                        }
                    }
                    // check if right pressed
                    for button in &pressed_buttons {
                        if *button == MouseButton::Right {
                            // Increase mass
                            planet_creation = PlanetCreation::Mass(
                                instant-instant.elapsed()/2, x, y
                            );
                            break;
                        }
                    }
                }
                PlanetCreation::Velocity(x, y, mass) => {
                    // render a line from planet creation position to cursor
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    canvas.draw_line(
                        Point::new(
                            camera_pos.0 as i32+shake.0 + (x/100.) as i32 + (window_size.0 as i32/2), camera_pos.1 as i32+shake.1 + (y/100.) as i32 + (window_size.1 as i32/2),
                        ),
                        Point::new(shake.0 + mouse_state.x(), shake.1 + mouse_state.y())
                    ).unwrap();
                    // check if left pressed
                    for button in &pressed_buttons {
                        if *button == MouseButton::Left {
                            // velocity and finish planet creation
                            let velocity = Velocity::new(
                                mouse_x-x,
                                mouse_y-y,
                            );
                            space.planets.push(
                                Planet::new(mass, Position::new(x, y), velocity)
                            );
                            planet_creation = PlanetCreation::NotStarted;
                            break;
                        }
                    }
                }
            }

            // status text
            let status_text_surface = font.render(&format!(
                    "{paused}{planets_number} planets, {speed}x speed, camera position: {cam_x:.3}x{cam_y:.3}, {planetcreation}",
                    paused=if paused { "PAUSED, " } else {""},
                    planets_number=space.planets.len(),
                    speed=simulation_speed,
                    cam_x=camera_pos.0,
                    cam_y=camera_pos.1,
                    planetcreation=match planet_creation {
                        PlanetCreation::NotStarted => "not creating planets right now".to_owned(),
                        PlanetCreation::Mass(instant, ..) => format!("mass: {:.3e} kg",
                                                ((instant.elapsed().as_secs_f64()+1.25).powi(2))*16e9
                                            ),
                        PlanetCreation::Velocity(..) => "getting inital velocity of planet".to_owned(),
                    }
                )).solid(Color::RGB(255, 255, 255)).unwrap();
            let size = (
                status_text_surface.width(),
                status_text_surface.height(),
            );
            let status_text_texture = texture_creator.create_texture_from_surface(status_text_surface).unwrap();
            canvas.copy_ex(
                &status_text_texture,
                None,
                Some(Rect::new(
                    5, 5,
                    size.0, size.1
                )),
                0.,
                None,
                false,
                false,
            ).unwrap();

            canvas.present();
        }


        let keyboard_state = event_pump.keyboard_state();
        let keys = keyboard_state.pressed_scancodes().collect();
        let pressed_keys = &keys - &prev_keys;
        for scancode in pressed_keys {
            if scancode == Scancode::Space {
                // pause simulation
                paused = !paused;
            }
            else if scancode == Scancode::C {
                // clear trails
                canvas.with_texture_canvas(&mut trail_texture, |texture_canvas| {
                    texture_canvas.set_draw_color(Color::RGBA(12, 3, 20, 255));
                    texture_canvas.clear();
                }).unwrap();
            }
            else if scancode == Scancode::N {
                // clear planets
                space.planets.clear();
            }
            else if scancode == Scancode::S{
                // increase simulation speed
                if simulation_speed < 128. {
                    simulation_speed *= 2.;
                }
            }
            else if scancode == Scancode::A{
                // decrease simulation speed
                if simulation_speed > 0.0078125 {
                    simulation_speed /= 2.;
                }
            }
        }
        for &scancode in &keys {
            if scancode == Scancode::Up {
                camera_pos.1 += CAMERA_SPEED*last_simulation_step.elapsed().as_secs_f64();
            }
            if scancode == Scancode::Down {
                camera_pos.1 -= CAMERA_SPEED*last_simulation_step.elapsed().as_secs_f64();
            }
            if scancode == Scancode::Left {
                camera_pos.0 += CAMERA_SPEED*last_simulation_step.elapsed().as_secs_f64();
            }
            if scancode == Scancode::Right {
                camera_pos.0 -= CAMERA_SPEED*last_simulation_step.elapsed().as_secs_f64();
            }
        }
        prev_keys = keys;
    }
}
