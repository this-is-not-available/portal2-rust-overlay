use crate::custom::duck::rendering::{Model, Vertex, Triangle, UV};

fn parse_int_safe(s: &str) -> usize {
    s.parse::<usize>().unwrap_or(1)
}

fn parse_float_safe(s: &str) -> f32 {
    s.parse::<f32>().unwrap_or(1.0)
}

/// Parses a single face line (triangle) from OBJ.
/// Returns: [(vertex_idx, uv_idx)]
fn parse_triangle_face(parts: &[&str]) -> [(usize, usize); 3] {
    let mut indices = [(0, 0); 3];
    for (i, part) in parts.iter().enumerate().take(3) {
        let sub_parts: Vec<&str> = part.split('/').collect();
        let v_idx = parse_int_safe(sub_parts[0]) - 1;
        let uv_idx = if sub_parts.len() > 1 && !sub_parts[1].is_empty() {
            parse_int_safe(sub_parts[1]) - 1
        } else {
            1
        };
        indices[i] = (v_idx, uv_idx);
    }
    indices
}

/// Parses a face line, returns a vector of triangles (as index triplets)
fn parse_face(parts: &[&str]) -> Vec<[(usize, usize); 3]> {
    let n = parts.len();
    let mut faces = Vec::new();
    if n == 4 {
        // triangle
        faces.push(parse_triangle_face(&parts[1..4]));
    } else if n == 5 {
        // quad: split into two triangles
        // OBJ: f v1 v2 v3 v4
        // Triangles: (v1,v2,v3), (v1,v3,v4)
        faces.push(parse_triangle_face(&[parts[1], parts[2], parts[3]]));
        faces.push(parse_triangle_face(&[parts[1], parts[3], parts[4]]));
    }
    faces
}

/// Parses an OBJ file content into a Model (vertices, triangles, UVs)
pub fn parse_obj(file_content: &str) -> Model {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut triangles: Vec<Triangle> = Vec::new();
    let mut uvs: Vec<UV> = Vec::new();

    for line in file_content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { continue; }

        match parts[0] {
            "v" => {
                // Vertex
                if parts.len() >= 4 {
                    let x = parse_float_safe(parts[1]);
                    let y = parse_float_safe(parts[2]);
                    let z = parse_float_safe(parts[3]);
                    vertices.push(Vertex { x, y, z });
                }
            }
            "vt" => {
                if parts.len() >= 3 {
                    let u = parse_float_safe(parts[1]);
                    let v = 1.0 - parse_float_safe(parts[2]); // flip V as a preprocessing step instead of a runtime one
                   uvs.push(UV { u, v });
                }
            }
            "f" => {
                // Face
                for idxs in parse_face(&parts) {
                    if idxs.iter().all(|&(vi, _)| vi < vertices.len()) {
                        // Clamp or skip out-of-bounds UVs
                        let valid_uv = |uv_idx: usize| uv_idx < uvs.len();
                        if valid_uv(idxs[0].1) && valid_uv(idxs[1].1) && valid_uv(idxs[2].1) {
                            triangles.push(Triangle {
                                a: vertices[idxs[0].0].clone(),
                                b: vertices[idxs[1].0].clone(),
                                c: vertices[idxs[2].0].clone(),
                                u: idxs[0].1,
                                v: idxs[1].1,
                                w: idxs[2].1,
                            });
                        } else if uvs.len() > 0 {
                            triangles.push(Triangle {
                                a: vertices[idxs[0].0].clone(),
                                b: vertices[idxs[1].0].clone(),
                                c: vertices[idxs[2].0].clone(),
                                u: 0, // default to first UV if corrupted UV found
                                v: 0, // default to first UV if corrupted UV found
                                w: 0, // default to first UV if corrupted UV found
                            });
                        }
                        // Skip corrupted UVs when 0 replacement UVs found
                    }
                }
            }
            _ => {
                // Ignore other lines (vn, etc.)
            }
        }
    }

    Model {
        triangles,
        uvs,
        xrot: 0.0,
        yrot: 0.0,
        zrot: 0.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    }
}