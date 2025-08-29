use core::f32;
use std::{error::Error, f32::consts::PI};

use glam::{Mat3, Vec3, Vec2};
use pixels::{Pixels, SurfaceTexture};
use winit::{dpi::LogicalSize, event_loop::{EventLoop}, window::{WindowBuilder}};


const WIDTH:u32 = 400;
const HEIGHT:u32 = 400;

fn hello_cube_cpu() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("CPU Hello Cube")
        .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let vertices: [Vec3; 8] = [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new( 0.5, -0.5, -0.5),
        Vec3::new( 0.5,  0.5, -0.5),
        Vec3::new(-0.5,  0.5, -0.5),
        Vec3::new(-0.5, -0.5,  0.5),
        Vec3::new( 0.5, -0.5,  0.5),
        Vec3::new( 0.5,  0.5,  0.5),
        Vec3::new(-0.5,  0.5,  0.5),
    ];

    let indices: [[usize; 3]; 12] = [
        [0, 1, 2], [0, 2, 3], // back
        [4, 5, 6], [4, 6, 7], // front
        [0, 1, 5], [0, 5, 4], // bottom
        [2, 3, 7], [2, 7, 6], // top
        [1, 2, 6], [1, 6, 5], // right
        [0, 3, 7], [0, 7, 4], // left
    ];
    let colors: [[u8; 4]; 12] = [
        [255,0,0,255], [255,0,0,255],
        [255,0,0,255], [255,0,0,255],
        [0,255,0,255], [0,255,0,255],
        [0,255,0,255], [0,255,0,255],
        [255,0,255,255], [255,0,255,255],
        [255,0,255,255], [255,0,255,255],
    ];

    let mut depth_buffer = vec![f32::INFINITY; (WIDTH*HEIGHT) as usize];
    let mut angle: f32 = PI / 4.0;
    
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                if let winit::event::WindowEvent::CloseRequested = event {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
            }
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            
            winit::event::Event::RedrawRequested(_) => {
                // Clear frame
                let frame = pixels.frame_mut();
                for pixel in frame.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[10, 10, 20, 255]);
                }

                // Clear depth buffer
                for d in depth_buffer.iter_mut() {
                    *d = f32::INFINITY;
                }
                
                // Apply rotation matrix
                let rot_m = Mat3::from_rotation_y(angle)*Mat3::from_rotation_x(angle);
                let transformed_vertices: Vec<Vec3> = vertices
                    .iter()
                    .map(|v| rot_m * *v)
                    .collect();

                let projected_vertices: Vec<Vec2> = transformed_vertices
                    .iter()
                    .map(|v| project(*v))
                    .collect();
                
                // Draw figure
                for (i, tri) in indices.iter().enumerate() {
                    let a = &projected_vertices[tri[0]];
                    let b = &projected_vertices[tri[1]];
                    let c = &projected_vertices[tri[2]];
                    let az  = transformed_vertices[tri[0]].z;
                    let bz  = transformed_vertices[tri[1]].z;
                    let cz  = transformed_vertices[tri[2]].z;
                    draw_triangle(&mut pixels,  &mut depth_buffer,
                        a, b, c,
                        az, bz, cz,
                        &colors[i]);
                }

                pixels.render().unwrap();
                angle += 0.02;
            }
            _ => {}
        }
    });
    
}

fn project(v: Vec3) -> Vec2 {
    let x: f32 = (v.x + 1.0) * 0.5 * (WIDTH as f32);
    let y: f32 = (1.0 - (v.y + 1.0) * 0.5) * (HEIGHT as f32);
    
    Vec2 { x: (x), y: (y) }
}

fn draw_triangle(
    pixels: &mut Pixels, depth_buffer: &mut [f32],
    a: &Vec2, b: &Vec2, c: &Vec2,
    az: f32, bz: f32, cz: f32,
    color: &[u8; 4]) {

    let buffer = pixels.frame_mut();
    // Find bounding box for triangle
    let min_x = a.x.min(b.x).min(c.x).max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).min(WIDTH as f32 - 1.0) as i32;
    let min_y = a.y.min(b.y).min(c.y).max(0.0) as i32;
    let max_y = a.y.max(b.y).max(c.y).min(HEIGHT as f32 - 1.0) as i32;

    // Loop over bounding box pixels
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vec2::new(x as f32 +0.5, y as f32 +0.5);
            let (u, v, w) = calculate_barycentric(&p, &a, &b, &c);
            
            // Draw if the pixel is inside the triangle
            if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                // Interpolate depth
                let z = az * u + bz * v + cz * w;
                let idx = y as usize * WIDTH as usize + x as usize;
                if z < depth_buffer[idx] {
                    depth_buffer[idx] = z;
                    let pix_idx = idx * 4;
                    if pix_idx + 3 < buffer.len() {
                        buffer[pix_idx..pix_idx + 4].copy_from_slice(color);
                    }
                }
            }

        }
    }

}

fn calculate_barycentric(p: &Vec2, a: &Vec2, b: &Vec2, c:&Vec2) -> (f32, f32, f32)
{
    // Generic point inside the triangle in barycentric coordinates:
    // p = w*a + v*b + u*c, where w + v + u = 1
    let v0 = *b - *a;
    let v1 = *c - *a;
    let v2 = *p - *a;

    // Systems of eq to be solved:
    // ---------------------------
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);

    let denom = d00 * d11 - d01 * d01;
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    (u, v, w)

}


