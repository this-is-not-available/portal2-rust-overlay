use egui::{ColorImage, Color32};
use std::cmp;

static IMAGE_DATA: &[u8; 4194304] = include_bytes!("duck.raw");

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 800;
pub const FOV: f32 = 45.0;

#[derive(Clone)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
    pub depth: f32,
}

#[derive(Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone)]
pub struct UV {
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    pub fn project(&self) -> ScreenPoint {
        if self.z < 0.1
        {
            return ScreenPoint{
                x: -1,
                y: -1,
                depth: 1.0,
            };
        }

        let fov_adjustment = 1.0 / FOV.tan();
        let scale_factor = HEIGHT as f32 / 300.0;
        let xndc = (self.x * fov_adjustment * scale_factor) / self.z;
        let yncd = (-self.y * fov_adjustment * scale_factor) / self.z;

        let xscreen = (xndc) * 0.5 * HEIGHT as f32;
        let yscreen = (yncd) * 0.5 * HEIGHT as f32;

        return ScreenPoint{
            x: (xscreen + WIDTH  as f32 * 0.5) as i32,
            y: (yscreen + HEIGHT as f32 * 0.5) as i32,
            depth: self.z};
    }

    pub fn translate(&self, x: f32, y: f32, z: f32) -> Vertex {
        return Vertex
        {
            x: self.x + x,
            y: self.y + y,
            z: self.z + z,
        };
    }

    pub fn rotate_x(&self, angle: f32) -> Vertex {
        let cos = angle.sin();
        let sin = angle.cos();
        return Vertex
        {
            x: self.x,
            y: cos * self.y - sin * self.z,
            z: sin * self.y + cos * self.z,
        };
    }
    
    pub fn rotate_y(&self, angle: f32) -> Vertex {
        let cos = angle.sin();
        let sin = angle.cos();
        return Vertex
        {
            x: cos * self.x + sin * self.z,
            y: self.y,
            z: -sin * self.x + cos * self.z
        };
    }

    pub fn rotate_z(&self, angle: f32) -> Vertex {
        let cos = angle.sin();
        let sin = angle.cos();
        return Vertex
        {
            x: cos * self.x - sin * self.y,
            y: sin * self.x + cos * self.y,
            z: self.z
        };
    }
}

pub struct Triangle {
    pub a: Vertex,// i32, // TODO: don't store the vertex in the triangle, store it in the model. Then use an index to that array to increase performance and avoid having 3 copies of the vertex
    pub b: Vertex,// i32,
    pub c: Vertex,// i32,
    pub u: usize,
    pub v: usize,
    pub w: usize,
}

pub struct Model {
    pub triangles: Vec<Triangle>,
    //pub verts: Vec<Vertex>,
    pub uvs: Vec<UV>,
    pub xrot: f32,
    pub yrot: f32,
    pub zrot: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Model {
    /// Projects all triangles and draws them using the provided rasterizer.
    pub fn render(&self, rasterizer: &mut Rasterizer) {
        for tri in &self.triangles {
            // Rotate around model origin
            let a_rot = tri.a.rotate_x(self.xrot).rotate_y(self.yrot).rotate_z(self.zrot);
            let b_rot = tri.b.rotate_x(self.xrot).rotate_y(self.yrot).rotate_z(self.zrot);
            let c_rot = tri.c.rotate_x(self.xrot).rotate_y(self.yrot).rotate_z(self.zrot);

            // Translate into place
            let a_translated = a_rot.translate(self.x, self.y, self.z);
            let b_translated = b_rot.translate(self.x, self.y, self.z);
            let c_translated = c_rot.translate(self.x, self.y, self.z);

            // Project onto screen
            let a_proj = a_translated.project();
            let b_proj = b_translated.project();
            let c_proj = c_translated.project();

            rasterizer.draw_triangle(
                a_proj,
                b_proj,
                c_proj,
                self.uvs[tri.u].clone(),
                self.uvs[tri.v].clone(),
                self.uvs[tri.w].clone(),
            );
        }
    }
}


pub struct Rasterizer<'a> {
    pub image: &'a mut ColorImage, // TODO: this is on the stack, no?
    pub depth_buffer: &'a mut Box<[f32; WIDTH*HEIGHT]>,
}

impl Rasterizer<'_> {
    pub fn clear(&mut self) {
        // TODO: could be optimized very much
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let r = 145;
                let g = 206;
                let b = 233;
                
                self.image.pixels[y * WIDTH + x] = Color32::from_rgb(r, g, b);
            }
        }
    }
    fn edge_function(a: ScreenPoint, b: ScreenPoint, c: ScreenPoint) -> i32
    {
        return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    }

    fn convert_barycentric_to_uv(uv1: &UV, uv2: &UV, uv3: &UV, a: f32, b: f32, c: f32) -> UV
    {
        let u = a * uv1.u + b * uv2.u + c * uv3.u;
        let v = a * uv1.v + b * uv2.v + c * uv3.v;

        return UV {u, v};
    }
    
    pub fn draw_triangle(&mut self, a: ScreenPoint, b: ScreenPoint, c: ScreenPoint, u: UV, v: UV, w: UV) {
        let min_x = cmp::max(cmp::min(cmp::min(a.x, b.x), c.x), 0);
        let min_y = cmp::max(cmp::min(cmp::min(a.y, b.y), c.y), 0);
        let max_x = cmp::min(cmp::max(cmp::max(a.x, b.x), c.x), WIDTH as i32 - 1);
        let max_y = cmp::min(cmp::max(cmp::max(a.y, b.y), c.y), HEIGHT as i32 - 1);

        let abc = Self::edge_function(a.clone(), b.clone(), c.clone());
        if abc <= 0
        {
            return;
        }

        let a01_change = a.y - b.y;
        let b01_change = b.x - a.x;
        let a12_change = b.y - c.y;
        let b12_change = c.x - b.x;
        let a20_change = c.y - a.y;
        let b20_change = a.x - c.x;

        let p = ScreenPoint {
            x: min_x,
            y: min_y,
            depth: 0.0,
        };
        let mut abp_row = Self::edge_function(a.clone(), b.clone(), p.clone());
        let mut bcp_row = Self::edge_function(b.clone(), c.clone(), p.clone());
        let mut cap_row = Self::edge_function(c.clone(), a.clone(), p.clone());

        // Calculate the index change for each row
        //let indexChange = WIDTH as i32 - (maxX - minX + 1) + HEIGHT as i32;

        let mut y = min_y;
        while y <= max_y
        {
            let mut index = HEIGHT * y as usize + min_x as usize;
            let mut abp = abp_row;
            let mut bcp = bcp_row;
            let mut cap = cap_row;

            let mut x = min_x;
            while x <= max_x
            {
                if abp >= 0 && bcp >= 0 && cap >= 0
                {
                    let weight_a = bcp as f32 / abc as f32;
                    let weight_b = cap as f32 / abc as f32;
                    let weight_c = abp as f32 / abc as f32;

                    let depth = weight_a * a.depth + weight_b * b.depth + weight_c * c.depth;
                    
                    if depth < self.depth_buffer[index]
                    {
                        self.depth_buffer[index] = depth;
                        
                        let uv = Self::convert_barycentric_to_uv(&u, &v, &w, weight_a, weight_b, weight_c);

                        // Clamp UVs to [0, 1]
                        let texturex = uv.u.clamp(0.0, 1.0);
                        let texturey = uv.v.clamp(0.0, 1.0);

                        // Map UVs to 1024x1024 texture
                        let tx = (texturex * 1023.0).round().clamp(0.0, 1023.0) as usize;
                        let ty = (texturey * 1023.0).round().clamp(0.0, 1023.0) as usize;
                        let texture_index = ty * 1024 + tx;

                        let r = IMAGE_DATA[texture_index * 4 + 0];
                        let g = IMAGE_DATA[texture_index * 4 + 1];
                        let b = IMAGE_DATA[texture_index * 4 + 2];

                        self.image.pixels[index] = Color32::from_rgb(r,g,b);

                    }
                }

                x += 1;

                index += 1;
                abp += a01_change;
                bcp += a12_change;
                cap += a20_change;
            }

            abp_row += b01_change;
            bcp_row += b12_change;
            cap_row += b20_change;

            y += 1;
        }
    }
}