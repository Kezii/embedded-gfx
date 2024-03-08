extern crate proc_macro;

use std::ops::Index;

use proc_macro::TokenStream;

#[proc_macro]
pub fn embed_stl(input: TokenStream) -> TokenStream {
    let file_path = input.to_string();

    let r = load_stl(file_path.trim_matches('"'));

    r.parse().unwrap()
}

fn load_stl(file_name: &str) -> String {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(file_name)
        .unwrap();
    let stl = stl_io::read_stl(&mut file).unwrap();

    let mut vertices = String::new();
    for vertex in stl.vertices {
        vertices += &format!(
            "[{}f32,{}f32,{}f32],",
            vertex.index(0),
            vertex.index(1),
            vertex.index(2)
        );
    }

    let mut faces = String::new();
    for triangle in &stl.faces {
        faces += &format!(
            "[{},{},{}],",
            triangle.vertices[0], triangle.vertices[1], triangle.vertices[2]
        );
    }

    let mut normals = String::new();
    for triangle in &stl.faces {
        normals += &format!(
            "[{}f32,{}f32,{}f32],",
            triangle.normal.index(0),
            triangle.normal.index(1),
            triangle.normal.index(2)
        );
    }

    let lines = embedded_gfx::mesh::Geometry::lines_from_faces(
        &stl.faces
            .iter()
            .map(|f| [f.vertices[0], f.vertices[1], f.vertices[2]])
            .collect::<Vec<_>>(),
    );

    let mut lines_ = String::new();
    for line in lines {
        lines_ += &format!("[{},{}],", line.0, line.1);
    }

    let mut ret: String = String::new();

    ret += &format!(
        "Geometry {{
        vertices: &[
            {vertices}
        ],
        faces: &[
            {faces}
        ],
        colors: &[],
        lines: &[
            {lines_}
        ],
        normals: &[
            {normals}
        ],
    }}"
    );

    ret
}
