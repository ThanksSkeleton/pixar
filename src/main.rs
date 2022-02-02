use std::ops::{Add, Mul};
use rand::Rng;
use rayon::prelude::*;
use rayon::iter::once;


#[derive(Debug, Copy, Clone)]
struct Vec3 
{
    x: f32,
    y: f32,
    z: f32
}

impl Vec3 
{
    fn new(x: f32, y: f32, z: f32) -> Self
    {
        Vec3 { x, y, z }
    }

    fn magmult(&self, other: &Vec3) -> f32 
    {
        return self.x * other.x + self.y * other.y + self.z * other.z;
    }

    fn normalize(self) -> Vec3 
    {
        return self * (1.0 / &self.magmult(&self).sqrt());
    }
}

fn new_zero() -> Vec3 
{
    return Vec3::new(0.0,0.0,0.0);
}

fn new_one(f: f32) -> Vec3
{
    return Vec3::new(f,f,f);
}

fn new_two(x: f32, y:f32) -> Vec3
{
    return Vec3::new(x,y,0.0);
}

// This sucks!

// Vec3 Vec3
impl Add<Vec3> for Vec3 
{
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Self {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
    }
}

// &Vec3 &Vec3
impl Add<&Vec3> for &Vec3 
{
    type Output = Vec3;

    fn add(self, other: &Vec3) -> Vec3 {
      Vec3 {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
    }
}

// Vec3 &Vec3
impl Add<Vec3> for &Vec3 
{
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
      Vec3 {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
    }
}

// &Vec3 Vec3
impl Add<&Vec3> for Vec3 
{
    type Output = Vec3;

    fn add(self, other: &Vec3) -> Vec3 {
      Vec3 {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
    }
}

// Vec3 Vec3
impl Mul<Vec3> for Vec3 
{
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {x: self.x * other.x, y: self.y * other.y, z: self.z * other.z}
    }
}

// &Vec3 &Vec3
impl Mul<&Vec3> for &Vec3 
{
    type Output = Vec3;

    fn mul(self, other: &Vec3) -> Vec3 {
        Vec3 {x: self.x * other.x, y: self.y * other.y, z: self.z * other.z}
    }
}

// Vec3 Vec3
impl Mul<Vec3> for &Vec3 
{
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {x: self.x * other.x, y: self.y * other.y, z: self.z * other.z}
    }
}

// &Vec3 &Vec3
impl Mul<&Vec3> for Vec3 
{
    type Output = Vec3;

    fn mul(self, other: &Vec3) -> Vec3 {
        Vec3 {x: self.x * other.x, y: self.y * other.y, z: self.z * other.z}
    }
}

impl Mul<f32> for Vec3 
{
    type Output = Self;

    fn mul(self, scalar: f32) -> Vec3 {
        Self {x: self.x * scalar, y: self.y * scalar, z: self.z * scalar}
    }
}

impl Mul<f32> for &Vec3 
{
    type Output = Vec3;

    fn mul(self, scalar: f32) -> Vec3 {
        Vec3 {x: self.x * scalar, y: self.y * scalar, z: self.z * scalar}
    }
}

fn min(a : f32, b: f32) -> f32 
{
  if a < b { a } else { b }
}

fn random_val() -> f32 
{
    let mut rng = rand::thread_rng();
    return rng.gen_range(0.0..1.0);
}

// Rectangle CSG equation. Returns minimum signed distance from
// space carved by
// lowerLeft vertex and opposite rectangle vertex upperRight.
fn BoxTest(position: &Vec3, lowerLeft: &Vec3, upperRight: &Vec3) -> f32
{
  let lowerLeft_2 = position + &(lowerLeft * -1.0);
  let upperRight_2 = upperRight + &(position * -1.0);
  return -min(
          min(
                  min(lowerLeft_2.x, upperRight_2.x),
                  min(lowerLeft_2.y, upperRight_2.y)),
          min(lowerLeft_2.z, upperRight_2.z));
}

#[derive(Debug, PartialEq)]
enum HitType 
{
    NONE,
    LETTER,
    WALL,
    SUN
}

fn ctf(c: char) -> f32
{
  return (((c as u8) as i8) - 79) as f32;
}

// Sample the world using Signed Distance Fields.
fn QueryDatabase(position: &Vec3) -> SimpleHit
{
  let no_hit = SimpleHit { distance: 1e9, hit_type : HitType::NONE };

  let words : [[char; 4]; 15] = [
               // 15 two points lines
          ['5','O','5','_'], ['5','W','9','W'], ['5','_','9','_'],                     // P (without curve)
          ['A','O','E','O'], ['C','O','C','_'], ['A','_','E','_'],                     // I
          ['I','O','Q','_'], ['I','_','Q','O'],                                        // X
          ['U','O','Y','_'], ['Y','_',']','O'], ['W','W','[','W'],                     // A
          ['a','O','a','_'], ['a','W','e','W'], ['a','_','e','_'], ['c','W','i','O']]; // R (without curve)

  let curves = [new_two(11.0, 6.0),new_two(-11.0, 6.0)];

  return 
          once(no_hit).
          chain(once(room_hit(position))).
          chain(words.into_par_iter().map(|w| letter_hit(w, position))).
          chain(curves.into_par_iter().map(|c| curve_hit(&c, position))).
          chain(once(sun_hit(position))).
          min_by(|sh_a, sh_b| if sh_a.distance < sh_b.distance { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater } ).unwrap();
}

fn letter_hit(word: [char; 4], position: &Vec3) -> SimpleHit
{
  let flattened_position = new_two(position.x, position.y); // Flattened position (z=0)

  let begin = new_two(ctf(word[0]), ctf(word[1])) * 0.5;
  let end = new_two(ctf(word[2]), ctf(word[3])) * 0.5 + &begin * -1.0;

  let o = &flattened_position + (&begin + &end * min(-min((begin + &flattened_position * -1.0).magmult(&end) / end.magmult(&end), 0.0), 1.0)) * -1.0;
  let distance = o.magmult(&o).sqrt();
  let distance_2 = (distance.powi(8) + position.z.powi(8)).powf(0.125) - 0.5;
  return SimpleHit { distance : distance_2, hit_type : HitType::LETTER }; 
}

fn curve_hit(curve: &Vec3, position: &Vec3) -> SimpleHit 
{
  let f = new_two(position.x, position.y); // Flattened position (z=0)
  let o = &f + curve * -1.0;
  let distance =     
    if o.x > 0.0 { 
      (o.magmult(&o).sqrt() - 2.0).abs() 
    } else { 
    let o_mod = if o.y > 0.0 { -2.0 } else { 2.0 };
    let o_2 = Vec3::new(o.x, o.y+o_mod, o.z);
    (o_2.magmult(&o_2)).sqrt()};
  let distance_2 = (distance.powi(8) + position.z.powi(8)).powf(0.125) - 0.5;
  return SimpleHit { distance : distance_2, hit_type : HitType::LETTER }; 
}

fn room_hit(position: &Vec3) -> SimpleHit 
{
  let roomDist = min(// min(A,B) = Union with Constructive solid geometry
               //-min carves an empty space
                -min(// Lower room
                     BoxTest(&position, &Vec3::new(-30.0, -0.5, -30.0), &Vec3::new(30.0, 18.0, 30.0)),
                     // Upper room
                     BoxTest(&position, &Vec3::new(-25.0, 17.0, -25.0), &Vec3::new(25.0, 20.0, 25.0))
                ),
                BoxTest( // Ceiling 'planks' spaced 8 units apart.
                  &Vec3::new((position.x).abs().rem_euclid(8.0),
                      position.y,
                      position.z),
                    &Vec3::new(1.5, 18.5, -25.0),
                    &Vec3::new(6.5, 20.0, 25.0)
                )
  );
  return SimpleHit { distance : roomDist, hit_type : HitType::WALL }; 
}

fn sun_hit(position: &Vec3) -> SimpleHit 
{
  return SimpleHit { distance: 19.9 - position.y, hit_type: HitType::SUN };
}

#[derive(Debug)]
struct SimpleHit 
{
  distance: f32,
  hit_type: HitType
}

#[derive(Debug)]
struct Hit
{
  hit_position: Vec3,
  hit_normal: Vec3,
  hit_type: HitType
}

impl Hit 
{
  fn new(hit_position: Vec3, hit_normal: Vec3, hit_type: HitType) -> Self
  {
      Hit{ hit_position, hit_normal, hit_type }
  }
}

// Perform signed sphere marching
// Returns hitType 0, 1, 2, or 3 and update hit position/normal
fn RayMarching(origin: &Vec3, direction: &Vec3) -> Hit
{
  //println!("RayMarching: in: {:?} {:?}", origin, direction);

  let mut noHitCount = 0;

  // Signed distance marching
  let mut total_d = 0.0;
  let mut noHitCount = 0;
  while total_d < 100.0 && noHitCount <= 99
  {
    let hitPos = origin + direction * total_d;
    let q1 =  QueryDatabase(&hitPos); 
    if q1.distance < 0.01 
    {
        let points : Vec<f32> = [new_two(0.01, 0.0), new_two(0.0, 0.01), Vec3::new(0.0, 0.0, 0.01)].into_par_iter().
                     map(|pt| QueryDatabase(&(&hitPos + pt)).distance - q1.distance).
                     collect();
        let hitNorm = Vec3::new(*points.get(0).unwrap(), *points.get(1).unwrap(), *points.get(2).unwrap()).normalize();
        return Hit::new(hitPos, hitNorm, q1.hit_type);
    } else {
      total_d += q1.distance;
      noHitCount += 1;
    }
  }

  return Hit::new(new_zero(), new_zero(), HitType::NONE);
}

fn trace(original_origin: &Vec3, original_direction: &Vec3, max_bounce: i32) -> Vec3 
{
  let mut direction = *original_direction;
  let mut origin = *original_origin;
  let mut attenuation = new_one(1.0);  
  let mut color = new_one(0.0);

  let light_direction = Vec3::new(0.6, 0.6, 1.0).normalize(); // Directional light

  for _bounce_count in 0..max_bounce
  {
    let full_hit = RayMarching(&origin, &direction);

    //println!("{:?} full_hit", full_hit);
    match full_hit.hit_type 
    {
      HitType::NONE => {  break; }
      HitType::LETTER => 
      {
        // Specular bounce on a letter. No color acc.
        direction = direction + full_hit.hit_normal * ( full_hit.hit_normal.magmult(&direction) * -2.0);
        origin = full_hit.hit_position + direction * 0.1;
        attenuation = attenuation * 0.2; // Attenuation via distance traveled.
      }
      HitType::WALL => 
      { // Wall hit uses color yellow?
        let incidence : f32 = full_hit.hit_normal.magmult(&light_direction);

        let random_angle : f32 = 6.283185 * random_val();
        let c : f32 = random_val();
        let s : f32 = (1.0 - c).sqrt();

        let g : f32 = if full_hit.hit_normal.z < 0.0 { -1.0 } else { 1.0 };
        let u : f32 = -1.0 / (g + full_hit.hit_normal.z);
        let v : f32 = full_hit.hit_normal.x * full_hit.hit_normal.y * u;

        direction = Vec3::new(v,
                              g + full_hit.hit_normal.y * full_hit.hit_normal.y * u,
                              -full_hit.hit_normal.y) * (random_angle.cos() * s)
                    +
                    Vec3::new(1.0 + g * full_hit.hit_normal.x * full_hit.hit_normal.x * u,
                              g * v,
                              -g * full_hit.hit_normal.x) * (random_angle.sin() * s) + full_hit.hit_normal * c.sqrt();
        origin = full_hit.hit_position + direction * 0.1;
        attenuation = attenuation * 0.2;
        if incidence > 0.0 && RayMarching(&(full_hit.hit_position + full_hit.hit_normal * 0.1), &light_direction).hit_type == HitType::SUN
        {
          color = color + attenuation * Vec3::new(500.0, 400.0, 100.0) * incidence;
          break;
        }
      }
      HitType::SUN => 
      {
        color = color + attenuation * Vec3::new(50.0, 80.0, 100.0); 
        break; // Sun Color
      }
    }
  }
  return color;
}

fn trace_debug(origin: &Vec3, direction: &Vec3, _max_bounce: &i32) -> Vec3 
{
  let light_direction = Vec3::new(0.6, 0.6, 1.0).normalize(); // Directional light

    let full_hit = RayMarching(&origin, &direction);

    //println!("{:?} full_hit", full_hit);
    match full_hit.hit_type 
    {
      HitType::NONE => {  return new_zero(); }
      HitType::LETTER => { return Vec3::new(1.0, 0.0, 0.0)}
      HitType::WALL => 
      {
        let incidence : f32 = full_hit.hit_normal.magmult(&light_direction);
        if incidence > 0.0 && RayMarching(&(full_hit.hit_position + full_hit.hit_normal * 0.1), &light_direction).hit_type == HitType::SUN
        {
          return Vec3::new(0.0, 1.0, 0.0);
        } else {
          return Vec3::new(0.0, 0.0, 1.0);
        }
      }
      HitType::SUN => { return new_one(1.0);}
    }
}

#[derive(Debug)]
struct DrawParameters 
{
  width: u32,
  height: u32,

  samples_count: i32,
  bounces: i32,

  name: String,

  trace_debug: bool,
  color_debug: bool
}


fn main() {

  let simple_tiny = DrawParameters 
  {
    width: 20,
    height: 20,

    samples_count: 8,
    bounces: 2,

    name: String::from("simplest_tiny.png"),

    trace_debug: false,
    color_debug: false
  };


  let simple_4_2_samples = DrawParameters 
  {
    width: 200,
    height: 200,

    samples_count: 4,
    bounces: 2,

    name: String::from("simplest_4x2.png"),

    trace_debug: false,
    color_debug: false
  };

  let simple_8_3_samples = DrawParameters 
  {
    width: 200,
    height: 200,

    samples_count: 8,
    bounces: 2,

    name: String::from("simplest_8x3.png"),

    trace_debug: false,
    color_debug: false
  };

  let simple = DrawParameters
  {
    width: 200,
    height: 200,

    samples_count: 1,
    bounces: 1,

    name: String::from("simplest.png"),

    trace_debug: false,
    color_debug: false
  };

  let simple_huge = DrawParameters
  {
    width: 960,
    height: 540,

    samples_count: 8,
    bounces: 3,

    name: String::from("simplest_huge.png"),

    trace_debug: false,
    color_debug: false
  };

  draw(simple_tiny);  
  draw(simple);  
  draw(simple_4_2_samples);
  draw(simple_8_3_samples);
  draw(simple_huge);

  //draw(simple_huge);

}// Andrew Kensler

fn draw(dp: DrawParameters) 
{
  println!("{:?} draw start", dp);
  let mut imgbuf = image::ImageBuffer::new(dp.width, dp.height);


  let position = Vec3::new(-22.0, 5.0, 25.0);
  let goal = (Vec3::new(-3.0, 4.0, 0.0) + &position * -1.0).normalize();
  let left = Vec3::new(goal.z, 0.0, -goal.x).normalize() * (1 as f32 / dp.width as f32);
  // Cross-product to get the up Vec3tor

  let up = Vec3::new( goal.y * left.z - goal.z * left.y,
                      goal.z * left.x - goal.x * left.z,
                      goal.x * left.y - goal.y * left.x);

  let colors: Vec<(u32, u32, Vec3)>= (0..dp.height).into_par_iter().
  flat_map(|y| (0..dp.width).into_par_iter().map(move |x| (x, y))).
  map(|(x,y)| 
  {
      let direction = &(&goal + &left * (x as f32 - dp.width as f32 / 2.0 + random_val()) + &up * (y as f32 - dp.height as f32 / 2.0 + random_val())).normalize();


      let color = (0..dp.samples_count).into_par_iter().
                  map(|_i| if dp.trace_debug { trace_debug(&position, direction, &dp.bounces) } else { trace(&position, direction, dp.bounces) }).
                  reduce(|| new_zero(), |acc, c| acc+c) * (1.0 / dp.samples_count as f32);
      return (x, y, color);
  }).collect();

  for c in colors 
  {
    output(&mut imgbuf, dp.width - c.0 - 1, dp.height - c.1 - 1, if dp.color_debug { color_mapping_debug(c.2) } else { color_mapping(c.2) });
  }

  imgbuf.save(dp.name).unwrap();
}

fn color_mapping_debug(color: Vec3) -> [u8; 3]
{  //println!("point {} {} {:?}", x, y, color);

  // // Reinhard tone mapping
  // color = color * ((1.0 / samples_Count as f32) + (14.0 / 241 as f32));

  //println!("point {} {} {:?}", x, y, color);

  let color = color * 255.0;
  return [color.x as u8, color.y as u8, color.z as u8];
}


fn color_mapping(color: Vec3) -> [u8; 3]
{  //println!("point {} {} {:?}", x, y, color);

  // // Reinhard tone mapping
  // color = color * ((1.0 / samples_Count as f32) + (14.0 / 241 as f32));

  //println!("point {} {} {:?}", x, y, color);

  let o = &color + new_one(1.0);
  let color = Vec3::new(color.x / o.x, color.y / o.y, &color.z / o.z) * 255.0;
  return [color.x as u8, color.y as u8, color.z as u8];
}


fn output(imgbuf : &mut image::ImageBuffer<image::Rgb<u8>, Vec<u8>>, x: u32, y: u32, color : [u8; 3]) 
{
  let pixel = imgbuf.get_pixel_mut(x, y);
  *pixel = image::Rgb(color);
}