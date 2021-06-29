use std::ops::{Neg, Sub, Add, Mul, Div};
use ::sdl2::pixels::Color;
use ::rand::thread_rng;
use ::rand::Rng;

#[derive(Debug, PartialEq, Clone)]
pub struct Space {
    pub planets: Vec<Planet>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Planet {
    pub mass: f64,
    pub position: Position,
    pub last_position: Position,
    pub trail_color: Color,
    pub velocity: Velocity,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
}

impl Space {
    pub fn new() -> Self {
        Space {
            planets: Vec::new(),
        }
    }
    pub fn simulate(&mut self, time: f64) -> f32 {
        let mut shake = 0.;
        for planet1 in 0..self.planets.len() {
            for planet2 in 0..self.planets.len() {
                if planet1 == planet2 {
                    continue;
                }
                let distance = (
                    (self.planets[planet2].position.x-self.planets[planet1].position.x).powi(2) +
                    (self.planets[planet2].position.y-self.planets[planet1].position.y).powi(2)
                ).sqrt();
                let acceleration = self.planets[planet2].mass/distance.powi(2);
                let speed = acceleration*time;
                let velocity = Velocity{
                    x: speed*(self.planets[planet2].position.x-self.planets[planet1].position.x)/distance,
                    y: speed*(self.planets[planet2].position.y-self.planets[planet1].position.y)/distance
                };
                self.planets[planet1].add_velocity(velocity);
            }
            // Finally move the planets
            self.planets[planet1].move_for(time);
        }
        // collisions
        let mut planets_to_remove = Vec::new();
        for planet1 in 0..self.planets.len() {
            for planet2 in (planet1+1)..self.planets.len() {
                let distance = (
                    (self.planets[planet2].position.x-self.planets[planet1].position.x).powi(2) +
                    (self.planets[planet2].position.y-self.planets[planet1].position.y).powi(2)
                ).sqrt();
                let mass1 = self.planets[planet1].mass;
                let mass2 = self.planets[planet2].mass;
                if distance < 400. {
                    // merge the planets
                    let (bigger, smaller) = if mass1 < mass2 { (planet2, planet1) } else { (planet1, planet2) };
                    let smaller_velocity = self.planets[smaller].velocity;
                    let smaller_mass = self.planets[smaller].mass;
                    let bigger_mass = self.planets[bigger].mass;
                    self.planets[bigger].mass += smaller_mass;
                    self.planets[bigger].add_velocity(smaller_velocity*smaller_mass/bigger_mass);
                    planets_to_remove.push(smaller);

                    // add some screenshake
                    if mass1 > 2e11 && mass2 > 2e11 {
                        shake += (mass1*mass2/1e23) as f32;
                    }
                }
            }
        }
        planets_to_remove.sort();
        planets_to_remove.reverse();
        for i in planets_to_remove {
            self.planets.remove(i);
        }
        shake
    }
}

impl Planet {
    pub fn new(mass: f64, position: Position, velocity: Velocity) -> Self {
        let mut rng = thread_rng();
        Planet {
            mass,
            position,
            last_position: position,
            trail_color: Color::RGBA(rng.gen(), rng.gen(), rng.gen(), 255),
            velocity,
        }
    }
    fn move_for(&mut self, time: f64) {
        self.position = self.position + self.velocity.to_position()*time;
    }
    pub fn add_velocity(&mut self, velocity: Velocity) {
        self.velocity = self.velocity + velocity;
    }
    pub fn radius(mass: f64) -> f64 {
        mass.powf(1./3.)/3.52
    }
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Position {
            x, y,
        }
    }
}

impl Velocity {
    pub fn new(x: f64, y: f64) -> Self {
        Velocity {
            x, y,
        }
    }
    pub fn to_position(self) -> Position {
        Position::new(self.x, self.y)
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f64> for Position {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<f64> for Position {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        let other = if other == 0. { std::f64::EPSILON } else { other };
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Add for Velocity {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Velocity {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f64> for Velocity {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<f64> for Velocity {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        let other = if other == 0. { std::f64::EPSILON } else { other };
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

impl Neg for Velocity {
    type Output = Self;

    fn neg(self) -> Self {
        Velocity {
            x: -self.x,
            y: -self.y,
        }
    }
}
